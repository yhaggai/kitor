use super::*;
use serde_json::json;
fn definition(layouts: serde_json::Value) -> GameDefinition {
    serde_json::from_value(json!({"scummvm_id":"new-game","title":"New Game",
        "detection_patterns":{"unique_files":["common.exe"],"file_patterns":[]},"patches":[],
        "behavior":{"detection":{"mode":"layouts","layouts":layouts}}}))
    .unwrap()
}
fn put(root: &Path, name: &str, data: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}
#[test]
fn content_identity_cannot_fall_back_to_a_shared_filename() {
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "common.exe", b"runner");
    put(
        temp.path(),
        "Data/APP.INFO",
        b"Another Studio\nAnother Game",
    );
    let def = definition(
        json!([{"root":".","required_files":["common.exe","data/app.info"],"file_checks":[{"kind":"text","path":"data/app.info","equals":"Studio\nGame","normalization":"trim_crlf"}]}]),
    );
    assert!(detect(&def, temp.path()).is_none());
    put(temp.path(), "Data/APP.INFO", b"Studio\r\nGame\r\n ");
    assert_eq!(detect(&def, temp.path()), temp.path().canonicalize().ok());
}
#[test]
fn directories_prefixes_sizes_and_output_data_root_are_generic() {
    let temp = tempfile::tempdir().unwrap();
    put(
        temp.path(),
        "Example.app/Contents/Resources/archive",
        b"KAPL\0\0\x80?payload",
    );
    let def = definition(
        json!([{"root":"Example.app","required_files":["Contents/Resources/archive"],"required_directories":["Contents/MacOS"],"data":"Contents/Resources","file_checks":[{"kind":"prefix","path":"Contents/Resources/archive","bytes":[75,65,80,76,0,0,128,63],"min_size":9}]}]),
    );
    assert!(detect(&def, temp.path()).is_none());
    fs::create_dir_all(temp.path().join("Example.app/Contents/MacOS")).unwrap();
    assert_eq!(
        detect(&def, temp.path()),
        temp.path()
            .join("Example.app/Contents/Resources")
            .canonicalize()
            .ok()
    );
    put(
        temp.path(),
        "Example.app/Contents/Resources/archive",
        b"KAPL\0\0\x80?",
    );
    assert!(detect(&def, temp.path()).is_none());
}
#[test]
fn digest_rules_support_full_sha256_and_sized_prefix_md5() {
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "data", b"abc-more");
    let hash = format!("{:x}", md5::Md5::digest(b"abc"));
    let def = definition(
        json!([{"root":".","required_files":["data"],"file_checks":[{"kind":"digest","path":"data","algorithm":"md5","bytes":3,"size":8,"values":[hash]}]}]),
    );
    assert!(detect(&def, temp.path()).is_some());
    put(temp.path(), "data", b"abd-more");
    assert!(detect(&def, temp.path()).is_none());
    let full = format!("{:x}", sha2::Sha256::digest(b"abc"));
    put(temp.path(), "data", b"abc");
    let def = definition(
        json!([{"root":".","required_files":["data"],"file_checks":[{"kind":"digest","path":"data","algorithm":"sha256","values":[full]}]}]),
    );
    assert!(detect(&def, temp.path()).is_some());
    put(temp.path(), "data", b"abc-more");
    assert!(detect(&def, temp.path()).is_none());
}
#[test]
fn ancestor_resolution_is_bounded_and_selected_directory_is_checked() {
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "game/marker", b"game");
    fs::create_dir_all(temp.path().join("game/Data")).unwrap();
    let def = definition(
        json!([{"root":".","ancestor":1,"selected_name":"Data","required_files":["marker"]}]),
    );
    assert_eq!(
        detect(&def, &temp.path().join("game/Data")),
        temp.path().join("game").canonicalize().ok()
    );
    fs::create_dir_all(temp.path().join("game/Other")).unwrap();
    assert!(detect(&def, &temp.path().join("game/Other")).is_none());
    let bad = definition(json!([{"root":"../game","required_files":["marker"]}]));
    assert!(bad.behavior.detection.validate().is_err());
    let bad = definition(json!([{"root":".","ancestor":4,"required_files":["marker"]}]));
    assert!(bad.behavior.detection.validate().is_err());
}
#[cfg(unix)]
#[test]
fn symlinks_do_not_establish_identity() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "outside", b"data");
    symlink(temp.path().join("outside"), temp.path().join("marker")).unwrap();
    let def = definition(json!([{"root":".","required_files":["marker"]}]));
    assert!(detect(&def, temp.path()).is_none());
}
