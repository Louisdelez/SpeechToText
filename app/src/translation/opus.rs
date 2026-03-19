use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::types::Language;

struct OpusModel {
    src: Language,
    tgt: Language,
    dir_name: &'static str,
    files: &'static [(&'static str, &'static str)],
}

const MODELS: &[OpusModel] = &[
    OpusModel {
        src: Language::Fr,
        tgt: Language::En,
        dir_name: "opus-mt-fr-en",
        files: &[
            ("model.bin", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-fr-en/resolve/main/model.bin"),
            ("config.json", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-fr-en/resolve/main/config.json"),
            ("shared_vocabulary.txt", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-fr-en/resolve/main/shared_vocabulary.txt"),
            ("source.spm", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-fr-en/resolve/main/source.spm"),
            ("target.spm", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-fr-en/resolve/main/target.spm"),
        ],
    },
    OpusModel {
        src: Language::En,
        tgt: Language::Fr,
        dir_name: "opus-mt-en-fr",
        files: &[
            ("model.bin", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-en-fr/resolve/main/model.bin"),
            ("config.json", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-en-fr/resolve/main/config.json"),
            ("shared_vocabulary.txt", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-en-fr/resolve/main/shared_vocabulary.txt"),
            ("source.spm", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-en-fr/resolve/main/source.spm"),
            ("target.spm", "https://huggingface.co/michaelfeil/ct2fast-opus-mt-en-fr/resolve/main/target.spm"),
        ],
    },
];

fn models_dir() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("speech-to-text").join("models")
}

fn find_model(src: Language, tgt: Language) -> Option<&'static OpusModel> {
    MODELS.iter().find(|m| m.src == src && m.tgt == tgt)
}

fn model_dir(m: &OpusModel) -> PathBuf {
    models_dir().join(m.dir_name)
}

pub fn opus_model_exists(src: Language, tgt: Language) -> bool {
    match find_model(src, tgt) {
        Some(m) => model_dir(m).join("model.bin").exists(),
        None => false,
    }
}

pub fn download_opus_model(
    src: Language,
    tgt: Language,
    on_progress: impl Fn(u8),
) -> Result<PathBuf, String> {
    let m = find_model(src, tgt)
        .ok_or_else(|| format!("No Opus-MT model for {:?} -> {:?}", src, tgt))?;

    let dir = model_dir(m);
    fs::create_dir_all(&dir).map_err(|e| format!("Create dir: {e}"))?;

    if dir.join("model.bin").exists() {
        on_progress(100);
        return Ok(dir);
    }

    let total_files = m.files.len();
    for (i, (filename, url)) in m.files.iter().enumerate() {
        let path = dir.join(filename);
        if path.exists() {
            continue;
        }

        let mut resp = reqwest::blocking::Client::new()
            .get(*url)
            .send()
            .map_err(|e| format!("Download {filename}: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("Download {filename} failed: {}", resp.status()));
        }

        let tmp = path.with_extension("part");
        let mut file = fs::File::create(&tmp).map_err(|e| format!("Create {filename}: {e}"))?;
        let mut buf = [0u8; 65536];
        loop {
            let n = std::io::Read::read(&mut resp, &mut buf)
                .map_err(|e| format!("Read {filename}: {e}"))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n])
                .map_err(|e| format!("Write {filename}: {e}"))?;
        }
        file.flush().map_err(|e| format!("Flush: {e}"))?;
        drop(file);
        fs::rename(&tmp, &path).map_err(|e| format!("Rename: {e}"))?;

        let pct = (((i + 1) * 100) / total_files).min(99) as u8;
        on_progress(pct);
    }

    on_progress(100);
    Ok(dir)
}

pub fn translate_opus(text: &str, src: Language, tgt: Language) -> Result<String, String> {
    let m = find_model(src, tgt)
        .ok_or_else(|| format!("No Opus-MT model for {:?} -> {:?}", src, tgt))?;

    let dir = model_dir(m);
    if !dir.join("model.bin").exists() {
        return Err("Model not downloaded".to_string());
    }

    use ct2rs::tokenizers::sentencepiece::Tokenizer;
    use ct2rs::{Config, TranslationOptions, Translator};

    let tokenizer = Tokenizer::new(&dir)
        .map_err(|e| format!("Load tokenizer: {e}"))?;

    let translator = Translator::with_tokenizer(&dir, tokenizer, &Config::default())
        .map_err(|e| format!("Load translator: {e}"))?;

    let options = TranslationOptions::<String, String>::default();

    let results = translator
        .translate_batch(
            &[text],
            &options,
            None,
        )
        .map_err(|e| format!("Translate: {e}"))?;

    if results.is_empty() {
        return Err("Empty translation result".to_string());
    }

    let (translated_text, _score) = &results[0];
    Ok(translated_text.trim().to_string())
}
