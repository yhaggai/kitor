use super::*;

#[test]
fn patched_toon_runtime_is_optional_and_isolated() {
    let root = tempfile::tempdir().unwrap();
    let standard = root.path().join("standard-scummvm");
    assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), standard);
    let directory = root.path().join("scummvm-toon-hebrew");
    std::fs::create_dir(&directory).unwrap();
    let patched = directory.join(if cfg!(target_os = "windows") { "scummvm.exe" } else { "scummvm" });
    std::fs::write(&patched, b"fixture").unwrap();
    assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), standard);
    std::fs::write(directory.join("toon.dat"), b"fixture").unwrap();
    assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), patched);
    assert_eq!(game_runtime_path(root.path(), "pajama", standard.clone()), standard);
    let mut cmd = Command::new("scummvm");
    add_toon_runtime_data(&mut cmd, root.path(), "toonstruck");
    assert_eq!(cmd.get_args().next().unwrap().to_string_lossy(),
        format!("--extrapath={}", directory.display()));
}

#[cfg(target_os = "macos")]
#[test]
fn drifter_launches_native_bundle_from_both_detected_and_manual_paths() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("TheDrifter.app");
    assert!(drifter_launch_command(root.path()).is_err());
    std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
    std::fs::create_dir_all(bundle.join("Contents/Resources/Data")).unwrap();
    std::fs::write(bundle.join("Contents/MacOS/The Drifter"), b"runner").unwrap();
    std::fs::write(bundle.join("Contents/Resources/Data/globalgamemanagers"), b"data").unwrap();
    for path in [root.path(), bundle.as_path()] {
        let cmd = drifter_launch_command(path).unwrap();
        assert_eq!(cmd.get_program(), "/usr/bin/open");
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec![bundle.canonicalize().unwrap().as_os_str()]);
        assert_eq!(cmd.get_current_dir(), Some(root.path().canonicalize().unwrap().as_path()));
    }
}


#[test]
fn pajama_subtitles_do_not_depend_on_saved_launch_args() {
    let mut cmd = Command::new("scummvm");
    add_required_game_options(&mut cmd, "pajama");
    cmd.args(["--language=he", "--fullscreen", "--auto-detect"]);
    assert!(cmd.get_args().any(|arg| arg == "--subtitles"));

    let mut other_game = Command::new("scummvm");
    add_required_game_options(&mut other_game, "kyra1-cd");
    assert_eq!(other_game.get_args().count(), 0);
}

#[test]
fn extra_path_is_a_single_long_option_including_spaces() {
    let mut cmd = Command::new("scummvm");
    add_game_paths(&mut cmd, Path::new("/launch folder"), Some(Path::new("/Game.app/game data")));
    assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec![
        std::ffi::OsStr::new("-p"), std::ffi::OsStr::new("/launch folder"),
        std::ffi::OsStr::new("--extrapath=/Game.app/game data")]);
}

#[cfg(unix)]
#[tokio::test]
async fn immediate_launch_failure_reaches_the_caller() {
    let log = std::env::temp_dir().join(format!("kitor-failed-launch-{}.log", std::process::id()));
    let mut cmd = Command::new("/bin/sh");
    cmd.args(["-c", "echo 'invalid launch option' >&2; exit 1"]);
    let error = start_game_process(&mut cmd, &log).await.unwrap_err();
    assert!(error.contains("invalid launch option"));
    std::fs::remove_file(log).unwrap();
}

#[test]
fn launch_signature_is_isolated_and_refreshed_after_patching() {
    let root = std::env::temp_dir().join(format!("kitor-kyra-launch-{}-{}",
        std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
    let data = root.join("data");
    let launch = root.join("launch");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("gemcut.pak"), b"original").unwrap();
    std::fs::write(data.join("FATE.PAK"), b"conflicting signature").unwrap();
    prepare_kyrandia_launch_path(&data, &launch).unwrap();
    assert_eq!(std::fs::read(launch.join("GEMCUT.PAK")).unwrap(), b"original");
    assert!(!launch.join("FATE.PAK").exists());
    std::fs::write(data.join("gemcut.pak"), b"patched").unwrap();
    prepare_kyrandia_launch_path(&data, &launch).unwrap();
    assert_eq!(std::fs::read(launch.join("GEMCUT.PAK")).unwrap(), b"patched");
    assert!(data.join("FATE.PAK").exists());
    std::fs::remove_dir_all(root).unwrap();
}
