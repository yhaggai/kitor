use std::path::PathBuf;
use super::*;
pub fn fixture(root: &Path) -> PathBuf {
    let bundle = root.join("oWlBoY.ApP");
    let data = bundle.join("Contents/Resources");
    fs::create_dir_all(data.join("Data/Localizations")).unwrap();
    fs::create_dir_all(data.join("Data/Maps")).unwrap();
    fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
    for name in [
        "Owlboy.exe",
        "FNA.dll",
        "Data/versioninfo.txt",
        "Data/Localizations/de.csv",
    ] {
        fs::write(data.join(name), b"fixture").unwrap();
    }
    fs::write(bundle.join("Contents/MacOS/Owlboy"), b"fixture").unwrap();
    data
}
#[test]
fn resolves_parent_bundle_data_and_rejects_incomplete() {
    let t = tempfile::tempdir().unwrap();
    assert!(detect(t.path()).is_none());
    let data = fixture(t.path());
    for selected in [t.path(), data.parent().unwrap().parent().unwrap(), &data] {
        assert_eq!(detect(selected), Some(data.canonicalize().unwrap()));
        if cfg!(target_os = "macos") {
            assert_eq!(
                launch_command(selected).unwrap().get_program(),
                "/usr/bin/open"
            );
        } else {
            assert!(launch_command(selected).is_err());
        }
    }
    fs::remove_file(data.join("FNA.dll")).unwrap();
    assert!(detect(t.path()).is_none());
}
#[test]
fn bundled_payload_matches_manifest() {
    use sha2::{Digest, Sha256};
    let t = tempfile::tempdir().unwrap();
    stage(t.path(), "he", VERSION).unwrap();
    for (manifest, platform) in [(MANIFEST, "osx"), (WINDOWS_MANIFEST, "windows")] {
        let m: serde_json::Value = serde_json::from_slice(manifest).unwrap();
        for f in m["platforms"][platform]["files"].as_array().unwrap() {
            let bytes = fs::read(t.path().join(f["source"]["path"].as_str().unwrap())).unwrap();
            assert_eq!(
                format!("{:x}", Sha256::digest(bytes)),
                f["source"]["sha256"].as_str().unwrap()
            );
        }
    }
    assert!(stage(t.path(), "he", "unknown").is_err());
}
#[test]
fn download_catalog_selects_platform_specific_payloads() {
    use sha2::{Digest, Sha256};
    let catalog: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../patches/download-packages.json"
    ))
    .unwrap();
    for (manifest, platform) in [(MANIFEST, "osx"), (WINDOWS_MANIFEST, "windows")] {
        let source = catalog["owlboy"][platform]["default"].as_str().unwrap();
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|root| root.join("patches-workspace").is_dir())
            .unwrap()
            .to_path_buf();
        assert_eq!(fs::read(workspace.join(source)).unwrap(), manifest);
        let m: serde_json::Value = serde_json::from_slice(manifest).unwrap();
        for f in m["platforms"][platform]["files"].as_array().unwrap() {
            let path = workspace
                .join(source)
                .parent()
                .unwrap()
                .join(f["source"]["path"].as_str().unwrap());
            let bytes = fs::read(path).unwrap();
            assert_eq!(
                format!("{:x}", Sha256::digest(bytes)),
                f["source"]["sha256"].as_str().unwrap()
            );
        }
    }
}
#[test]
fn windows_detection_and_launch() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().join("Owlboy");
    fs::create_dir_all(root.join("Data/Localizations")).unwrap();
    fs::create_dir_all(root.join("Data/Maps")).unwrap();
    for name in [
        "oWlBoY.ExE",
        "FNA.dll",
        "FNA3D.dll",
        "SDL2.dll",
        "Data/versioninfo.txt",
        "Data/Localizations/de.csv",
    ] {
        fs::write(root.join(name), b"fixture").unwrap();
    }
    assert_eq!(detect(&root), Some(root.canonicalize().unwrap()));
    assert_eq!(detect(&root.join("Data")), detect(&root));
    if cfg!(target_os = "windows") {
        let cmd = launch_command(&root).unwrap();
        let program = Path::new(cmd.get_program());
        let working_dir = cmd.get_current_dir().unwrap();
        let executable = root.join("oWlBoY.ExE");

        // Windows can spell the temporary directory with an 8.3 alias.
        // Check both command paths against the contents of the executable fixture.
        assert_eq!(program.file_name(), executable.file_name());
        fs::write(&executable, b"Owlboy test executable").unwrap();
        assert_eq!(fs::read(program).unwrap(), b"Owlboy test executable");
        assert_eq!(
            fs::read(working_dir.join("oWlBoY.ExE")).unwrap(),
            b"Owlboy test executable"
        );
        // Exercise the catalog route as well: it must not reject Windows before
        // reaching this platform-aware launcher (for either language edition).
        let behavior = crate::services::game_behavior::for_game("owlboy").unwrap();
        let profiles = behavior.launch.unwrap();
        for profile in [profiles.original, profiles.patched] {
            let command = crate::services::game_adapters::native_command(&profile, &root)
                .unwrap()
                .unwrap();
            assert_eq!(command.get_program(), cmd.get_program());
            assert_eq!(command.get_current_dir(), cmd.get_current_dir());
        }
    } else {
        assert!(launch_command(&root).is_err());
    }
    fs::remove_file(root.join("FNA.dll")).unwrap();
    assert!(detect(&root).is_none());
}
#[test]
#[ignore = "requires KITOR_OWLBOY_PATH pointing to pristine Windows FNA game files"]
fn windows_owned_files_roundtrip() {
    use crate::services::patch_lifecycle as lifecycle;
    let original = PathBuf::from(std::env::var_os("KITOR_OWLBOY_PATH").unwrap());
    let t = tempfile::tempdir().unwrap();
    let game = t.path().join("game");
    let package = t.path().join("package");
    fs::create_dir_all(&game).unwrap();
    fs::create_dir_all(&package).unwrap();
    stage(&package, "he", VERSION).unwrap();
    // Exercise the Windows plan explicitly, including when run on another host.
    fs::write(package.join("patch.json"), WINDOWS_MANIFEST).unwrap();
    let m: serde_json::Value = serde_json::from_slice(WINDOWS_MANIFEST).unwrap();
    let files = m["platforms"]["windows"]["files"].as_array().unwrap();
    for f in files {
        if let Some(input) = f["source"]["input"].as_str() {
            let dest = game.join(input);
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::copy(original.join(input), dest).unwrap();
        }
    }
    let sentinel = game.join("user-save.sav");
    fs::write(&sentinel, b"keep save").unwrap();
    let recovery = t.path().join("recovery");
    let journal = lifecycle::install(
        &package,
        "patch.json",
        &game,
        &recovery,
        "owlboy",
        "he",
        VERSION,
        "windows",
    )
    .unwrap();
    for f in files {
        assert_eq!(
            lifecycle::hash(&game.join(f["path"].as_str().unwrap())).unwrap(),
            f["installed_sha256"].as_str().unwrap()
        );
    }
    lifecycle::uninstall(&journal, &game).unwrap();
    for f in files {
        let path = game.join(f["path"].as_str().unwrap());
        if let Some(hash) = f["original_sha256"].as_str() {
            assert_eq!(lifecycle::hash(&path).unwrap(), hash);
        } else {
            assert!(!path.exists());
        }
    }
    assert_eq!(fs::read(&sentinel).unwrap(), b"keep save");
    // An unsupported executable must be refused before any translation is installed.
    fs::write(game.join("Owlboy.exe"), b"unsupported edition").unwrap();
    assert!(lifecycle::install(
        &package,
        "patch.json",
        &game,
        &t.path().join("rejected"),
        "owlboy",
        "he",
        VERSION,
        "windows"
    )
    .is_err());
    assert!(!game.join("Data/Localizations/he.csv").exists());
    assert_eq!(
        fs::read(game.join("Owlboy.exe")).unwrap(),
        b"unsupported edition"
    );
}

fn launch_command(path: &Path) -> Result<std::process::Command, String> {
    let data = detect(path).ok_or("Incomplete Owlboy installation")?;
    let profile = crate::services::game_behavior::for_game("owlboy")?.launch_profile(false);
    crate::services::game_adapters::native_command(&profile, &data)?
        .ok_or("Missing native profile".into())
}

fn detect(path: &Path) -> Option<PathBuf> {
    let registry = crate::services::registry_service::load_registry().ok()?;
    let game = registry.games.iter().find(|game|game.scummvm_id=="owlboy")?;
    crate::services::game_adapters::detect(game,path)
}
