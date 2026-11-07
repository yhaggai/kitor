//! Bounded, allowlisted diagnostics: never persist raw errors, paths or credentials.
use std::{io::{Read, Write}, path::Path, sync::Mutex};
use serde_json::{json, Value};
static LOCK: Mutex<()> = Mutex::new(());
const LIMIT: u64 = 128 * 1024;

pub fn record(dir: &Path, operation: &str, target: &str, phase: &str) {
    let Ok(_guard) = LOCK.lock() else { return };
    let _ = (|| -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("diagnostics.jsonl");
        if std::fs::metadata(&path).map(|m| m.len() >= LIMIT).unwrap_or(false) {
            let previous = dir.join("diagnostics.previous.jsonl");
            if previous.exists() { std::fs::remove_file(&previous)?; }
            std::fs::rename(&path, previous)?;
        }
        let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", json!({"time": chrono::Utc::now().to_rfc3339(),
            "operation": operation, "target": target.chars().take(128).collect::<String>(), "phase": phase}))
    })(); // Diagnostics must never prevent a launch or installation.
}

pub fn outcome(result: &Result<(), String>) -> &'static str {
    match result {
        Ok(()) => "completed",
        Err(error) => {
            let error = error.to_lowercase();
            if error.contains("permission") || error.contains("access denied") { "failed: permission" }
            else if error.contains("checksum") || error.contains("hash mismatch") { "failed: checksum" }
            else if error.contains("not found") || error.contains("missing") { "failed: missing file or configuration" }
            else if error.contains("download") || error.contains("http") { "failed: download" }
            else { "failed: other (include the displayed error in your description)" }
        }
    }
}

pub fn history(dir: &Path) -> Vec<Value> {
    let Ok(_guard) = LOCK.lock() else { return vec![] };
    let mut events = Vec::new();
    for name in ["diagnostics.previous.jsonl", "diagnostics.jsonl"] {
        if let Ok(file) = std::fs::File::open(dir.join(name)) {
            let mut text = String::new();
            if file.take(LIMIT + 4096).read_to_string(&mut text).is_ok() {
                events.extend(text.lines().filter_map(|line| serde_json::from_str::<Value>(line).ok()));
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_error_text_is_not_recorded() {
        let dir = tempfile::tempdir().unwrap();
        let result = Err("download https://secret/token failed for /Users/alice".into());
        record(dir.path(), "launch", "42", outcome(&result));
        let report = history(dir.path());
        assert_eq!(report[0]["phase"], "failed: download");
        assert!(!serde_json::to_string(&report).unwrap().contains("secret"));
    }
    #[test]
    fn rotates_history() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("diagnostics.jsonl"), " ".repeat(LIMIT as usize)).unwrap();
        record(dir.path(), "launch", "42", "started");
        assert!(dir.path().join("diagnostics.previous.jsonl").exists());
        assert_eq!(history(dir.path()).len(), 1);
    }
}
