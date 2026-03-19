use std::fs;
use std::io::Write;
use std::path::PathBuf;

const LLM_MODEL_FILENAME: &str = "qwen2.5-1.5b-instruct-q4_k_m.gguf";
const LLM_MODEL_URL: &str = "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf";

pub fn models_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("speech-to-text").join("models")
}

pub fn llm_model_path() -> PathBuf {
    models_dir().join(LLM_MODEL_FILENAME)
}

pub fn llm_model_exists() -> bool {
    llm_model_path().exists()
}

pub fn download_llm_model(on_progress: impl Fn(u8)) -> Result<PathBuf, String> {
    let dir = models_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create models dir: {e}"))?;

    let path = llm_model_path();
    if path.exists() {
        on_progress(100);
        return Ok(path);
    }

    let mut resp = reqwest::blocking::Client::new()
        .get(LLM_MODEL_URL)
        .send()
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download failed with status: {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let tmp_path = path.with_extension("gguf.part");
    let mut file =
        fs::File::create(&tmp_path).map_err(|e| format!("Create temp file failed: {e}"))?;

    let mut downloaded: u64 = 0;
    let mut last_pct: u8 = 0;
    let mut buf = [0u8; 65536];

    loop {
        let n = std::io::Read::read(&mut resp, &mut buf)
            .map_err(|e| format!("Download read failed: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("Write model failed: {e}"))?;
        downloaded += n as u64;

        if total > 0 {
            let pct = ((downloaded * 100) / total).min(99) as u8;
            if pct != last_pct {
                on_progress(pct);
                last_pct = pct;
            }
        }
    }

    file.flush().map_err(|e| format!("Flush failed: {e}"))?;
    drop(file);

    fs::rename(&tmp_path, &path).map_err(|e| format!("Rename model failed: {e}"))?;
    on_progress(100);

    Ok(path)
}
