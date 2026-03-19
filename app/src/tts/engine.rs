use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::types::{KokoroConfig, Language, TtsEngine};

#[derive(Serialize)]
struct TtsRequest {
    engine: String,
    text: String,
    language: String,
    output_path: String,
    use_gpu: bool,
    // Kokoro-specific
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blend_voice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blend_ratio: Option<(f32, f32)>,
}

#[derive(Deserialize)]
struct TtsResponse {
    success: bool,
    audio_path: Option<String>,
    error: Option<String>,
}

fn tts_output_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("speech-to-text").join("tts-output")
}

fn find_tts_worker() -> Result<PathBuf, String> {
    // Check relative to executable (for installed/packaged builds)
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));

    let candidates = [
        exe_dir.join("tts-worker").join("tts_worker.py"),
        // cargo run: exe is in target/debug/ or target/release/
        exe_dir.join("../../tts-worker/tts_worker.py"),
        // current working directory
        PathBuf::from("tts-worker/tts_worker.py"),
    ];

    for path in &candidates {
        if let Ok(canonical) = path.canonicalize() {
            if canonical.exists() {
                return Ok(canonical);
            }
        }
    }

    Err("tts_worker.py introuvable. Verifiez que tts-worker/ est present.".to_string())
}

fn find_python() -> Result<String, String> {
    // Check for venv python first — do NOT canonicalize, as resolving symlinks
    // would point to the system python and bypass the venv's site-packages.
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe.parent().unwrap_or(std::path::Path::new("."));

    let venv_candidates = [
        exe_dir.join("../../tts-worker/venv/bin/python3"),
        exe_dir.join("../../tts-worker/venv/bin/python"),
        PathBuf::from("tts-worker/venv/bin/python3"),
        PathBuf::from("tts-worker/venv/bin/python"),
    ];

    for path in &venv_candidates {
        eprintln!("[TTS] Checking python: {:?} exists={}", path, path.exists());
        if path.exists() {
            let path_str = path.to_string_lossy().to_string();
            eprintln!("[TTS] Found venv python: {}", path_str);
            return Ok(path_str);
        }
    }

    eprintln!("[TTS] WARNING: venv not found, falling back to system python3");
    Ok("python3".to_string())
}

pub fn generate_speech(
    text: &str,
    engine: TtsEngine,
    language: Language,
    use_gpu: bool,
    kokoro: &KokoroConfig,
) -> Result<String, String> {
    let worker_script = find_tts_worker()?;
    let python = find_python()?;

    let output_dir = tts_output_dir();
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Creation dossier TTS: {e}"))?;

    let output_path = output_dir.join(format!(
        "tts_{}.wav",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    ));

    let engine_name = match engine {
        TtsEngine::Kokoro => "kokoro",
        TtsEngine::Chatterbox => "chatterbox",
        TtsEngine::Qwen3 => "qwen3",
    };

    let lang_code = match language {
        Language::Fr => "fr",
        Language::En => "en",
    };

    // Build Kokoro-specific fields
    let (voice, speed, blend_voice, blend_ratio) = if engine == TtsEngine::Kokoro {
        let bv = if kokoro.blend_enabled {
            Some(kokoro.blend_voice.code.to_string())
        } else {
            None
        };
        let br = if kokoro.blend_enabled {
            Some(kokoro.blend_ratio.weights())
        } else {
            None
        };
        (
            Some(kokoro.voice.code.to_string()),
            Some(kokoro.speed.value()),
            bv,
            br,
        )
    } else {
        (None, None, None, None)
    };

    let req = TtsRequest {
        engine: engine_name.to_string(),
        text: text.to_string(),
        language: lang_code.to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        use_gpu,
        voice,
        speed,
        blend_voice,
        blend_ratio,
    };

    let req_json = serde_json::to_string(&req).map_err(|e| format!("Serialize: {e}"))?;

    eprintln!("[TTS] Launching {} with engine={}", python, engine_name);

    let mut child = Command::new(&python)
        .arg(worker_script.to_str().unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Lancement tts-worker: {e}"))?;

    {
        let stdin = child.stdin.as_mut().ok_or("Pas de stdin")?;
        writeln!(stdin, "{req_json}").map_err(|e| format!("Ecriture stdin: {e}"))?;
        stdin.flush().map_err(|e| format!("Flush stdin: {e}"))?;
    }
    drop(child.stdin.take());

    let stdout = child.stdout.take().ok_or("Pas de stdout")?;
    let reader = BufReader::new(stdout);

    let mut response_line = String::new();
    for line in reader.lines() {
        match line {
            Ok(l) if !l.trim().is_empty() => {
                response_line = l;
                break;
            }
            Ok(_) => continue,
            Err(e) => return Err(format!("Lecture stdout: {e}")),
        }
    }

    let _ = child.kill();
    let _ = child.wait();

    if response_line.is_empty() {
        return Err("Pas de reponse du tts-worker".to_string());
    }

    let resp: TtsResponse =
        serde_json::from_str(&response_line).map_err(|e| format!("Parse reponse: {e}"))?;

    if resp.success {
        if let Some(path) = resp.audio_path {
            Ok(path)
        } else {
            Err("Pas de chemin audio dans la reponse".to_string())
        }
    } else {
        Err(resp.error.unwrap_or_else(|| "Erreur inconnue".to_string()))
    }
}
