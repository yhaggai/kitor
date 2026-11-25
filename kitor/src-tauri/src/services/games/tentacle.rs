//! Independent classic Hebrew patch, with optional extraction from Remastered.
use std::path::{Path, PathBuf};
#[cfg(any(test, feature = "standalone-patch-tools"))]
use std::fs;
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub const VERSION: &str = "2.0-beta2-kitor1";

pub fn detect_classic(path: &Path) -> Option<PathBuf> {
    super::super::game_detection::detect_registered("tentacle", path, Some(super::super::registry_service::DetectionEdition::Classic))
}
#[cfg(feature = "standalone-patch-tools")]
pub fn detect_remastered(path: &Path) -> Option<PathBuf> {
    super::super::game_detection::detect_registered("day-of-the-tentacle-remastered", path, None)
}
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn detect(path: &Path) -> Option<PathBuf> {
    super::super::game_detection::detect_registered("tentacle", path, None)
}
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn stage(
    destination: &Path,
    language: &str,
    version: &str,
    target: &Path,
) -> Result<(), String> {
    if language != "he" || version != VERSION {
        return Err("Unknown classic Tentacle patch version".into());
    }
    let assets: &[(&str, &[u8])] = &[
        (
            "patch.json",
            if detect_classic(target).is_some() {
                include_bytes!("../../../../../patches-workspace/tentacle/he/patch.json").as_slice()
            } else {
                include_bytes!(
                    "../../../../../patches-workspace/tentacle/he/patch.remastered-source.json"
                )
                .as_slice()
            },
        ),
        (
            "recipes/classic.000.json",
            include_bytes!("../../../../../patches-workspace/tentacle/he/recipes/classic.000.json"),
        ),
        (
            "recipes/classic.001.json",
            include_bytes!("../../../../../patches-workspace/tentacle/he/recipes/classic.001.json"),
        ),
        (
            "recipes/tentacle.000.json",
            include_bytes!("../../../../../patches-workspace/tentacle/he/recipes/tentacle.000.json"),
        ),
        (
            "recipes/tentacle.001.json",
            include_bytes!("../../../../../patches-workspace/tentacle/he/recipes/tentacle.001.json"),
        ),
        (
            "recipes/monster.so3.json",
            include_bytes!("../../../../../patches-workspace/tentacle/he/recipes/monster.so3.json"),
        ),
        (
            "recipes/windows-tentacle.000.json",
            include_bytes!(
                "../../../../../patches-workspace/tentacle/he/recipes/windows-tentacle.000.json"
            ),
        ),
        (
            "recipes/windows-tentacle.001.json",
            include_bytes!(
                "../../../../../patches-workspace/tentacle/he/recipes/windows-tentacle.001.json"
            ),
        ),
        (
            "recipes/windows-monster.so3.json",
            include_bytes!(
                "../../../../../patches-workspace/tentacle/he/recipes/windows-monster.so3.json"
            ),
        ),
    ];
    for (name, bytes) in assets {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_complete_layouts_only() {
        let d = tempfile::tempdir().unwrap();
        let root = d.path();
        let data = root.join("Dott.app/Contents/Resources");
        fs::create_dir_all(&data).unwrap();
        fs::write(
            data.join("tenta.cle"),
            [b"KAPL\0\0\x80?".as_slice(), &[0; 40]].concat(),
        )
        .unwrap();
        assert!(detect(root).is_none());
        fs::create_dir_all(root.join("Dott.app/Contents/MacOS")).unwrap();
        fs::write(root.join("Dott.app/Contents/MacOS/Dott"), b"runner").unwrap();
        for p in [root.to_path_buf(), root.join("Dott.app"), data.clone()] {
            assert_eq!(detect(&p).unwrap(), root.canonicalize().unwrap());
        }
        fs::write(data.join("tenta.cle"), b"unrelated").unwrap();
        assert!(detect(root).is_none());
        fs::write(
            root.join("TENTA.CLE"),
            [b"KAPL\0\0\x80?".as_slice(), &[0; 40]].concat(),
        )
        .unwrap();
        assert!(detect(root).is_none());
        fs::write(root.join("DOTT.EXE"), b"runner").unwrap();
        assert!(detect(root).is_some());
    }
}

#[cfg(test)]
mod owned_tests {
    use crate::services::patch_lifecycle as lifecycle;
    use super::*;
    #[test]
    #[ignore = "requires KITOR_TENTACLE_WINDOWS; validates owned Windows data on disposable copy"]
    fn windows_archive_roundtrip() {
        let source = PathBuf::from(std::env::var("KITOR_TENTACLE_WINDOWS").unwrap());
        let d = tempfile::tempdir().unwrap();
        let game = d.path().join("game");
        fs::create_dir(&game).unwrap();
        fs::copy(source.join("tenta.cle"), game.join("tenta.cle")).unwrap();
        fs::write(game.join("Dott.exe"), b"runner").unwrap();
        let before = lifecycle::hash(&game.join("tenta.cle")).unwrap();
        let package = d.path().join("package");
        stage(&package, "he", VERSION, &game).unwrap();
        fs::write(game.join("save.dat"), b"save before").unwrap();
        let journal = lifecycle::install(
            &package,
            "patch.json",
            &game,
            &d.path().join("recovery"),
            "tentacle",
            "he",
            VERSION,
            "windows",
        )
        .unwrap();
        assert_eq!(fs::read_dir(journal.join("originals")).unwrap().count(), 0);
        assert_eq!(
            lifecycle::hash(&game.join("hebrew/tentacle.000")).unwrap(),
            "21bdb9663389bd01b6bbb1cb50bdce985ec68adcb5fae3efcd6f384288066536"
        );
        assert_eq!(
            lifecycle::hash(&game.join("hebrew/monster.so3")).unwrap(),
            "281f0f8a8bacb0464bfb74b0d3af3b086a2a638df5290ac36658207e0f0330ba"
        );
        fs::write(game.join("later.sav"), b"save after").unwrap();
        lifecycle::uninstall(&journal, &game).unwrap();
        assert!(!game.join("hebrew").exists());
        assert_eq!(lifecycle::hash(&game.join("tenta.cle")).unwrap(), before);
        assert_eq!(fs::read(game.join("save.dat")).unwrap(), b"save before");
        assert_eq!(fs::read(game.join("later.sav")).unwrap(), b"save after");
    }
}
