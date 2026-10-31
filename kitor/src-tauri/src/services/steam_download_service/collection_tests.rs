use super::*;
#[test]
fn imports_only_verified_files_and_never_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let source = root.join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("resource.001"), b"original").unwrap();
    std::fs::write(source.join("unrelated-patch"), b"excluded").unwrap();
    let recipe = ImportRecipe {
        id: "sq3".into(),
        title: "SQ3".into(),
        folder: "sq3".into(),
        files: vec![ImportFile {
            name: "resource.001".into(),
            sha256: format!("{:x}", Sha256::digest(b"original")),
        }],
    };
    let destination = root.join("game");
    import_recipe(&source, &destination, &recipe).unwrap();
    assert!(!destination.join("unrelated-patch").exists());
    std::fs::write(destination.join("save"), b"save").unwrap();
    assert!(import_recipe(&source, &destination, &recipe).is_err());
    assert_eq!(std::fs::read(destination.join("save")).unwrap(), b"save");
    std::fs::write(source.join("resource.001"), b"modified").unwrap();
    assert!(import_recipe(&source, &root.join("bad"), &recipe).is_err());
    assert!(!root.join("bad").exists());
}
#[cfg(unix)]
#[test]
fn rejects_symlinked_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir(root.join("real")).unwrap();
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
    assert!(reject_links(&root.join("link/nested")).is_err());
}
#[tokio::test]
async fn pending_request_resumes_without_spawning_even_after_log_rotation() {
    for rotate in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir(&logs).unwrap();
        let log = logs.join("console_log.txt");
        std::fs::write(&log, "old log\n").unwrap();
        let meta = std::fs::metadata(&log).unwrap();
        let marker = dir.path().join("steam-download-10110-10111.json");
        std::fs::write(
            &marker,
            serde_json::to_vec(&(meta.len(), log_identity(&meta))).unwrap(),
        )
        .unwrap();
        if rotate {
            std::fs::rename(&log, logs.join("previous.txt")).unwrap();
        }
        let prefix = if rotate { "" } else { "old log\n" };
        std::fs::write(&log, format!("{prefix}Depot download complete : \"/Steam/app_10110/depot_10111\" (manifest 123)\n")).unwrap();
        // This cannot be spawned: success proves retry resumed the pending request.
        download_depot(
            dir.path(),
            Path::new("/nonexistent-steam"),
            dir.path(),
            10110,
            10111,
        )
        .await
        .unwrap();
        assert!(!marker.exists());
    }
}
#[test]
fn collection_scan_checks_existing_editions_in_preferred_order() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Space Quest Collection");
    let modern = root.join("2016_SpaceQuestCollection/sq4");
    let old = root.join("sq4");
    let vga = root.join("sq1vga");
    for path in [&modern, &old, &vga, &root.join("unrelated")] {
        std::fs::create_dir_all(path).unwrap();
    }
    assert_eq!(collection_game_folders(&root), vec![vga, modern, old]);
    assert!(collection_game_folders(tmp.path()).is_empty());
}

#[test]
fn bundled_recipes_are_distinct_and_contained() {
    let recipes = recipes();
    assert_eq!(recipes.len(), 7);
    let mut ids = std::collections::HashSet::new();
    for recipe in recipes {
        assert!(ids.insert(recipe.id));
        assert!(!recipe.files.is_empty());
        for file in recipe.files {
            assert_eq!(Path::new(&file.name).components().count(), 1);
            assert!(matches!(
                Path::new(&file.name).components().next(),
                Some(std::path::Component::Normal(_))
            ));
            assert_eq!(file.sha256.len(), 64);
        }
    }
    assert_eq!(ids, ["sq1", "sq1sci", "sq2", "sq3", "sq4", "sq5", "sq6"]
        .into_iter().map(str::to_owned).collect());
}
#[cfg(target_os = "macos")]
#[tokio::test]
#[ignore = "Requires the owned Space Quest Steam depot downloaded locally"]
async fn live_collection_imports_and_reuses_each_game() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let state = crate::AppState {
        db: crate::db::init_db(&root).await.unwrap(),
        app_data_dir: root.clone(),
    };
    let first = download(10110, vec!["sq1".into()], &state).await.unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].scummvm_id, "sq1");
    let original_path = first[0].install_path.clone();
    // Existing installations may be patched or have saves; retry must preserve them.
    std::fs::write(Path::new(&original_path).join("VOL.0"), b"patched data").unwrap();
    std::fs::write(Path::new(&original_path).join("save.test"), b"save").unwrap();
    let all = download(
        10110,
        recipes().iter().map(|r| r.id.clone()).collect(),
        &state,
    )
    .await
    .unwrap();
    assert_eq!(all.len(), 5);
    assert_eq!(
        super::super::game_service::get_all_games(&state.db)
            .await
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        std::fs::read(Path::new(&original_path).join("VOL.0")).unwrap(),
        b"patched data"
    );
    assert_eq!(
        std::fs::read(Path::new(&original_path).join("save.test")).unwrap(),
        b"save"
    );
    for game in all.iter().filter(|g| g.scummvm_id != "sq1") {
        let recipe = recipes()
            .into_iter()
            .find(|r| r.id == game.scummvm_id)
            .unwrap();
        verify_recipe(Path::new(&game.install_path), &recipe).unwrap();
        assert!(Path::new(&game.install_path).starts_with(root.join("games")));
    }
}

#[cfg(target_os = "macos")]
#[tokio::test]
#[ignore = "Requires the owned Police Quest Steam depot downloaded locally"]
async fn live_police_quest_import_and_detection() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let state = crate::AppState {
        db: crate::db::init_db(&root).await.unwrap(),
        app_data_dir: root.clone(),
    };
    let games = download(494740, vec!["pq1".into()], &state).await.unwrap();
    assert_eq!(games.len(), 1);
    let game = &games[0];
    assert_eq!(game.scummvm_id, "pq1");
    let recipe = products().into_iter().find(|p| p.app_id == 494740).unwrap().games.remove(0);
    verify_recipe(Path::new(&game.install_path), &recipe).unwrap();
    assert!(!Path::new(&game.install_path).join("RESOURCE.MAP").exists());
    std::fs::write(Path::new(&game.install_path).join("save.test"), b"save").unwrap();
    let again = download(494740, vec!["pq1".into()], &state).await.unwrap();
    assert_eq!(again[0].install_path, game.install_path);
    assert_eq!(std::fs::read(Path::new(&game.install_path).join("save.test")).unwrap(), b"save");
}

#[test]
fn police_quest_detects_ega_layouts_without_matching_vga() {
    let definition = super::super::registry_service::load_registry().unwrap().games
        .into_iter().find(|g| g.scummvm_id == "pq1").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("PQ1/EGA")).unwrap();
    std::fs::write(root.join("PQ1/RESOURCE.MAP"), b"VGA").unwrap();
    assert!(super::super::game_adapters::detect(&definition, &root).is_none());
    for file in ["LOGDIR", "PICDIR", "SNDDIR", "VIEWDIR", "WORDS.TOK", "VOL.0", "VOL.1", "VOL.2"] {
        std::fs::write(root.join("PQ1/EGA").join(file), b"EGA").unwrap();
    }
    assert_eq!(super::super::game_adapters::detect(&definition, &root), Some(root.join("PQ1/EGA")));
    assert_eq!(super::super::game_adapters::detect(&definition, &root.join("PQ1/EGA")), Some(root.join("PQ1/EGA")));
}

#[test]
fn intact_collection_copy_keeps_games_outside_curated_recipes() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let game = source.join("2016_SpaceQuestCollection/sq6");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(game.join("resource.map"), b"game data").unwrap();
    let destination = dir.path().join("collection");
    copy_checked(&source, &destination).unwrap();
    assert_eq!(std::fs::read(destination.join("2016_SpaceQuestCollection/sq6/resource.map")).unwrap(), b"game data");
    assert!(copy_checked(&source, &destination).is_err());
}
