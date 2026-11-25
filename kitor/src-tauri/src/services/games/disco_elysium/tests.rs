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
fn detect(path: &Path) -> Option<PathBuf> {
    crate::services::registry_service::load_registry()
        .ok()
        .and_then(|registry| {
            registry
                .games
                .iter()
                .find(|game| game.scummvm_id == "disco-elysium")
                .and_then(|game| crate::services::game_adapters::detect(game, path))
        })
}
