use super::*;
use std::path::PathBuf;
#[test]
fn detection_requires_title_specific_executable_and_archive_header() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("MISE.exe"), b"fixture").unwrap();
    fs::write(d.path().join("Monkey1.pak"), b"not a pak").unwrap();
    assert!(detect(d.path(), "monkey-se-native").is_none());
    fs::write(d.path().join("Monkey1.pak"), b"KAPL\0\0\x80?").unwrap();
    assert!(detect(d.path(), "monkey-se-native").is_some());
    assert!(detect(d.path(), "monkey2-se-native").is_none());
    assert!(!supported("monkey-se"));
}
#[test]
fn staging_rejects_wrong_identity_and_version() {
    let d = tempfile::tempdir().unwrap();
    assert!(stage(d.path(), "monkey-se-native", "en", VERSION).is_err());
    assert!(stage(d.path(), "monkey-se-native", "he", "unknown").is_err());
    assert!(stage(d.path(), "monkey-se", "he", VERSION).is_err());
}
fn detect(root: &Path, id: &str) -> Option<PathBuf> {
    crate::services::game_detection::detect_registered(id, root, None)
}
