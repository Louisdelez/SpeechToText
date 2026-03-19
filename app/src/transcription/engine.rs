use std::sync::mpsc;
use std::thread;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::transcription::model;
use crate::translation;
use crate::types::{AppMode, ComputeDevice, DeviceConfig, KokoroConfig, Language, PromptLength, TranslationEngine, TtsEngine, WritingStyle};

pub enum WorkerRequest {
    Transcribe {
        samples: Vec<f32>,
        duration_secs: f32,
    },
    TranslateText(String),
    SwitchDevice(ComputeDevice),
    SetLanguage(Language),
    SetMode {
        mode: AppMode,
        source_lang: Language,
        target_lang: Language,
        engine: TranslationEngine,
        prompt_length: PromptLength,
        writing_style: WritingStyle,
        corrector_lang: Language,
        tts_engine: TtsEngine,
        tts_lang: Language,
        kokoro: KokoroConfig,
        device_config: DeviceConfig,
    },
}

#[derive(Debug, Clone)]
pub enum WorkerResponse {
    ModelDownloading { progress: u8 },
    ModelReady,
    ModelError(String),
    TranscriptionResult {
        text: String,
        translated: Option<String>,
        duration_secs: f32,
    },
    TranscriptionError(String),
    DeviceSwitched(ComputeDevice),
    LanguageChanged(Language),
    StatusUpdate(String),
}

fn do_translate(
    text: &str,
    src: Language,
    tgt: Language,
    engine: TranslationEngine,
    use_gpu: bool,
    status_tx: &mpsc::Sender<WorkerResponse>,
) -> Result<String, String> {
    match engine {
        TranslationEngine::Fast => {
            // Opus-MT via ct2rs
            if !translation::opus::opus_model_exists(src, tgt) {
                let _ = status_tx.send(WorkerResponse::StatusUpdate(
                    "Telechargement Opus-MT...".to_string(),
                ));
                translation::opus::download_opus_model(src, tgt, |_| {})?;
            }
            translation::opus::translate_opus(text, src, tgt)
        }
        TranslationEngine::Smart => {
            // LLM via subprocess
            if !translation::model::llm_model_exists() {
                let _ = status_tx.send(WorkerResponse::StatusUpdate(
                    "Telechargement du modele LLM...".to_string(),
                ));
                translation::model::download_llm_model(|_| {})?;
            }
            let llm_path = translation::model::llm_model_path();
            translation::llm::translate_with_llm(
                text,
                src,
                tgt,
                llm_path.to_str().unwrap(),
                use_gpu,
            )
        }
    }
}

fn do_summarize(
    text: &str,
    lang: Language,
    use_gpu: bool,
    status_tx: &mpsc::Sender<WorkerResponse>,
) -> Result<String, String> {
    if !translation::model::llm_model_exists() {
        let _ = status_tx.send(WorkerResponse::StatusUpdate(
            "Telechargement du modele LLM...".to_string(),
        ));
        translation::model::download_llm_model(|_| {})?;
    }
    let llm_path = translation::model::llm_model_path();
    translation::llm::summarize_with_llm(text, lang, llm_path.to_str().unwrap(), use_gpu)
}

fn do_prompt_engineer(
    text: &str,
    lang: Language,
    prompt_length: PromptLength,
    use_gpu: bool,
    status_tx: &mpsc::Sender<WorkerResponse>,
) -> Result<String, String> {
    if !translation::model::llm_model_exists() {
        let _ = status_tx.send(WorkerResponse::StatusUpdate(
            "Telechargement du modele LLM...".to_string(),
        ));
        translation::model::download_llm_model(|_| {})?;
    }
    let llm_path = translation::model::llm_model_path();
    translation::llm::prompt_engineer_with_llm(text, prompt_length, llm_path.to_str().unwrap(), use_gpu)
}

fn do_tts(
    text: &str,
    engine: TtsEngine,
    lang: Language,
    use_gpu: bool,
    kokoro: &KokoroConfig,
    status_tx: &mpsc::Sender<WorkerResponse>,
) -> Result<String, String> {
    let _ = status_tx.send(WorkerResponse::StatusUpdate(
        "Generation audio en cours...".to_string(),
    ));

    let audio_path = crate::tts::engine::generate_speech(text, engine, lang, use_gpu, kokoro)?;

    // Play audio in background thread
    crate::tts::player::play_wav_async(audio_path);

    Ok("Audio genere".to_string())
}

fn do_correct(
    text: &str,
    lang: Language,
    style: WritingStyle,
    use_gpu: bool,
    status_tx: &mpsc::Sender<WorkerResponse>,
) -> Result<String, String> {
    if !translation::model::llm_model_exists() {
        let _ = status_tx.send(WorkerResponse::StatusUpdate(
            "Telechargement du modele LLM...".to_string(),
        ));
        translation::model::download_llm_model(|_| {})?;
    }
    let llm_path = translation::model::llm_model_path();
    translation::llm::correct_with_llm(text, lang, style, llm_path.to_str().unwrap(), use_gpu)
}

pub struct TranscriptionWorker {
    tx: mpsc::Sender<WorkerRequest>,
}

impl TranscriptionWorker {
    pub fn spawn(
        response_tx: mpsc::Sender<WorkerResponse>,
        device: ComputeDevice,
        language: Language,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<WorkerRequest>();

        thread::spawn(move || {
            let _ = response_tx.send(WorkerResponse::ModelDownloading { progress: 0 });

            let model_path = match model::download_model(|pct| {
                let _ = response_tx.send(WorkerResponse::ModelDownloading { progress: pct });
            }) {
                Ok(p) => p,
                Err(e) => {
                    let _ = response_tx.send(WorkerResponse::ModelError(e));
                    return;
                }
            };

            let mut ctx = match load_whisper(&model_path, device) {
                Ok(c) => c,
                Err(e) => {
                    let _ = response_tx.send(WorkerResponse::ModelError(e));
                    return;
                }
            };

            let _ = response_tx.send(WorkerResponse::ModelReady);

            let mut current_lang = language;
            let mut current_device = device;
            let mut current_mode = AppMode::SpeechToText;
            let mut source_lang = Language::Fr;
            let mut target_lang = Language::En;
            let mut engine = TranslationEngine::Fast;
            let mut prompt_length = PromptLength::Medium;
            let mut writing_style = WritingStyle::Authentic;
            let mut corrector_lang = Language::Fr;
            let mut tts_engine = TtsEngine::Kokoro;
            let mut tts_lang = Language::Fr;
            let mut kokoro_cfg = KokoroConfig::default();
            let mut dev_cfg = DeviceConfig::default();

            while let Ok(req) = rx.recv() {
                match req {
                    WorkerRequest::Transcribe {
                        samples,
                        duration_secs,
                    } => {
                        let whisper_lang = if current_mode == AppMode::Translation {
                            source_lang
                        } else {
                            current_lang
                        };

                        let result = run_transcription(&ctx, &samples, whisper_lang);
                        match result {
                            Ok(text) => {
                                let processed = match current_mode {
                                    AppMode::Translation if source_lang != target_lang => {
                                        let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                            "Traduction en cours...".to_string(),
                                        ));
                                        Some(match do_translate(&text, source_lang, target_lang, engine, dev_cfg.llm_use_gpu(), &response_tx) {
                                            Ok(t) => t,
                                            Err(e) => format!("[Erreur: {e}]"),
                                        })
                                    }
                                    AppMode::Summary => {
                                        let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                            "Resume en cours...".to_string(),
                                        ));
                                        Some(match do_summarize(&text, current_lang, dev_cfg.llm_use_gpu(), &response_tx) {
                                            Ok(t) => t,
                                            Err(e) => format!("[Erreur: {e}]"),
                                        })
                                    }
                                    AppMode::PromptEngineer => {
                                        let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                            "Generation du prompt...".to_string(),
                                        ));
                                        Some(match do_prompt_engineer(&text, current_lang, prompt_length, dev_cfg.llm_use_gpu(), &response_tx) {
                                            Ok(t) => t,
                                            Err(e) => format!("[Erreur: {e}]"),
                                        })
                                    }
                                    AppMode::Corrector => {
                                        let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                            "Correction en cours...".to_string(),
                                        ));
                                        Some(match do_correct(&text, corrector_lang, writing_style, dev_cfg.llm_use_gpu(), &response_tx) {
                                            Ok(t) => t,
                                            Err(e) => format!("[Erreur: {e}]"),
                                        })
                                    }
                                    AppMode::TextToSpeech => {
                                        Some(match do_tts(&text, tts_engine, tts_lang, dev_cfg.tts_use_gpu(), &kokoro_cfg, &response_tx) {
                                            Ok(t) => t,
                                            Err(e) => format!("[Erreur: {e}]"),
                                        })
                                    }
                                    _ => None,
                                };

                                let _ = response_tx.send(
                                    WorkerResponse::TranscriptionResult {
                                        text,
                                        translated: processed,
                                        duration_secs,
                                    },
                                );
                            }
                            Err(e) => {
                                let _ =
                                    response_tx.send(WorkerResponse::TranscriptionError(e));
                            }
                        }
                    }
                    WorkerRequest::TranslateText(text) => {
                        eprintln!("[DEBUG] TranslateText received: mode={:?} text_len={}", current_mode, text.len());
                        let result = match current_mode {
                            AppMode::Translation => {
                                let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                    "Traduction en cours...".to_string(),
                                ));
                                match do_translate(&text, source_lang, target_lang, engine, dev_cfg.llm_use_gpu(), &response_tx) {
                                    Ok(t) => t,
                                    Err(e) => format!("[Erreur: {e}]"),
                                }
                            }
                            AppMode::Summary => {
                                let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                    "Resume en cours...".to_string(),
                                ));
                                match do_summarize(&text, current_lang, dev_cfg.llm_use_gpu(), &response_tx) {
                                    Ok(t) => t,
                                    Err(e) => format!("[Erreur: {e}]"),
                                }
                            }
                            AppMode::PromptEngineer => {
                                let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                    "Generation du prompt...".to_string(),
                                ));
                                match do_prompt_engineer(&text, current_lang, prompt_length, dev_cfg.llm_use_gpu(), &response_tx) {
                                    Ok(t) => t,
                                    Err(e) => format!("[Erreur: {e}]"),
                                }
                            }
                            AppMode::Corrector => {
                                let _ = response_tx.send(WorkerResponse::StatusUpdate(
                                    "Correction en cours...".to_string(),
                                ));
                                match do_correct(&text, corrector_lang, writing_style, dev_cfg.llm_use_gpu(), &response_tx) {
                                    Ok(t) => t,
                                    Err(e) => format!("[Erreur: {e}]"),
                                }
                            }
                            AppMode::TextToSpeech => {
                                match do_tts(&text, tts_engine, tts_lang, dev_cfg.tts_use_gpu(), &kokoro_cfg, &response_tx) {
                                    Ok(t) => t,
                                    Err(e) => format!("[Erreur: {e}]"),
                                }
                            }
                            _ => text.clone(),
                        };

                        eprintln!("[DEBUG] Sending result, translated_len={}", result.len());
                        let _ = response_tx.send(WorkerResponse::TranscriptionResult {
                            text,
                            translated: Some(result),
                            duration_secs: 0.0,
                        });
                    }
                    WorkerRequest::SwitchDevice(new_device) => {
                        match load_whisper(&model_path, new_device) {
                            Ok(c) => {
                                ctx = c;
                                current_device = new_device;
                                let _ = response_tx
                                    .send(WorkerResponse::DeviceSwitched(new_device));
                            }
                            Err(e) => {
                                let _ = response_tx.send(WorkerResponse::ModelError(e));
                            }
                        }
                    }
                    WorkerRequest::SetLanguage(lang) => {
                        current_lang = lang;
                        let _ = response_tx.send(WorkerResponse::LanguageChanged(lang));
                    }
                    WorkerRequest::SetMode {
                        mode,
                        source_lang: src,
                        target_lang: tgt,
                        engine: eng,
                        prompt_length: pl,
                        writing_style: ws,
                        corrector_lang: cl,
                        tts_engine: te,
                        tts_lang: tl,
                        kokoro: kc,
                        device_config: dc,
                    } => {
                        current_mode = mode;
                        source_lang = src;
                        target_lang = tgt;
                        engine = eng;
                        prompt_length = pl;
                        writing_style = ws;
                        corrector_lang = cl;
                        tts_engine = te;
                        tts_lang = tl;
                        kokoro_cfg = kc;
                        dev_cfg = dc;
                    }
                }
            }
        });

        TranscriptionWorker { tx }
    }

    pub fn transcribe(&self, samples: Vec<f32>, duration_secs: f32) {
        let _ = self.tx.send(WorkerRequest::Transcribe {
            samples,
            duration_secs,
        });
    }

    pub fn switch_device(&self, device: ComputeDevice) {
        let _ = self.tx.send(WorkerRequest::SwitchDevice(device));
    }

    pub fn translate_text(&self, text: String) {
        let _ = self.tx.send(WorkerRequest::TranslateText(text));
    }

    pub fn set_language(&self, lang: Language) {
        let _ = self.tx.send(WorkerRequest::SetLanguage(lang));
    }

    pub fn set_mode(
        &self,
        mode: AppMode,
        source_lang: Language,
        target_lang: Language,
        engine: TranslationEngine,
        prompt_length: PromptLength,
        writing_style: WritingStyle,
        corrector_lang: Language,
        tts_engine: TtsEngine,
        tts_lang: Language,
        kokoro: KokoroConfig,
        device_config: DeviceConfig,
    ) {
        let _ = self.tx.send(WorkerRequest::SetMode {
            mode,
            source_lang,
            target_lang,
            engine,
            prompt_length,
            writing_style,
            corrector_lang,
            tts_engine,
            tts_lang,
            kokoro,
            device_config,
        });
    }
}

fn load_whisper(
    model_path: &std::path::Path,
    device: ComputeDevice,
) -> Result<WhisperContext, String> {
    let mut params = WhisperContextParameters::default();
    params.use_gpu(device.use_gpu());

    WhisperContext::new_with_params(model_path.to_str().unwrap(), params)
        .map_err(|e| format!("Load model: {e}"))
}

fn run_transcription(
    ctx: &WhisperContext,
    samples: &[f32],
    lang: Language,
) -> Result<String, String> {
    let mut state = ctx.create_state().map_err(|e| format!("Create state: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(lang.whisper_code()));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);

    state
        .full(params, samples)
        .map_err(|e| format!("Transcription failed: {e}"))?;

    let num_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(s) = segment.to_str_lossy() {
                text.push_str(&s);
            }
        }
    }

    Ok(text.trim().to_string())
}
