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
