use super::*;

#[test]
fn catalog_profiles_keep_native_and_patched_launches_distinct() {
    for id in ["monkey-se", "monkey2-se"] {
        let behavior = for_game(id).unwrap();
        assert_eq!(behavior.launch_profile(false).is_native(), !cfg!(target_os = "macos"));
        assert!(behavior.launch_profile(true).native.is_none());
        assert_eq!(
            behavior.launch_profile(true).subdirectory.as_deref(),
            Some("hebrew")
        );
    }
    let grim = for_game("grim-fandango-remastered").unwrap();
    assert!(!grim.launch_profile(false).prepare_hebrew_preferences);
    assert!(grim.launch_profile(true).prepare_hebrew_preferences);
}

#[test]
fn declarative_detection_does_not_require_a_patch() {
    let mut definition = super::super::registry_service::load_registry()
        .unwrap()
        .games
        .remove(0);
    definition.scummvm_id = "new-unpatched-game".into();
    definition.patches.clear();
    definition.behavior = GameBehavior::default();
    definition.detection_patterns.unique_files = vec!["unique.dat".into()];
    let temp = tempfile::tempdir().unwrap();
    assert!(super::super::game_adapters::detect(&definition, temp.path()).is_none());
    std::fs::write(temp.path().join("unique.dat"), b"fixture").unwrap();
    assert_eq!(
        super::super::game_adapters::detect(&definition, temp.path()),
        Some(temp.path().to_path_buf())
    );
}

#[test]
fn native_command_can_be_added_entirely_through_configuration() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("game-runner"), b"fixture").unwrap();
    let platform = crate::services::patch_lifecycle::platform();
    let profile: LaunchProfile = serde_json::from_value(serde_json::json!({
        "commands": {platform: {"program": "game-runner", "args": ["--windowed"], "env": {"GAME_LANGUAGE": "en"}}}
    })).unwrap();
    assert!(profile.is_native());
    let cmd = crate::services::game_adapters::native_command(
        &profile,
        root.path(),
        "previously-unknown-game",
    )
    .unwrap()
    .unwrap();
    assert_eq!(cmd.get_program(), root.path().join("game-runner"));
    assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec!["--windowed"]);
    assert_eq!(cmd.get_current_dir(), Some(root.path()));
}

#[test]
fn invalid_profiles_fail_before_launch() {
    assert!(serde_json::from_value::<GameBehavior>(
        serde_json::json!({"detection": {"adapter": "misspelled_adapter"}})
    )
    .is_err());
    let profile = LaunchProfile {
        subdirectory: Some("../another-game".into()),
        ..Default::default()
    };
    let behavior = GameBehavior {
        launch: Some(LaunchProfiles {
            original: profile.clone(),
            original_macos: None,
            patched: profile,
        }),
        ..Default::default()
    };
    assert!(behavior.validate().is_err());
}

#[test]
fn monkey_original_macos_uses_classic_scummvm_data() {
    for (id, target) in [("monkey-se", "scumm:monkey"), ("monkey2-se", "scumm:monkey2")] {
        let behavior = for_game(id).unwrap();
        let profiles = behavior.launch.as_ref().unwrap();
        assert!(profiles.original.is_native());
        let mac = profiles.original_macos.as_ref().unwrap();
        assert!(!mac.is_native());
        assert_eq!(mac.args, vec!["--fullscreen", target]);
        assert!(mac.subdirectory.is_none());
        if cfg!(target_os = "macos") {
            assert_eq!(behavior.launch_profile(false).args, mac.args);
        }
    }
}
