use super::*;
#[test]
fn detects_missing_and_incomplete_game() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("game");
    assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "missing");
    std::fs::create_dir_all(root.join("audio")).unwrap();
    for name in ["Monkey2.exe", "monkey2.pak", "audio/SFXOriginal.xwb"] {
        std::fs::write(root.join(name), b"data").unwrap();
    }
    assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "present");
    std::fs::remove_file(root.join("monkey2.pak")).unwrap();
    assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "missing");
}
#[test]
fn manual_patch_detects_changed_files() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("game");
    std::fs::create_dir_all(root.join("hebrew")).unwrap();
    let file = root.join("hebrew/MONKEY2.000");
    std::fs::write(&file, b"patched").unwrap();
    let record = t.path().join("manifest.json");
    let manifest = serde_json::json!({"game_root":root,"patch":"monkey2-se/he","status":"installed","files":[{"path":"hebrew/MONKEY2.000","sha256":patch_lifecycle::hash(&file).unwrap()}]});
    std::fs::write(&record, serde_json::to_vec(&manifest).unwrap()).unwrap();
    let mut health = GameHealth {
        game_status: "present".into(),
        patch_status: "not_installed".into(),
        details: vec![],
        checked_at: String::new(),
    };
    verify_manual(&record, &root, "monkey2-se", &mut health).unwrap();
    assert_eq!(health.patch_status, "installed");
    std::fs::remove_file(&file).unwrap();
    verify_manual(&record, &root, "monkey2-se", &mut health).unwrap();
    assert_eq!(health.patch_status, "damaged");
    assert!(!file.exists());
}

#[test]
#[ignore = "Read-only check of this machine's downloaded and manually patched game"]
fn live_monkey2_refresh() {
    let home = std::env::home_dir().unwrap();
    let root=home.join("Library/Application Support/Steam/Steam.AppBundle/Steam/Contents/MacOS/steamapps/content/app_32460/depot_32461");
    assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "present");
    let record=home.join("Library/Application Support/haggai/manual-patches/monkey2-steam-he-1.1-kitor2/manifest.json");
    let mut health = GameHealth {
        game_status: "present".into(),
        patch_status: "not_installed".into(),
        details: vec![],
        checked_at: String::new(),
    };
    verify_manual(&record, &root, "monkey2-se", &mut health).unwrap();
    assert_eq!(health.patch_status, "installed");
}

#[test]
fn verifies_patch_bytes_without_changing_them() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("game");
    let backup = t.path().join("backup");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&backup).unwrap();
    let file = root.join("patched");
    std::fs::write(&file, b"patched").unwrap();
    let manifest = serde_json::json!({"schema_version":1,"target":root.canonicalize().unwrap(),"game":"test","language":"he","version":"1","status":"active","created_dirs":[],"files":[{"path":"patched","original_hash":null,"installed_hash":patch_lifecycle::hash(&file).unwrap(),"started":true,"restored":false},{"path":"removed","original_hash":"old","installed_hash":null,"started":true,"restored":false}]});
    std::fs::write(
        backup.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    assert!(
        patch_lifecycle::verify_installed(&backup, &root, "test", "1")
            .unwrap()
            .is_empty()
    );
    std::fs::write(&file, b"changed").unwrap();
    assert_eq!(
        patch_lifecycle::verify_installed(&backup, &root, "test", "1")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(std::fs::read(&file).unwrap(), b"changed");
    std::fs::remove_file(&file).unwrap();
    assert!(
        patch_lifecycle::verify_installed(&backup, &root, "test", "1").unwrap()[0]
            .contains("missing")
    );
    assert!(patch_lifecycle::verify_installed(&backup, &root, "other", "1").is_err());
    assert!(patch_lifecycle::verify_installed(t.path(), &root, "test", "1").is_err());
}
