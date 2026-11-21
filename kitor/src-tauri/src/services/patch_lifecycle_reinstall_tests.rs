use super::{can_reinstall_after_store_restore, tests::Fixture, uninstall, verify_installed};
use serde_json::json;
use std::fs;

#[test]
fn reinstalls_after_storefront_restores_originals_and_leaves_patch_files() {
    let fixture = Fixture::new();
    let game_data = fixture.game.join("game.dat");
    let helper = fixture.game.join("helper.dll");
    let save = fixture.game.join("save.dat");
    fs::write(&game_data, b"original").unwrap();
    fixture.plan(
        json!([
            fixture.copy("game.dat", "modify", b"translated"),
            fixture.copy("helper.dll", "create", b"helper"),
        ]),
        "1",
    );
    let first = fixture.install("osx", "1").unwrap();
    assert!(!can_reinstall_after_store_restore(&first, &fixture.game).unwrap());
    fs::write(&game_data, b"original").unwrap();
    fs::write(&save, b"progress").unwrap();

    fs::write(&helper, b"unrelated replacement").unwrap();
    assert!(!can_reinstall_after_store_restore(&first, &fixture.game).unwrap());
    assert!(fixture.install("osx", "1").is_err());
    assert_eq!(fs::read(&helper).unwrap(), b"unrelated replacement");
    assert_eq!(fs::read(&game_data).unwrap(), b"original");

    fs::write(&helper, b"helper").unwrap();
    assert!(can_reinstall_after_store_restore(&first, &fixture.game).unwrap());
    let second = fixture.install("osx", "1").unwrap();
    assert_ne!(first, second);
    assert!(verify_installed(&second, &fixture.game, "test", "1")
        .unwrap()
        .is_empty());
    assert_eq!(fixture.install("osx", "1").unwrap(), second);

    uninstall(&second, &fixture.game).unwrap();
    assert_eq!(fs::read(&game_data).unwrap(), b"original");
    assert!(!helper.exists());
    assert_eq!(fs::read(&save).unwrap(), b"progress");
}
