use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use crate::types::Language;

fn models_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("speech-to-text").join("models")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelId {
    Whisper,
    OpusFrEn,
    OpusEnFr,
    Qwen25,
    KokoroOnnx,
}

impl ModelId {
    pub const ALL: &'static [ModelId] = &[
        ModelId::Whisper,
        ModelId::OpusFrEn,
        ModelId::OpusEnFr,
        ModelId::Qwen25,
        ModelId::KokoroOnnx,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ModelId::Whisper => "Whisper Medium",
            ModelId::OpusFrEn => "Opus-MT FR → EN",
            ModelId::OpusEnFr => "Opus-MT EN → FR",
            ModelId::Qwen25 => "Qwen 2.5 1.5B",
            ModelId::KokoroOnnx => "Kokoro TTS (ONNX)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ModelId::Whisper => "Transcription vocale (Speech to Text)",
            ModelId::OpusFrEn => "Traduction rapide francais vers anglais",
            ModelId::OpusEnFr => "Traduction rapide anglais vers francais",
            ModelId::Qwen25 => "Traduction intelligente, resume, correcteur, prompt",
            ModelId::KokoroOnnx => "Synthese vocale legere (Text to Speech)",
        }
    }

    pub fn expected_size(&self) -> &'static str {
        match self {
            ModelId::Whisper => "~1.5 Go",
            ModelId::OpusFrEn => "~140 Mo",
            ModelId::OpusEnFr => "~140 Mo",
            ModelId::Qwen25 => "~1.0 Go",
            ModelId::KokoroOnnx => "~350 Mo",
        }
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        let dir = models_dir();
        match self {
            ModelId::Whisper => vec![dir.join("ggml-medium.bin")],
            ModelId::OpusFrEn => vec![dir.join("opus-mt-fr-en")],
            ModelId::OpusEnFr => vec![dir.join("opus-mt-en-fr")],
            ModelId::Qwen25 => vec![dir.join("qwen2.5-1.5b-instruct-q4_k_m.gguf")],
            ModelId::KokoroOnnx => vec![
                dir.join("kokoro-onnx").join("kokoro-v1.0.onnx"),
                dir.join("kokoro-onnx").join("voices-v1.0.bin"),
            ],
        }
    }

    pub fn exists(&self) -> bool {
        self.paths().iter().all(|p| p.exists())
    }

    pub fn size_bytes(&self) -> u64 {
        self.paths().iter().map(|p| dir_or_file_size(p)).sum()
    }
}

fn dir_or_file_size(path: &std::path::Path) -> u64 {
    if path.is_dir() {
        let mut total = 0u64;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    } else if path.is_file() {
        fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "—".to_string();
    }
    if bytes >= 1_000_000_000 {
        format!("{:.1} Go", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.0} Mo", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.0} Ko", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} o")
    }
}

pub fn delete_model(id: ModelId) -> Result<(), String> {
    for path in id.paths() {
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| format!("Suppression {}: {e}", path.display()))?;
        } else if path.is_file() {
            fs::remove_file(&path).map_err(|e| format!("Suppression {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ModelEvent {
    Progress { id: ModelId, pct: u8 },
    Complete(ModelId),
    Error { id: ModelId, error: String },
}

pub fn download_model_async(id: ModelId, tx: mpsc::Sender<ModelEvent>) {
    thread::spawn(move || {
        let result = download_model_blocking(id, |pct| {
            let _ = tx.send(ModelEvent::Progress { id, pct });
        });
        match result {
            Ok(()) => { let _ = tx.send(ModelEvent::Complete(id)); }
            Err(e) => { let _ = tx.send(ModelEvent::Error { id, error: e }); }
        }
    });
}

fn download_model_blocking(id: ModelId, on_progress: impl Fn(u8)) -> Result<(), String> {
    match id {
        ModelId::Whisper => {
            crate::transcription::model::download_model(&on_progress)?;
            Ok(())
        }
        ModelId::OpusFrEn => {
            crate::translation::opus::download_opus_model(Language::Fr, Language::En, &on_progress)?;
            Ok(())
        }
        ModelId::OpusEnFr => {
            crate::translation::opus::download_opus_model(Language::En, Language::Fr, &on_progress)?;
            Ok(())
        }
        ModelId::Qwen25 => {
            crate::translation::model::download_llm_model(&on_progress)?;
            Ok(())
        }
        ModelId::KokoroOnnx => {
            download_kokoro_onnx(&on_progress)
        }
    }
}

fn download_kokoro_onnx(on_progress: &impl Fn(u8)) -> Result<(), String> {
    let dir = models_dir().join("kokoro-onnx");
    fs::create_dir_all(&dir).map_err(|e| format!("Mkdir: {e}"))?;

    let base_url = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0";

    let files = [
        ("kokoro-v1.0.onnx", 50u8),   // ~first half of progress
        ("voices-v1.0.bin", 100u8),
    ];

    for (filename, target_pct) in &files {
        let dest = dir.join(filename);
        if dest.exists() {
            on_progress(*target_pct);
            continue;
        }

        let url = format!("{base_url}/{filename}");
        download_file_with_progress(&url, &dest, on_progress, *target_pct)?;
    }

    Ok(())
}

fn download_file_with_progress(
    url: &str,
    dest: &std::path::Path,
    on_progress: &impl Fn(u8),
    target_pct: u8,
) -> Result<(), String> {
    let mut resp = reqwest::blocking::Client::new()
        .get(url)
        .send()
        .map_err(|e| format!("Download: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let tmp = dest.with_extension("part");
    let mut file = fs::File::create(&tmp).map_err(|e| format!("Create: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 65536];

    loop {
        let n = std::io::Read::read(&mut resp, &mut buf)
            .map_err(|e| format!("Read: {e}"))?;
        if n == 0 { break; }
        file.write_all(&buf[..n]).map_err(|e| format!("Write: {e}"))?;
        downloaded += n as u64;
        if total > 0 {
            let pct = ((downloaded * target_pct as u64) / total).min(target_pct as u64) as u8;
            on_progress(pct);
        }
    }

    file.flush().map_err(|e| format!("Flush: {e}"))?;
    drop(file);
    fs::rename(&tmp, dest).map_err(|e| format!("Rename: {e}"))?;
    on_progress(target_pct);
    Ok(())
}
