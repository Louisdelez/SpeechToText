use std::fs;
use std::path::PathBuf;

use crate::types::Conversation;

fn history_path() -> PathBuf {
    let base = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("speech-to-text").join("conversations.json")
}

pub fn load() -> Vec<Conversation> {
    let path = history_path();
    if !path.exists() {
        return Vec::new();
    }
    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save(conversations: &[Conversation]) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(data) = serde_json::to_string_pretty(conversations) {
        let _ = fs::write(&path, data);
    }
}
