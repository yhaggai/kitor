use super::*;

#[test]
fn old_registry_recovers_reviewed_behavior_without_trusting_saved_args() {
    let bundled: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut old = bundled.clone();
    for game in &mut old.games {
        game.behavior = Default::default();
        game.launch_config = Some(LaunchConfig {
            launcher: None,
            subdirectory: Some("obsolete".into()),
            args: vec!["obsolete".into()],
        });
    }
    prepare_behaviors(&mut old, &bundled).unwrap();
    for (restored, expected) in old.games.iter().zip(&bundled.games) {
        assert_eq!(
            serde_json::to_value(&restored.behavior).unwrap(),
            serde_json::to_value(&expected.behavior).unwrap()
        );
    }
}

#[test]
fn saved_bundled_entry_migrates_to_reviewed_download() {
    let built: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut external = built.clone();
    let game = external
        .games
        .iter_mut()
        .find(|g| g.scummvm_id == "jenny-leclue")
        .unwrap();
    let original = game.patches[0].clone();
    game.patches[0].source_type = "bundled".into();
    game.patches[0].download_url = "bundled://jenny-leclue/he".into();
    game.patches[0].version = "old".into();
    game.patches[0].release_revision += 1;
    migrate_bundled_packages(&mut external, &built);
    let resolved = &external
        .games
        .iter()
        .find(|g| g.scummvm_id == "jenny-leclue")
        .unwrap()
        .patches[0];
    assert_eq!(resolved.source_type, "github");
    assert_eq!(resolved.download_url, original.download_url);
    assert_eq!(resolved.version, original.version);
    assert_eq!(resolved.release_revision, original.release_revision);
}

#[test]
fn embedded_moomin_patch_remains_available_in_the_loaded_registry() {
    let registry = load_registry().unwrap();
    let game = registry
        .games
        .iter()
        .find(|game| game.scummvm_id == "moomin-midsummer-madness")
        .expect("Moomin game is present");
    assert_eq!(game.behavior.bundled_resource.as_deref(), Some("moomin-midsummer-madness"));
    assert!(game.patches.iter().any(|patch| {
        patch.language == "he" && patch.source_type == "bundled"
    }));
}

#[cfg(not(debug_assertions))]
#[test]
fn production_registry_ignores_external_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let external = dir.path().join("registry.json");
    std::fs::write(&external, r#"{"games":[],"attacker":"override"}"#).unwrap();
    assert_eq!(registry_content(&external).unwrap(),
        include_str!("../../../../patches/registry.json"));
}
