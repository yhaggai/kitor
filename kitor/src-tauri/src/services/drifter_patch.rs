//! The Drifter loads custom translations from Unity's persistent data directory on macOS.
use super::patch_lifecycle;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const VERSION: &str = "2026-07-09-kitor2";
const MANIFEST: &[u8] = include_bytes!("../../../../patches-workspace/drifter/he/patch.json");
const PAYLOAD: &[u8] =
    include_bytes!("../../../../patches-workspace/drifter/he/payload/translation.csv");
const MAC_ASSEMBLY_SHA256: &str =
    "eafbbb982652ebfdb378d4b4b45d719d906f441716bcb76353754ecdc0616916";

pub fn stage(dir: &Path, language: &str, version: &str) -> Result<(), String> {
    if language != "he" || version != VERSION {
        return Err("Unknown bundled Drifter patch version".into());
    }
    fs::create_dir_all(dir.join("payload")).map_err(|e| e.to_string())?;
    fs::write(dir.join("patch.json"), MANIFEST).map_err(|e| e.to_string())?;
    fs::write(dir.join("payload/translation.csv"), PAYLOAD).map_err(|e| e.to_string())
}

pub fn validate_macos_build(game: &Path) -> Result<(), String> {
    let assembly =
        patch_lifecycle::safe_path(game, "Contents/Resources/Data/Managed/Assembly-CSharp.dll")?;
    if patch_lifecycle::hash(&assembly)? != MAC_ASSEMBLY_SHA256 {
        return Err("This Drifter macOS build has not been validated for the Hebrew patch (expected Steam build 22758595)".into());
    }
    Ok(())
}

// Require the game's existing user-data directory. Starting the game creates it.
// Never accept a package-supplied arbitrary external path, or follow symlinks.
pub fn macos_target(home: &Path) -> Result<PathBuf, String> {
    let target =
        patch_lifecycle::safe_path(home, "Library/Application Support/com.Powerhoof.TheDrifter")?;
    if !target.is_dir() {
        return Err("Start The Drifter once, then retry installing the translation".into());
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn userdata_roundtrip_preserves_saves_and_original_translation() {
        for original in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let home = temp.path().join("home");
            let data = home.join("Library/Application Support/com.Powerhoof.TheDrifter");
            fs::create_dir_all(&data).unwrap();
            fs::write(data.join("Save1.sav"), b"save").unwrap();
            if original {
                fs::write(data.join("translation.csv"), b"previous translation").unwrap();
            }
            let package = temp.path().join("package");
            stage(&package, "he", VERSION).unwrap();
            let target = macos_target(&home).unwrap();
            let storage = temp.path().join("transactions");
            let install = || {
                patch_lifecycle::install(
                    &package,
                    "patch.json",
                    &target,
                    &storage,
                    "drifter",
                    "he",
                    VERSION,
                    "osx",
                )
                .unwrap()
            };
            let journal = install();
            assert_eq!(install(), journal);
            assert_eq!(fs::read(data.join("translation.csv")).unwrap(), PAYLOAD);
            fs::write(data.join("later.sav"), b"later save").unwrap();
            patch_lifecycle::uninstall(&journal, &target).unwrap();
            if original {
                assert_eq!(
                    fs::read(data.join("translation.csv")).unwrap(),
                    b"previous translation"
                );
            } else {
                assert!(!data.join("translation.csv").exists());
            }
            assert_eq!(fs::read(data.join("Save1.sav")).unwrap(), b"save");
            assert_eq!(fs::read(data.join("later.sav")).unwrap(), b"later save");
        }
    }
    #[test]
    fn rejects_missing_data_and_unknown_package() {
        let temp = tempfile::tempdir().unwrap();
        assert!(macos_target(temp.path()).is_err());
        assert!(stage(temp.path(), "he", "unknown").is_err());
        assert!(validate_macos_build(temp.path()).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_user_data() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("Library/Application Support")).unwrap();
        std::os::unix::fs::symlink(
            temp.path(),
            temp.path()
                .join("Library/Application Support/com.Powerhoof.TheDrifter"),
        )
        .unwrap();
        assert!(macos_target(temp.path()).is_err());
    }
}
