use super::*;

#[tokio::test]
async fn manual_patch_evidence_drives_library_and_launch_state() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("game");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("translated.dat"), b"translated").unwrap();
    let behavior = super::super::game_behavior::for_game("monkey2-se").unwrap();
    let record = temp
        .path()
        .join(behavior.manual_patch_manifest.as_ref().unwrap());
    std::fs::create_dir_all(record.parent().unwrap()).unwrap();
    std::fs::write(&record, serde_json::to_vec(&serde_json::json!({
        "game_root": root, "patch": "monkey2-se/he", "status": "installed", "files": [
            {"path": "translated.dat", "sha256": super::super::patch_lifecycle::hash(&root.join("translated.dat")).unwrap()}
        ]
    })).unwrap()).unwrap();
    let db = crate::db::init_db(&temp.path().join("db")).await.unwrap();
    let game = super::super::game_service::add_game_manually(
        &db,
        "monkey2-se".into(),
        "Fixture".into(),
        root.to_string_lossy().into_owned(),
        temp.path().to_path_buf(),
    )
    .await
    .unwrap();
    let state = resolve(&db, &game, temp.path(), &behavior).await.unwrap();
    assert!(state.is_patched());
    assert!(state.recorded.is_empty()); // No fabricated backup/restore record.
    assert!(!behavior.launch_profile(state.is_patched()).is_native());
    let library =
        super::super::game_service::get_games_with_patches(&db, temp.path().to_path_buf())
            .await
            .unwrap();
    assert!(library[0].is_patched);
    assert!(library[0].requires_scummvm);
    assert_eq!(
        library[0].health.as_ref().unwrap().patch_status,
        "installed"
    );
    std::fs::write(root.join("translated.dat"), b"changed").unwrap();
    let state = resolve(&db, &game, temp.path(), &behavior).await.unwrap();
    assert!(!state.is_patched());
    assert_eq!(state.manual_health.unwrap().patch_status, "damaged");
    let other = temp.path().join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(super::super::game_refresh_service::inspect_manual(
        &behavior,
        temp.path(),
        &other,
        "monkey2-se"
    )
    .is_none());
}
