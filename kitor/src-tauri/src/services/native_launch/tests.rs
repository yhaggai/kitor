use super::super::game_behavior;
use super::*;
use serde_json::json;

fn config(value: serde_json::Value) -> NativeCommand {
    serde_json::from_value(value).unwrap()
}
fn put(root: &Path, path: &str, bytes: &[u8]) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn catalog_command(id: &str, patched: bool, platform: Platform) -> NativeCommand {
    game_behavior::for_game(id)
        .unwrap()
        .launch_profile(patched)
        .commands[&platform]
        .clone()
}

#[test]
fn executable_capabilities_work_without_a_game_identity() {
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "Runner.EXE", b"fixture");
    put(temp.path(), "data.bin", b"data");
    let config = config(
        json!({"program": "runner.exe", "args": ["--windowed", "a b"],
        "env": {"GAME_LANGUAGE": "he"}, "required_files": ["data.bin"]}),
    );
    let command = command_for_platform(&config, temp.path(), Platform::Windows).unwrap();
    assert_eq!(command.get_program(), temp.path().join("Runner.EXE"));
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        vec!["--windowed", "a b"]
    );
    assert_eq!(command.get_current_dir(), Some(temp.path()));
    assert_eq!(
        command.get_envs().collect::<Vec<_>>(),
        vec![(
            std::ffi::OsStr::new("GAME_LANGUAGE"),
            Some(std::ffi::OsStr::new("he"))
        )]
    );
}

#[test]
fn missing_native_platform_never_falls_back_to_scummvm() {
    let platform = match Platform::current() {
        Platform::Windows => "osx",
        _ => "windows",
    };
    let profile =
        serde_json::from_value(json!({"commands": {platform: {"program": "game"}}})).unwrap();
    let temp = tempfile::tempdir().unwrap();
    assert!(native_command(&profile, temp.path()).is_err());
    assert!(native_command(&LaunchProfile::default(), temp.path())
        .unwrap()
        .is_none());
}

#[test]
fn monkey_launches_use_configured_signatures_dependencies_and_steam_environment() {
    for id in ["monkey-se-native", "monkey2-se-native"] {
        let temp = tempfile::tempdir().unwrap();
        let (exe, archive, app) = if id == "monkey-se-native" {
            ("MISE.exe", "Monkey1.pak", "32360")
        } else {
            ("Monkey2.exe", "monkey2.pak", "32460")
        };
        put(temp.path(), exe, b"runner");
        put(temp.path(), archive, b"KAPL\0\0\x80?payload");
        put(temp.path(), "steam_api.dll", b"steam");
        for patched in [false, true] {
            let config = catalog_command(id, patched, Platform::Windows);
            let cmd = command_for_platform(&config, temp.path(), Platform::Windows).unwrap();
            assert_eq!(cmd.get_program(), temp.path().join(exe));
            assert!(cmd.get_envs().any(
                |(key, value)| key == "SteamAppId" && value == Some(std::ffi::OsStr::new(app))
            ));
        }
        let config = catalog_command(id, false, Platform::Windows);
        if id == "monkey2-se-native" {
            fs::remove_file(temp.path().join("steam_api.dll")).unwrap();
            assert!(command_for_platform(&config, temp.path(), Platform::Windows).is_err());
            put(temp.path(), "steam_api.dll", b"steam");
        }
        put(temp.path(), archive, b"WRONG\0\x80?payload");
        assert!(command_for_platform(&config, temp.path(), Platform::Windows).is_err());
    }
}

#[test]
fn grim_launch_is_declarative_with_custom_preparation_only_for_hebrew() {
    let original = catalog_command("grim-fandango-remastered", false, Platform::Windows);
    let patched = catalog_command("grim-fandango-remastered", true, Platform::Windows);
    assert_eq!(original.preparation, None);
    assert_eq!(
        patched.preparation,
        Some(NativePreparation::GrimHebrewPreferences)
    );
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "GrimFandango.exe", b"runner");
    assert!(command_for_platform(&original, temp.path(), Platform::Windows).is_err());
    put(temp.path(), "grim.en.tab", b"text");
    assert_eq!(
        command_for_platform(&original, temp.path(), Platform::Windows)
            .unwrap()
            .get_program(),
        temp.path().join("GrimFandango.exe")
    );
}

#[test]
fn tentacle_launch_preserves_archive_validation_and_platform_environment() {
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "Dott.exe", b"runner");
    put(temp.path(), "tenta.cle", b"KAPL\0\0\x80?archive");
    let config = catalog_command("day-of-the-tentacle-remastered", false, Platform::Windows);
    let cmd = command_for_platform(&config, temp.path(), Platform::Windows).unwrap();
    assert!(cmd
        .get_envs()
        .any(|(key, value)| key == "SteamAppId" && value == Some(std::ffi::OsStr::new("388210"))));
    put(temp.path(), "tenta.cle", b"not a supported archive");
    assert!(command_for_platform(&config, temp.path(), Platform::Windows).is_err());
}

#[test]
fn invalid_capabilities_and_escaping_paths_fail_validation() {
    for value in [
        json!({"program":"../runner"}),
        json!({"program":"/bin/sh"}),
        json!({"program":"runner", "executable_files":["../other"]}),
        json!({"program":"runner", "signatures":[{"path":"data","prefix":[]}]}),
        json!({"program":"runner", "method":"app_bundle"}),
        json!({"program":"runner", "root":"app_bundle"}),
        json!({"program":"runner", "root":"app_bundle", "bundle_name":"nested/Game.app"}),
        json!({"program":"runner", "working_directory":"bundle_parent"}),
        json!({"program":"runner", "bundle_name":"Game.app"}),
    ] {
        assert!(
            config(value.clone()).validate().is_err(),
            "accepted {value}"
        );
    }
    for value in [
        json!({"program":"runner","method":"shell"}),
        json!({"program":"runner","root":"undertale"}),
        json!({"program":"runner","preparation":"unknown"}),
    ] {
        assert!(serde_json::from_value::<NativeCommand>(value).is_err());
    }
}

#[cfg(unix)]
#[test]
fn validation_completes_before_permission_changes_and_rejects_symlinks() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let temp = tempfile::tempdir().unwrap();
    put(temp.path(), "runner", b"runner");
    let runner = temp.path().join("runner");
    fs::set_permissions(&runner, fs::Permissions::from_mode(0o640)).unwrap();
    let config = config(
        json!({"program":"runner","executable_files":["runner"],"required_files":["missing"]}),
    );
    assert!(command_for_platform(&config, temp.path(), Platform::MacOS).is_err());
    assert_eq!(
        fs::metadata(&runner).unwrap().permissions().mode() & 0o777,
        0o640
    );
    symlink(&runner, temp.path().join("missing")).unwrap();
    assert!(command_for_platform(&config, temp.path(), Platform::MacOS).is_err());
    assert_eq!(
        fs::metadata(&runner).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[cfg(target_os = "macos")]
#[test]
fn undertale_resolves_bundle_and_resources_and_repairs_only_owner_execute() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let bundle = temp.path().join("UNDERTALE.app");
    put(&bundle, "Contents/MacOS/Mac_Runner", b"runner");
    put(&bundle, "Contents/Resources/game.ios", b"data");
    let runner = bundle.join("Contents/MacOS/Mac_Runner");
    fs::set_permissions(&runner, fs::Permissions::from_mode(0o640)).unwrap();
    for patched in [false, true] {
        let config = catalog_command("undertale", patched, Platform::MacOS);
        for path in [
            temp.path().to_path_buf(),
            bundle.clone(),
            bundle.join("Contents/Resources"),
        ] {
            let command = command_for_platform(&config, &path, Platform::MacOS).unwrap();
            assert_eq!(command.get_program(), runner.canonicalize().unwrap());
            assert_eq!(
                command.get_current_dir(),
                Some(
                    bundle
                        .join("Contents/Resources")
                        .canonicalize()
                        .unwrap()
                        .as_path()
                )
            );
            assert_eq!(
                command
                    .get_envs()
                    .filter(|(_, value)| *value == Some(std::ffi::OsStr::new("391540")))
                    .count(),
                2
            );
        }
    }
    assert_eq!(
        fs::metadata(runner).unwrap().permissions().mode() & 0o777,
        0o740
    );
    // A Windows-named data executable inside the app does not establish Windows compatibility.
    put(
        &bundle,
        "Contents/Resources/UNDERTALE.exe",
        b"data executable",
    );
    let windows = catalog_command("undertale", false, Platform::Windows);
    assert!(command_for_platform(
        &windows,
        &bundle.join("Contents/Resources"),
        Platform::Windows
    )
    .is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn app_bundle_capability_uses_launchservices_and_preserves_drifter_layouts() {
    let temp = tempfile::tempdir().unwrap();
    let bundle = temp.path().join("TheDrifter.app");
    put(&bundle, "Contents/MacOS/The Drifter", b"runner");
    put(
        &bundle,
        "Contents/Resources/Data/globalgamemanagers",
        b"data",
    );
    for patched in [false, true] {
        let config = catalog_command("drifter", patched, Platform::MacOS);
        for path in [temp.path(), bundle.as_path()] {
            let command = command_for_platform(&config, path, Platform::MacOS).unwrap();
            assert_eq!(command.get_program(), "/usr/bin/open");
            assert_eq!(
                command.get_args().collect::<Vec<_>>(),
                vec![bundle.canonicalize().unwrap()]
            );
            assert_eq!(
                command.get_current_dir(),
                Some(temp.path().canonicalize().unwrap().as_path())
            );
        }
    }
    fs::remove_file(bundle.join("Contents/Resources/Data/globalgamemanagers")).unwrap();
    assert!(command_for_platform(
        &catalog_command("drifter", false, Platform::MacOS),
        temp.path(),
        Platform::MacOS
    )
    .is_err());
}

#[test]
fn capability_platform_mismatches_are_rejected_before_filesystem_access() {
    let bundle = config(
        json!({"root":"app_bundle","bundle_name":"Game.app","program":"Contents/MacOS/Game"}),
    );
    assert!(bundle.validate_for_platform(Platform::Windows).is_err());
    let preparation = config(json!({"program":"runner","preparation":"grim_hebrew_preferences"}));
    assert!(preparation.validate_for_platform(Platform::MacOS).is_err());
    let invalid_import = game_behavior::GameBehavior {
        launch: Some(game_behavior::LaunchProfiles {
            original: LaunchProfile {
                validate_on_import: true,
                ..Default::default()
            },
            original_macos: None,
            patched: LaunchProfile::default(),
        }),
        ..Default::default()
    };
    assert!(invalid_import.validate().is_err());
}
