//! Independent classic Hebrew patch, with optional extraction from Remastered.
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
pub const VERSION: &str = "2.0-beta2-kitor1";
fn child(root: &Path, name: &str) -> Option<PathBuf> {
    fs::read_dir(root)
        .ok()?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|e| e.path())
}
fn descend(root: &Path, parts: &[&str]) -> Option<PathBuf> {
    let mut p = root.to_path_buf();
    for part in parts {
        p = child(&p, part)?;
    }
    Some(p)
}
fn archive(path: &Path) -> bool {
    let Ok(mut f) = fs::File::open(path) else {
        return false;
    };
    let mut h = [0; 8];
    f.metadata().is_ok_and(|m| m.len() > 40)
        && f.read_exact(&mut h).is_ok()
        && h == *b"KAPL\0\0\x80?"
}
/// Resolve to the Steam/GOG installation root, never inside its signed app bundle.
/// The archive signature plus title-specific executable is the fallback detector:
/// ScummVM cannot inspect classic data while it is still inside tenta.cle.
pub fn detect_remastered(path: &Path) -> Option<PathBuf> {
    let mut roots = vec![path.to_path_buf()];
    if path
        .file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("Dott.app"))
    {
        roots.push(path.parent()?.to_path_buf());
    }
    if path
        .file_name()
        .is_some_and(|n| n.eq_ignore_ascii_case("Resources"))
    {
        roots.push(path.parent()?.parent()?.parent()?.to_path_buf());
    }
    for root in roots {
        let mac = descend(&root, &["Dott.app", "Contents", "Resources", "tenta.cle"]);
        let runner = descend(&root, &["Dott.app", "Contents", "MacOS", "Dott"]);
        if mac.is_some_and(|p| archive(&p)) && runner.is_some_and(|p| p.is_file()) {
            return root.canonicalize().ok();
        }
        let win = child(&root, "tenta.cle");
        let exe = child(&root, "Dott.exe");
        if win.is_some_and(|p| archive(&p)) && exe.is_some_and(|p| p.is_file()) {
            return root.canonicalize().ok();
        }
    }
    None
}
/// Loose classic files, including common GOG wrappers. An edition fingerprint is
/// checked again by the install plan before any originals are changed.
pub fn detect_classic(path: &Path) -> Option<PathBuf> {
    let mut roots = vec![path.to_path_buf()];
    for parts in [
        &["Contents", "Resources", "game", "game"][..],
        &["Contents", "Resources", "game"][..],
        &["classic", "en"][..],
    ] {
        if let Some(p) = descend(path, parts) {
            roots.push(p);
        }
    }
    for root in roots {
        let Some(index) = child(&root, "tentacle.000") else {
            continue;
        };
        let Some(data) = child(&root, "tentacle.001") else {
            continue;
        };
        if index.is_file() && data.is_file() && fs::metadata(&data).ok()?.len() > 0 {
            let hash = super::patch_lifecycle::hash(&index).ok()?;
            if [
                "c89a825fcb2c4ea4d9678feab9c23e924ba578293d25a5d098cb3cb39ac86cb9",
                "21bdb9663389bd01b6bbb1cb50bdce985ec68adcb5fae3efcd6f384288066536",
            ]
            .contains(&hash.as_str())
            {
                return root.canonicalize().ok();
            }
        }
    }
    None
}
pub fn detect(path: &Path) -> Option<PathBuf> {
    detect_classic(path).or_else(|| detect_remastered(path))
}
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
                include_bytes!("../../../../patches-workspace/tentacle/he/patch.json").as_slice()
            } else {
                include_bytes!(
                    "../../../../patches-workspace/tentacle/he/patch.remastered-source.json"
                )
                .as_slice()
            },
        ),
        (
            "recipes/classic.000.json",
            include_bytes!("../../../../patches-workspace/tentacle/he/recipes/classic.000.json"),
        ),
        (
            "recipes/classic.001.json",
            include_bytes!("../../../../patches-workspace/tentacle/he/recipes/classic.001.json"),
        ),
        (
            "recipes/tentacle.000.json",
            include_bytes!("../../../../patches-workspace/tentacle/he/recipes/tentacle.000.json"),
        ),
        (
            "recipes/tentacle.001.json",
            include_bytes!("../../../../patches-workspace/tentacle/he/recipes/tentacle.001.json"),
        ),
        (
            "recipes/monster.so3.json",
            include_bytes!("../../../../patches-workspace/tentacle/he/recipes/monster.so3.json"),
        ),
        (
            "recipes/windows-tentacle.000.json",
            include_bytes!(
                "../../../../patches-workspace/tentacle/he/recipes/windows-tentacle.000.json"
            ),
        ),
        (
            "recipes/windows-tentacle.001.json",
            include_bytes!(
                "../../../../patches-workspace/tentacle/he/recipes/windows-tentacle.001.json"
            ),
        ),
        (
            "recipes/windows-monster.so3.json",
            include_bytes!(
                "../../../../patches-workspace/tentacle/he/recipes/windows-monster.so3.json"
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
    use super::super::patch_lifecycle as lifecycle;
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
