use super::{launch_args, normalize_collection_args};

#[test]
fn original_launch_does_not_force_patch_language() {
    let args = vec!["--language=he".into(), "--fullscreen".into(), "--auto-detect".into()];
    for id in ["pajama", "toonstruck", "pq1", "torin", "sky"] {
        assert_eq!(launch_args(id, args.clone(), false), vec!["--fullscreen", "--auto-detect"]);
        assert_eq!(launch_args(id, args.clone(), true), args);
    }
    assert_eq!(launch_args("unpatched-game", vec![], false), vec!["--auto-detect"]);
    assert_eq!(launch_args("kyra1-cd", vec!["kyra:kyra1".into()], false), vec!["kyra:kyra1"]);
}

#[test]
fn patched_sq1_vga_enables_hebrew_with_stale_settings() {
    for args in [vec![], vec!["--language=en".into()], vec!["--language=he".into()]] {
        assert_eq!(normalize_collection_args("sq1sci", args, true),
            vec!["--language=he", "--platform=dos", "sci:sq1sci"]);
    }
    assert_eq!(normalize_collection_args("sq1sci", vec!["--language=he".into()], false),
        vec!["--platform=dos", "sci:sq1sci"]);
}

#[test]
fn original_and_patched_collection_launches_select_the_correct_language() {
    let args = vec!["--language=he".into(), "--fullscreen".into(), "--auto-detect".into()];
    assert_eq!(normalize_collection_args("sq1", args.clone(), false), vec!["--fullscreen", "--platform=dos", "agi:sq1"]);
    assert_eq!(normalize_collection_args("sq1", args.clone(), true), vec!["--language=he", "--fullscreen", "--platform=dos", "agi:sq1"]);
    assert_eq!(normalize_collection_args("monkey2-se", args.clone(), false), args);
}
