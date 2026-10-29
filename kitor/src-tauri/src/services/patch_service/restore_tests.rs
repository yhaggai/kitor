use super::*;

#[tokio::test]
async fn update_reconciles_stale_database_and_restore_preserves_unrelated_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let data = root.join("game");
    let package = root.join("package");
    std::fs::create_dir(&data).unwrap();
    std::fs::create_dir(&package).unwrap();
    std::fs::write(data.join("GEMCUT.PAK"), b"original").unwrap();
    std::fs::write(data.join("save.dat"), b"saved game").unwrap();
    std::fs::write(package.join("translated"), b"translated").unwrap();
    let checksum = patch_lifecycle::hash(&package.join("translated")).unwrap();
    std::fs::write(package.join("patch.json"), serde_json::to_vec(&serde_json::json!({
        "schema_version": 1, "scummvm_id": "kyra1-cd", "language": "he", "version": "test",
        "filename": "test.zip", "platforms": {"osx": {}},
        "files": [{"path":"GEMCUT.PAK","operation":"modify", "source": {
            "kind":"copy", "path":"translated", "sha256":checksum
        }}]
    })).unwrap()).unwrap();
    let backup = patch_lifecycle::install(&package, "patch.json", &data,
        &root.join("transactions"), "kyra1-cd", "he", "test", "osx").unwrap();
    let db = crate::db::init_db(&root.join("db")).await.unwrap();
    let game = crate::services::game_service::add_game_manually(&db,
        "kyra1-cd".into(), "Kyrandia".into(), data.to_string_lossy().into_owned(), root.to_path_buf())
        .await.unwrap();
    let installed = installed_patch::ActiveModel {
        game_id: Set(game.id), scummvm_id: Set("kyra1-cd".into()),
        version: Set("test".into()), installed_date: Set(chrono::Utc::now().naive_utc()),
        status: Set("active".into()), backup_path: Set(Some(backup.to_string_lossy().into_owned())),
        ..Default::default()
    }.insert(&db).await.unwrap();
    // Simulate a successful file commit followed by an app exit before SQL
    // was updated. The next read must use disk identity, not the registry.
    let mut manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(package.join("patch.json")).unwrap()).unwrap();
    manifest["version"] = serde_json::json!("test2");
    manifest["release_revision"] = serde_json::json!(1);
    std::fs::write(package.join("translated"), b"new translation").unwrap();
    manifest["files"][0]["source"]["sha256"] = serde_json::json!(patch_lifecycle::hash(&package.join("translated")).unwrap());
    std::fs::write(package.join("patch.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
    patch_lifecycle::update(&backup, &package, "patch.json", &data, "kyra1-cd", "he", "test2", "osx").unwrap();
    assert_eq!(installed_patch::Entity::find_by_id(installed.id).one(&db).await.unwrap().unwrap().version, "test");
    let reconciled = get_installed_patches(&db, game.id).await.unwrap();
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].id, installed.id);
    assert_eq!(reconciled[0].version, "test2");
    // Removing tracking leaves recovery data on disk. Another copy of the
    // same title must not inherit that patch, even if SQL reuses the ID.
    crate::services::game_service::remove_game(&db, game.id).await.unwrap();
    let other = root.join("other-copy");
    std::fs::create_dir(&other).unwrap();
    let other_game = crate::services::game_service::add_game_manually(&db,
        "kyra1-cd".into(), "Other copy".into(), other.to_string_lossy().into_owned(), root.to_path_buf())
        .await.unwrap();
    assert!(get_installed_patches(&db, other_game.id).await.unwrap().is_empty());
    crate::services::game_service::remove_game(&db, other_game.id).await.unwrap();
    std::fs::write(data.join("KYRAGEM.PAK"), b"signature").unwrap();
    let scanned = crate::services::game_service::scan_directory(&db, data.to_str().unwrap(), root.to_path_buf())
        .await.unwrap();
    let game = scanned.into_iter().find(|g| g.scummvm_id == "kyra1-cd").unwrap();
    let library = crate::services::game_service::get_games_with_patches(&db, root.to_path_buf()).await.unwrap();
    let patched = library.iter().find(|g| g.id == game.id).unwrap();
    assert_eq!(patched.installed_patches.len(), 1);
    assert_eq!(patched.installed_patches[0].id, installed.id);
    assert_eq!(patched.installed_patches[0].version, "test2");
    assert_eq!(get_installed_patches(&db, game.id).await.unwrap().len(), 1);

    std::fs::write(data.join("later.txt"), b"user file").unwrap();
    std::fs::write(data.join("GEMCUT.PAK"), b"another mod").unwrap();
    assert!(restore_backup(&db, installed.id).await.is_err());
    assert_eq!(get_installed_patches(&db, game.id).await.unwrap().len(), 1);
    std::fs::write(data.join("GEMCUT.PAK"), b"new translation").unwrap();
    restore_backup(&db, installed.id).await.unwrap();
    assert_eq!(std::fs::read(data.join("GEMCUT.PAK")).unwrap(), b"original");
    assert_eq!(std::fs::read(data.join("save.dat")).unwrap(), b"saved game");
    assert_eq!(std::fs::read(data.join("later.txt")).unwrap(), b"user file");
    assert!(get_installed_patches(&db, game.id).await.unwrap().is_empty());
}
