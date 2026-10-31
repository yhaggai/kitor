use super::*;
#[test]
fn completion_is_depot_specific() {
    assert!(completion("Depot download complete : \"/Steam/MacOS\\steamapps\\content\\app_32460\\depot_32461\" (manifest 123)",32460,32461));
    assert!(!completion(
        "Downloading depot 32461 (40 files)",
        32460,
        32461
    ));
    assert!(!completion(
        "Depot download complete : \"/app_32460/depot_324610\"",
        32460,
        32461
    ));
    assert!(!completion(
        "Depot download complete : \"/app_99/depot_32461\"",
        32460,
        32461
    ));
}
#[test]
fn import_preserves_bytes() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("source");
    std::fs::create_dir(&s).unwrap();
    std::fs::write(s.join("game"), b"game data").unwrap();
    let d = t.path().join("dest");
    copy_checked(&s, &d).unwrap();
    assert_eq!(std::fs::read(d.join("game")).unwrap(), b"game data");
}
#[cfg(unix)]
#[test]
fn import_rejects_links() {
    let t = tempfile::tempdir().unwrap();
    let s = t.path().join("link");
    std::os::unix::fs::symlink("/etc/hosts", &s).unwrap();
    assert!(copy_checked(&s, &t.path().join("dest")).is_err());
}
