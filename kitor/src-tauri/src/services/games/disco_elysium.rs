//! Final Cut native layouts. app.info identifies the title, not a generic Unity filename.
use std::{
    fs,
    path::{Path, PathBuf},
};
fn child(root: &Path, relative: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path = fs::read_dir(&path)
            .ok()?
            .flatten()
            .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(part))?
            .path();
    }
    Some(path)
}
fn complete(root: &Path, mac: bool) -> bool {
    let data = if mac {
        "Contents/Resources/Data"
    } else {
        "disco_Data"
    };
    let exe = if mac {
        "Contents/MacOS/Disco Elysium"
    } else {
        "disco.exe"
    };
    let platform = if mac { "OSX" } else { "Windows" };
    let Some(info) = child(root, &format!("{data}/app.info")) else {
        return false;
    };
    if fs::read_to_string(info)
        .ok()
        .is_none_or(|s| s.replace('\r', "").trim() != "ZAUM Studio\nDisco Elysium")
    {
        return false;
    }
    [
        exe.to_owned(),
        format!("{data}/globalgamemanagers"),
        format!("{data}/StreamingAssets/AssetBundles/{platform}/lockits/arabic"),
        format!("{data}/StreamingAssets/AssetBundles/{platform}/fonts/arabic"),
    ]
    .iter()
    .all(|name| child(root, name).is_some_and(|p| p.is_file()))
}
pub fn detect(path: &Path) -> Option<PathBuf> {
    // Accept install roots, .app bundles, and their data folders without saving
    // an inner resource directory as the installation root.
    let mut roots = vec![path.to_path_buf()];
    roots.extend(path.ancestors().skip(1).take(3).map(Path::to_path_buf));
    if let Some(app) = child(path, "disco.app") {
        roots.push(app);
    }
    for root in roots {
        if complete(&root, true) || complete(&root, false) {
            return root.canonicalize().ok();
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;
    fn layout(root: &Path, mac: bool) {
        let data = if mac {
            "Contents/Resources/Data"
        } else {
            "disco_Data"
        };
        let platform = if mac { "OSX" } else { "Windows" };
        for rel in [
            if mac {
                "Contents/MacOS/Disco Elysium".into()
            } else {
                "disco.exe".into()
            },
            format!("{data}/globalgamemanagers"),
            format!("{data}/StreamingAssets/AssetBundles/{platform}/lockits/arabic"),
            format!("{data}/StreamingAssets/AssetBundles/{platform}/fonts/arabic"),
            format!("{data}/app.info"),
        ] {
            let p = root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"fixture").unwrap();
        }
        fs::write(
            root.join(format!("{data}/app.info")),
            b"ZAUM Studio\nDisco Elysium",
        )
        .unwrap();
    }
    #[test]
    fn disco_layouts_resolve_same_root_and_reject_lookalikes() {
        for mac in [true, false] {
            let t = tempfile::tempdir().unwrap();
            let root = if mac {
                t.path().join("disco.app")
            } else {
                t.path().join("game")
            };
            layout(&root, mac);
            let expected = root.canonicalize().unwrap();
            let data = if mac {
                "Contents/Resources/Data"
            } else {
                "disco_Data"
            };
            assert_eq!(detect(&root), Some(expected.clone()));
            assert_eq!(detect(&root.join(data)), Some(expected.clone()));
            if mac {
                assert_eq!(detect(t.path()), Some(expected.clone()));
            }
            assert_eq!(detect(&root), detect(&root));
            fs::write(
                root.join(data).join("app.info"),
                b"Another Studio\nAnother Unity Game",
            )
            .unwrap();
            assert!(detect(&root).is_none());
            fs::write(
                root.join(data).join("app.info"),
                b"ZAUM Studio\nDisco Elysium",
            )
            .unwrap();
            fs::remove_file(root.join(data).join("globalgamemanagers")).unwrap();
            assert!(detect(&root).is_none());
        }
    }
    #[test]
    fn disco_case_insensitive_windows_layout() {
        let t = tempfile::tempdir().unwrap();
        layout(t.path(), false);
        fs::rename(t.path().join("disco.exe"), t.path().join("DISCO.EXE")).unwrap();
        assert_eq!(detect(t.path()), Some(t.path().canonicalize().unwrap()));
    }
}
/// Explicit opt-in for private testing; never compiled into release builds.
#[cfg(debug_assertions)]
pub fn preview_archive() -> Result<Option<PathBuf>, String> {
    let Some(path) = std::env::var_os("KITOR_DISCO_PREVIEW") else {
        return Ok(None);
    };
    let path = PathBuf::from(path)
        .canonicalize()
        .map_err(|e| format!("Local Disco preview: {e}"))?;
    if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("zip") {
        return Err("KITOR_DISCO_PREVIEW must point to a local ZIP".into());
    }
    Ok(Some(path))
}
#[cfg(debug_assertions)]
pub fn preview_definition(
    path: &Path,
) -> Result<super::super::registry_service::PatchDefinition, String> {
    use super::super::{patch_lifecycle, registry_service::PatchDefinition};
    Ok(PatchDefinition {
        release_revision:0,install_manifest:Some("patch.json".into()),language:"he".into(),
        version:"0.1.0-preview1".into(),author:"Kitor local preview".into(),
        download_url:"local-preview://disco-elysium".into(),source_type:"local-preview".into(),
        checksum:Some(patch_lifecycle::hash(path)?),file_size:Some(fs::metadata(path).map_err(|e|e.to_string())?.len() as i64),
        description:"Private Hebrew preview (4,165 entries). Select Arabic in-game. Untranslated text remains Arabic.".into(),
        filename:Some("Disco-Elysium-Hebrew-0.1.0-preview1.zip".into()),post_install_script:None,
    })
}
