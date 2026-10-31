use crate::services::game_behavior::for_game;

#[test]
fn original_and_patched_profiles_select_language_and_engine() {
    for id in ["pajama", "toonstruck", "pq1", "torin", "sky", "sq1", "sq1sci", "sq2", "sq3", "sq5"] {
        let behavior = for_game(id).unwrap();
        assert!(!behavior.launch_profile(false).args.iter().any(|arg| arg.starts_with("--language=")), "{id}");
        assert!(behavior.launch_profile(true).args.contains(&"--language=he".to_string()), "{id}");
    }
    for (id, engine) in [("sq1", "agi"), ("sq1sci", "sci"), ("sq2", "agi"), ("sq3", "sci"), ("sq5", "sci")] {
        for patched in [false, true] {
            let profile = for_game(id).unwrap().launch_profile(patched);
            assert!(profile.args.contains(&format!("{engine}:{id}")));
            assert!(profile.args.contains(&"--platform=dos".into()));
            assert!(!profile.args.contains(&"--auto-detect".into()));
        }
    }
    assert_eq!(for_game("unknown").unwrap().launch_profile(false).args, vec!["--auto-detect"]);
}
