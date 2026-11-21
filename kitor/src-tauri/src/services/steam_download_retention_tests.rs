use super::retain_download;

#[test]
fn unsupported_edition_keeps_downloaded_files_and_reports_location() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("content");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("game.dat"), b"original game").unwrap();
    let root = directory.path().to_path_buf();
    let mut staging = Some((directory, source.clone()));
    let error = retain_download(&mut staging, "Unsupported edition".into());
    drop(staging);
    assert_eq!(std::fs::read(source.join("game.dat")).unwrap(), b"original game");
    assert!(error.contains(&source.display().to_string()));
    assert!(error.contains("Unsupported edition"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_steam_download_is_left_alone() {
    assert_eq!(retain_download(&mut None, "Unsupported edition".into()), "Unsupported edition");
}
