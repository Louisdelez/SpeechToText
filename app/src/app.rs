use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use iced::widget::{button, column, container, pick_list, row, scrollable, text, Space};
use iced::{Alignment, Element, Length, Task};

use crate::audio::decoder;
use crate::audio::recorder::Recorder;
use crate::clipboard;
use crate::history;
use crate::i18n;
use crate::models::{self, ModelEvent, ModelId};
use crate::transcription::engine::{TranscriptionWorker, WorkerResponse};
use crate::types::*;
use crate::ui::{controls, empty_state, history as history_view, icons, message_bubble, settings, theme};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleRecording,
    ImportFile,
    FileSelected(Option<(Vec<f32>, f32, String)>),
    CopyText(usize),
    CopyHistoryText(usize),
    CheckWorker,
    ToggleDevice,
    SetGlobalDevice(DeviceChoice),
    SetWhisperDevice(DeviceChoice),
    SetOpusFrEnDevice(DeviceChoice),
    SetOpusEnFrDevice(DeviceChoice),
    SetQwenDevice(DeviceChoice),
    SetKokoroDevice(DeviceChoice),
    ToggleSettings,
    ToggleHistory,
    ClearHistory,
    SetIcon,
    SetLanguage(Language),
    ToggleLangDropdown,
    SetMode(AppMode),
    SetSourceLang(Language),
    SetTargetLang(Language),
    SetEngine(TranslationEngine),
    SetPromptLength(PromptLength),
    SetWritingStyle(WritingStyle),
    SetCorrectorLang(Language),
    SetTtsEngine(TtsEngine),
    SetTtsLang(Language),
    SetKokoroVoice(KokoroVoice),
    SetKokoroSpeed(KokoroSpeed),
    ToggleKokoroBlend,
    SetKokoroBlendVoice(KokoroVoice),
    SetKokoroBlendRatio(KokoroBlendRatio),
    DownloadModel(ModelId),
    DeleteModel(ModelId),
    RedownloadModel(ModelId),
    TextInputChanged(String),
    SubmitTextInput,
    OpenConversation(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum View {
    Main,
    Settings,
    History,
}

pub struct App {
    conversations: Vec<Conversation>,
    current_conv_idx: Option<usize>,
    recording_state: RecordingState,
    model_state: ModelState,
    recorder: Option<Recorder>,
    recording_start: Option<Instant>,
    elapsed_secs: f32,
    worker: Option<TranscriptionWorker>,
    worker_rx: Option<mpsc::Receiver<WorkerResponse>>,
    device: ComputeDevice,
    switching_device: bool,
    device_config: DeviceConfig,
    current_view: View,
    language: Language,
    show_lang_dropdown: bool,
    app_mode: AppMode,
    source_lang: Language,
    target_lang: Language,
    translation_engine: TranslationEngine,
    prompt_length: PromptLength,
    writing_style: WritingStyle,
    corrector_lang: Language,
    tts_engine: TtsEngine,
    tts_lang: Language,
    kokoro: KokoroConfig,
    status_text: Option<String>,
    text_input_value: String,
    audio_levels: Vec<f32>,
    spinner_tick: u32,
    model_op_tx: mpsc::Sender<ModelEvent>,
    model_op_rx: Option<mpsc::Receiver<ModelEvent>>,
    model_downloads: HashMap<ModelId, u8>,
}

impl App {
    fn current_messages(&self) -> &[ChatMessage] {
        match self.current_conv_idx {
            Some(idx) => &self.conversations[idx].messages,
            None => &[],
        }
    }

    fn ensure_conversation(&mut self) {
        // Create a new conversation if none exists for current mode
        let needs_new = match self.current_conv_idx {
            Some(idx) => self.conversations[idx].mode != self.app_mode,
            None => true,
        };
        if needs_new {
            // Look for existing conversation for this mode
            let existing = self.conversations.iter().position(|c| c.mode == self.app_mode);
            if let Some(idx) = existing {
                self.current_conv_idx = Some(idx);
            } else {
                self.conversations.push(Conversation {
                    mode: self.app_mode,
                    messages: Vec::new(),
                    created_at: chrono::Local::now(),
                });
                self.current_conv_idx = Some(self.conversations.len() - 1);
            }
        }
    }

    fn push_message(&mut self, role: MessageRole, text: String) {
        self.ensure_conversation();
        if let Some(idx) = self.current_conv_idx {
            self.conversations[idx].messages.push(ChatMessage {
                role,
                text,
                timestamp: chrono::Local::now(),
            });
            history::save(&self.conversations);
        }
    }

    pub fn new() -> (Self, Task<Message>) {
        let (resp_tx, resp_rx) = mpsc::channel();
        let (model_op_tx, model_op_rx) = mpsc::channel();
        let device = ComputeDevice::Gpu;
        let language = Language::Fr;
        let worker = TranscriptionWorker::spawn(resp_tx, device, language);
        let conversations = history::load();

        let app = App {
            conversations,
            current_conv_idx: None,
            recording_state: RecordingState::Idle,
            model_state: ModelState::NotDownloaded,
            recorder: None,
            recording_start: None,
            elapsed_secs: 0.0,
            worker: Some(worker),
            worker_rx: Some(resp_rx),
            device,
            switching_device: false,
            device_config: DeviceConfig::default(),
            current_view: View::Main,
            language,
            show_lang_dropdown: false,
            app_mode: AppMode::SpeechToText,
            source_lang: Language::Fr,
            target_lang: Language::En,
            translation_engine: TranslationEngine::Fast,
            prompt_length: PromptLength::Medium,
            writing_style: WritingStyle::Authentic,
            corrector_lang: Language::Fr,
            tts_engine: TtsEngine::Kokoro,
            tts_lang: Language::Fr,
            kokoro: KokoroConfig::default(),
            status_text: None,
            text_input_value: String::new(),
            audio_levels: Vec::new(),
            spinner_tick: 0,
            model_op_tx,
            model_op_rx: Some(model_op_rx),
            model_downloads: HashMap::new(),
        };

        (app, set_window_icon())
    }

    fn sync_mode(&self) {
        if let Some(worker) = &self.worker {
            worker.set_mode(
                self.app_mode,
                self.source_lang,
                self.target_lang,
                self.translation_engine,
                self.prompt_length,
                self.writing_style,
                self.corrector_lang,
                self.tts_engine,
                self.tts_lang,
                self.kokoro,
                self.device_config,
            );
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleRecording => {
                match self.recording_state {
                    RecordingState::Idle => {
                        match Recorder::start() {
                            Ok(rec) => {
                                self.recorder = Some(rec);
                                self.recording_state = RecordingState::Recording;
                                self.recording_start = Some(Instant::now());
                                self.elapsed_secs = 0.0;
                                self.status_text = None;
                            }
                            Err(e) => eprintln!("Failed to start recording: {e}"),
                        }
                    }
                    RecordingState::Recording => {
                        if let Some(rec) = self.recorder.take() {
                            let (samples, duration) = rec.stop();
                            self.recording_state = RecordingState::Processing;
                            self.recording_start = None;
                            if let Some(worker) = &self.worker {
                                worker.transcribe(samples, duration);
                            }
                        }
                    }
                    RecordingState::Processing | RecordingState::Translating => {}
                }
                Task::none()
            }

            Message::ImportFile => {
                Task::perform(
                    async {
                        let handle = rfd::AsyncFileDialog::new()
                            .add_filter("Audio", &["mp3", "wav", "flac", "ogg", "m4a", "aac"])
                            .pick_file()
                            .await;
                        match handle {
                            Some(file) => {
                                let path = file.path().to_path_buf();
                                let name = file.file_name();
                                match decoder::decode_file(&path) {
                                    Ok((samples, dur)) => Some((samples, dur, name)),
                                    Err(e) => { eprintln!("Decode error: {e}"); None }
                                }
                            }
                            None => None,
                        }
                    },
                    Message::FileSelected,
                )
            }

            Message::FileSelected(result) => {
                if let Some((samples, duration, _)) = result {
                    self.recording_state = RecordingState::Processing;
                    if let Some(worker) = &self.worker {
                        worker.transcribe(samples, duration);
                    }
                }
                Task::none()
            }

            Message::CopyText(idx) => {
                let msgs = self.current_messages();
                if let Some(msg) = msgs.get(idx) {
                    let _ = clipboard::copy_to_clipboard(&msg.text);
                }
                Task::none()
            }

            Message::CopyHistoryText(idx) => {
                // Copy from a specific conversation in history view
                // idx encodes conv_idx * 10000 + msg_idx
                let conv_idx = idx / 10000;
                let msg_idx = idx % 10000;
                if let Some(conv) = self.conversations.get(conv_idx) {
                    if let Some(msg) = conv.messages.get(msg_idx) {
                        let _ = clipboard::copy_to_clipboard(&msg.text);
                    }
                }
                Task::none()
            }

            Message::ToggleDevice => {
                if !self.switching_device && self.recording_state == RecordingState::Idle {
                    let new_device = match self.device {
                        ComputeDevice::Cpu => ComputeDevice::Gpu,
                        ComputeDevice::Gpu => ComputeDevice::Cpu,
                    };
                    self.switching_device = true;
                    if let Some(worker) = &self.worker {
                        worker.switch_device(new_device);
                    }
                }
                Task::none()
            }

            Message::SetGlobalDevice(choice) => {
                self.device_config.global = choice;
                // Also switch Whisper if it's on Auto
                if self.device_config.whisper == DeviceChoice::Auto {
                    let new_dev = self.device_config.whisper_device();
                    if new_dev != self.device && !self.switching_device && self.recording_state == RecordingState::Idle {
                        self.switching_device = true;
                        if let Some(worker) = &self.worker { worker.switch_device(new_dev); }
                    }
                }
                self.sync_mode();
                Task::none()
            }

            Message::SetWhisperDevice(choice) => {
                self.device_config.whisper = choice;
                let new_dev = self.device_config.whisper_device();
                if new_dev != self.device && !self.switching_device && self.recording_state == RecordingState::Idle {
                    self.switching_device = true;
                    if let Some(worker) = &self.worker { worker.switch_device(new_dev); }
                }
                Task::none()
            }

            Message::SetOpusFrEnDevice(choice) => { self.device_config.opus_fr_en = choice; self.sync_mode(); Task::none() }
            Message::SetOpusEnFrDevice(choice) => { self.device_config.opus_en_fr = choice; self.sync_mode(); Task::none() }
            Message::SetQwenDevice(choice) => { self.device_config.qwen = choice; self.sync_mode(); Task::none() }
            Message::SetKokoroDevice(choice) => { self.device_config.kokoro = choice; self.sync_mode(); Task::none() }

            Message::ToggleSettings => {
                self.current_view = if self.current_view == View::Settings { View::Main } else { View::Settings };
                Task::none()
            }

            Message::ToggleHistory => {
                self.current_view = if self.current_view == View::History { View::Main } else { View::History };
                Task::none()
            }

            Message::ClearHistory => {
                self.conversations.clear();
                self.current_conv_idx = None;
                history::save(&self.conversations);
                Task::none()
            }

            Message::SetIcon => {
                let icon = make_icon();
                let icon = std::sync::Arc::new(std::sync::Mutex::new(Some(icon)));
                iced::window::oldest().and_then(move |id| {
                    let taken = icon.lock().unwrap().take();
                    if let Some(ic) = taken { iced::window::set_icon(id, ic) } else { Task::none() }
                })
            }

            Message::SetLanguage(lang) => {
                self.show_lang_dropdown = false;
                if lang != self.language {
                    self.language = lang;
                    if let Some(worker) = &self.worker { worker.set_language(lang); }
                }
                Task::none()
            }

            Message::ToggleLangDropdown => {
                self.show_lang_dropdown = !self.show_lang_dropdown;
                Task::none()
            }

            Message::OpenConversation(idx) => {
                if idx < self.conversations.len() {
                    self.current_conv_idx = Some(idx);
                    self.app_mode = self.conversations[idx].mode;
                    self.sync_mode();
                    self.current_view = View::Main;
                }
                Task::none()
            }

            Message::SetMode(mode) => {
                self.app_mode = mode;
                // Always create a new conversation when switching mode
                self.conversations.push(Conversation {
                    mode,
                    messages: Vec::new(),
                    created_at: chrono::Local::now(),
                });
                self.current_conv_idx = Some(self.conversations.len() - 1);
                self.sync_mode();
                Task::none()
            }

            Message::SetSourceLang(lang) => { self.source_lang = lang; self.sync_mode(); Task::none() }
            Message::SetTargetLang(lang) => { self.target_lang = lang; self.sync_mode(); Task::none() }
            Message::SetEngine(eng) => { self.translation_engine = eng; self.sync_mode(); Task::none() }
            Message::SetPromptLength(len) => { self.prompt_length = len; self.sync_mode(); Task::none() }
            Message::SetWritingStyle(style) => { self.writing_style = style; self.sync_mode(); Task::none() }
            Message::SetCorrectorLang(lang) => { self.corrector_lang = lang; self.sync_mode(); Task::none() }
            Message::SetTtsEngine(eng) => { self.tts_engine = eng; self.sync_mode(); Task::none() }
            Message::SetTtsLang(lang) => { self.tts_lang = lang; self.sync_mode(); Task::none() }
            Message::SetKokoroVoice(v) => { self.kokoro.voice = v; self.sync_mode(); Task::none() }
            Message::SetKokoroSpeed(s) => { self.kokoro.speed = s; self.sync_mode(); Task::none() }
            Message::ToggleKokoroBlend => { self.kokoro.blend_enabled = !self.kokoro.blend_enabled; self.sync_mode(); Task::none() }
            Message::SetKokoroBlendVoice(v) => { self.kokoro.blend_voice = v; self.sync_mode(); Task::none() }
            Message::SetKokoroBlendRatio(r) => { self.kokoro.blend_ratio = r; self.sync_mode(); Task::none() }

            Message::DownloadModel(id) => {
                if !self.model_downloads.contains_key(&id) {
                    self.model_downloads.insert(id, 0);
                    models::download_model_async(id, self.model_op_tx.clone());
                }
                Task::none()
            }

            Message::DeleteModel(id) => {
                if !self.model_downloads.contains_key(&id) {
                    if let Err(e) = models::delete_model(id) {
                        eprintln!("[Models] Delete error: {e}");
                    }
                }
                Task::none()
            }

            Message::RedownloadModel(id) => {
                if !self.model_downloads.contains_key(&id) {
                    let _ = models::delete_model(id);
                    self.model_downloads.insert(id, 0);
                    models::download_model_async(id, self.model_op_tx.clone());
                }
                Task::none()
            }

            Message::TextInputChanged(val) => { self.text_input_value = val; Task::none() }

            Message::SubmitTextInput => {
                let input = self.text_input_value.trim().to_string();
                if !input.is_empty() && self.recording_state == RecordingState::Idle {
                    self.text_input_value.clear();
                    self.push_message(MessageRole::User, input.clone());

                    match self.app_mode {
                        AppMode::Translation if self.source_lang != self.target_lang => {
                            self.recording_state = RecordingState::Translating;
                            if let Some(worker) = &self.worker { worker.translate_text(input); }
                        }
                        AppMode::Summary | AppMode::PromptEngineer | AppMode::Corrector | AppMode::TextToSpeech => {
                            self.recording_state = RecordingState::Translating;
                            if let Some(worker) = &self.worker { worker.translate_text(input); }
                        }
                        _ => {
                            // Speech to Text: just keep the user message
                        }
                    }
                }
                Task::none()
            }

            Message::CheckWorker => {
                if let Some(start) = self.recording_start {
                    self.elapsed_secs = start.elapsed().as_secs_f32();
                }

                self.spinner_tick = self.spinner_tick.wrapping_add(1);

                if let Some(ref recorder) = self.recorder {
                    let (_, history) = recorder.get_levels();
                    self.audio_levels = history;
                } else if !self.audio_levels.is_empty() {
                    self.audio_levels.clear();
                }

                // Poll model download events
                if let Some(ref rx) = self.model_op_rx {
                    while let Ok(evt) = rx.try_recv() {
                        match evt {
                            ModelEvent::Progress { id, pct } => {
                                self.model_downloads.insert(id, pct);
                            }
                            ModelEvent::Complete(id) => {
                                self.model_downloads.remove(&id);
                            }
                            ModelEvent::Error { id, error } => {
                                eprintln!("[Models] Download error for {:?}: {}", id, error);
                                self.model_downloads.remove(&id);
                            }
                        }
                    }
                }

                // Collect responses first to avoid borrow conflict
                let responses: Vec<WorkerResponse> = if let Some(rx) = &self.worker_rx {
                    let mut v = Vec::new();
                    while let Ok(resp) = rx.try_recv() { v.push(resp); }
                    v
                } else {
                    Vec::new()
                };

                for resp in responses {
                    match resp {
                        WorkerResponse::ModelDownloading { progress } => {
                            self.model_state = ModelState::Downloading { progress_pct: progress };
                        }
                        WorkerResponse::ModelReady => { self.model_state = ModelState::Ready; }
                        WorkerResponse::ModelError(e) => { self.model_state = ModelState::Error(e); }
                        WorkerResponse::TranscriptionResult { text, translated, duration_secs: _ } => {
                            self.recording_state = RecordingState::Idle;
                            self.status_text = None;

                            if !text.is_empty() {
                                if self.app_mode == AppMode::SpeechToText {
                                    self.push_message(MessageRole::Assistant, text);
                                } else {
                                    // Only push User message if not already there (from text input)
                                    let already_pushed = self.current_conv_idx
                                        .and_then(|idx| self.conversations[idx].messages.last())
                                        .map(|m| m.role == MessageRole::User && m.text == text)
                                        .unwrap_or(false);
                                    if !already_pushed {
                                        self.push_message(MessageRole::User, text);
                                    }
                                    if let Some(tr) = translated {
                                        if !tr.is_empty() {
                                            self.push_message(MessageRole::Assistant, tr);
                                        }
                                    }
                                }
                            }
                        }
                        WorkerResponse::TranscriptionError(e) => {
                            self.recording_state = RecordingState::Idle;
                            self.status_text = None;
                            self.push_message(MessageRole::Assistant, format!("[Erreur: {e}]"));
                        }
                        WorkerResponse::DeviceSwitched(dev) => {
                            self.device = dev;
                            self.switching_device = false;
                        }
                        WorkerResponse::LanguageChanged(lang) => { self.language = lang; }
                        WorkerResponse::StatusUpdate(s) => {
                            self.status_text = Some(s);
                            self.recording_state = RecordingState::Translating;
                        }
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let s = i18n::t(self.language);
        let inner = match self.current_view {
            View::Main => self.view_main(s),
            View::Settings => settings::view(&self.device, self.switching_device, &self.device_config, &self.language, self.show_lang_dropdown, &self.kokoro, &self.model_downloads, s),
            View::History => history_view::view(&self.conversations, s),
        };

        container(inner)
            .max_width(480)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .padding([0, 12])
            .style(theme::main_container)
            .into()
    }

    fn view_main(&self, s: &'static i18n::Strings) -> Element<'_, Message> {
        // Header
        let history_btn = button(icons::icon(icons::HISTORY, 18.0).center().style(theme::muted_text))
            .padding([6, 8]).style(theme::gear_button).on_press(Message::ToggleHistory);

        let mode_picker = pick_list(AppMode::ALL, Some(self.app_mode), Message::SetMode)
            .text_size(14.0).padding([8, 14]).style(theme::mode_pick_list).menu_style(theme::dark_menu);

        let gear_btn = button(icons::icon(icons::SETTINGS, 18.0).center().style(theme::muted_text))
            .padding([6, 8]).style(theme::gear_button).on_press(Message::ToggleSettings);

        let header = container(
            row![history_btn, Space::new().width(Length::Fill), mode_picker, Space::new().width(Length::Fill), gear_btn]
                .align_y(Alignment::Center).width(Length::Fill),
        ).width(Length::Fill).padding([12, 8]).style(theme::header_container);

        // Translation lang bar
        let lang_bar: Option<Element<'_, Message>> = if self.app_mode == AppMode::Translation {
            let src = pick_list(Language::ALL, Some(self.source_lang), Message::SetSourceLang)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);
            let tgt = pick_list(Language::ALL, Some(self.target_lang), Message::SetTargetLang)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);
            let eng = pick_list(TranslationEngine::ALL, Some(self.translation_engine), Message::SetEngine)
                .text_size(12.0).padding([6, 12]).style(theme::lang_pick_list).menu_style(theme::dark_menu);

            Some(container(
                row![src, text("→").size(16).style(theme::muted_text), tgt, Space::new().width(Length::Fill), eng]
                    .spacing(10).align_y(Alignment::Center),
            ).width(Length::Fill).padding([8, 16]).into())
        } else {
            None
        };

        // Corrector bar
        let corrector_bar: Option<Element<'_, Message>> = if self.app_mode == AppMode::Corrector {
            let lang_picker = pick_list(Language::ALL, Some(self.corrector_lang), Message::SetCorrectorLang)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);
            let style_picker = pick_list(WritingStyle::ALL, Some(self.writing_style), Message::SetWritingStyle)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);

            Some(container(
                row![lang_picker, Space::new().width(Length::Fill), style_picker]
                    .spacing(10).align_y(Alignment::Center),
            ).width(Length::Fill).padding([8, 16]).into())
        } else {
            None
        };

        // TTS bar
        let tts_bar: Option<Element<'_, Message>> = if self.app_mode == AppMode::TextToSpeech {
            let lang_picker = pick_list(Language::ALL, Some(self.tts_lang), Message::SetTtsLang)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);
            let engine_picker = pick_list(TtsEngine::ALL, Some(self.tts_engine), Message::SetTtsEngine)
                .text_size(12.0).padding([6, 12]).style(theme::lang_pick_list).menu_style(theme::dark_menu);

            Some(container(
                row![lang_picker, Space::new().width(Length::Fill), engine_picker]
                    .spacing(10).align_y(Alignment::Center),
            ).width(Length::Fill).padding([8, 16]).into())
        } else {
            None
        };

        // Prompt length bar
        let prompt_bar: Option<Element<'_, Message>> = if self.app_mode == AppMode::PromptEngineer {
            let length_picker = pick_list(PromptLength::ALL, Some(self.prompt_length), Message::SetPromptLength)
                .text_size(13.0).padding([8, 14]).style(theme::lang_pick_list).menu_style(theme::dark_menu);

            Some(container(
                row![text("Longueur :").size(13).style(theme::muted_text), length_picker]
                    .spacing(8).align_y(Alignment::Center),
            ).width(Length::Fill).padding([8, 16]).into())
        } else {
            None
        };

        // Messages
        let messages = self.current_messages();
        let middle: Element<'_, Message> = if messages.is_empty() {
            empty_state::view(&self.model_state, self.app_mode, s)
        } else {
            let mut msgs = column![].spacing(8).padding([8, 4]);

            for (i, msg) in messages.iter().enumerate() {
                msgs = msgs.push(message_bubble::view(i, msg));
            }

            let is_busy = matches!(self.recording_state, RecordingState::Processing | RecordingState::Translating);
            if is_busy {
                let status = self.status_text.as_deref().unwrap_or(s.transcribing);
                msgs = msgs.push(
                    container(text(status).size(13).style(theme::muted_text))
                        .width(Length::Fill).center_x(Length::Fill).padding([12, 0]),
                );
            }

            scrollable(msgs).anchor_bottom().height(Length::Fill).into()
        };

        let controls = controls::view(&self.recording_state, self.elapsed_secs, &self.text_input_value, &self.audio_levels, self.spinner_tick, s);

        let mut layout = column![header];
        if let Some(bar) = lang_bar { layout = layout.push(bar); }
        if let Some(bar) = corrector_bar { layout = layout.push(bar); }
        if let Some(bar) = prompt_bar { layout = layout.push(bar); }
        if let Some(bar) = tts_bar { layout = layout.push(bar); }
        layout = layout.push(middle);
        layout = layout.push(controls);
        layout.into()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::CheckWorker)
    }
}

fn set_window_icon() -> Task<Message> { Task::perform(async {}, |_| Message::SetIcon) }

fn make_icon() -> iced::window::Icon {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(ICON_BYTES).expect("Invalid icon PNG");
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    iced::window::icon::from_rgba(rgba.into_raw(), w, h).expect("Invalid icon data")
}
