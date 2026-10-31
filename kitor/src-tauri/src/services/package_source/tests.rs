use super::*;

#[test]
fn every_embedded_package_matches_its_catalog_identity() {
    let registry = crate::services::registry_service::load_registry().unwrap();
    for game in registry.games {
        for patch in &game.patches {
            if patch.source_type != "bundled" {
                continue;
            }
            let source = PackageSource::resolve(patch, &game.behavior).unwrap();
            if let PackageSource::Embedded(key) = source {
                for platform in ["windows", "osx"] {
                    if let Some(support) = &game.platforms {
                        if (platform == "windows" && !support.windows) || (platform == "osx" && !support.osx) {
                            continue;
                        }
                    }
                    let assets = embedded_assets(&format!("{key}/{platform}")).unwrap();
                    let manifest = assets
                        .iter()
                        .find(|(name, _)| *name == "patch.json")
                        .unwrap()
                        .1;
                    let package: crate::services::patch_lifecycle::Package =
                        serde_json::from_slice(manifest).unwrap();
                    assert_eq!(package.scummvm_id, game.scummvm_id);
                    assert_eq!(package.version, patch.version);
                    assert_eq!(package.language, patch.language);
                    assert_eq!(package.release_revision, patch.release_revision);
                    let staging = tempfile::tempdir().unwrap();
                    for (name, _) in assets {
                        crate::services::patch_lifecycle::safe_path(staging.path(), name).unwrap();
                    }
                }
            }
        }
    }
}

#[test]
fn embedded_staging_preserves_existing_package_bytes() {
    let ids: &[&str] = if cfg!(target_os = "windows") {
        &[
            "drifter",
            "grim-fandango-remastered",
            "monkey-se-native",
            "monkey2-se-native",
            "day-of-the-tentacle-remastered",
        ]
    } else if cfg!(target_os = "macos") {
        &["drifter", "owlboy", "day-of-the-tentacle-remastered"]
    } else {
        return;
    };
    for &id in ids {
        let registry = crate::services::registry_service::load_registry().unwrap();
        let game = registry
            .games
            .into_iter()
            .find(|g| g.scummvm_id == id)
            .unwrap();
        let patch = &game.patches[0];
        let old = tempfile::tempdir().unwrap();
        let new = tempfile::tempdir().unwrap();
        match id {
            "drifter" => {
                crate::services::drifter_patch::stage(old.path(), &patch.language, &patch.version)
                    .unwrap()
            }
            "owlboy" => {
                crate::services::owlboy::stage(old.path(), &patch.language, &patch.version).unwrap()
            }
            "grim-fandango-remastered" => {
                crate::services::grim_remastered::stage(old.path(), &patch.language, &patch.version)
                    .unwrap()
            }
            "monkey-se-native" | "monkey2-se-native" => crate::services::monkey_native::stage(
                old.path(),
                id,
                &patch.language,
                &patch.version,
            )
            .unwrap(),
            _ => crate::services::tentacle_remastered::stage(
                old.path(),
                &patch.language,
                &patch.version,
            )
            .unwrap(),
        }
        let source = PackageSource::resolve(patch, &game.behavior).unwrap();
        source
            .stage(
                new.path(),
                Path::new("unused.zip"),
                id,
                &patch.language,
                &patch.version,
                old.path(),
            )
            .unwrap();
        fn files(
            root: &Path,
            relative: &Path,
        ) -> std::collections::BTreeMap<std::path::PathBuf, String> {
            let mut found = std::collections::BTreeMap::new();
            for entry in std::fs::read_dir(root.join(relative)).unwrap() {
                let entry = entry.unwrap();
                let path = relative.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    found.extend(files(root, &path));
                } else {
                    found.insert(path, crate::services::patch_lifecycle::hash(&entry.path()).unwrap());
                }
            }
            found
        }
        // The legacy Owlboy helper stages both platforms. Compare exactly the
        // current platform's declared assets, including the manifest, rather
        // than expecting unused assets from the other platform to be shipped.
        let PackageSource::Embedded(key) = &source else { panic!("Expected embedded package: {id}") };
        let platform = if cfg!(target_os = "windows") { "windows" } else { "osx" };
        let names: std::collections::BTreeSet<std::path::PathBuf> =
            embedded_assets(&format!("{key}/{platform}")).unwrap().iter()
                .map(|(name, _)| std::path::PathBuf::from(name)).collect();
        let mut expected = files(old.path(), Path::new(""));
        expected.retain(|name, _| names.contains(name));
        let actual = files(new.path(), Path::new(""));
        assert_eq!(expected.keys().cloned().collect::<std::collections::BTreeSet<_>>(), names, "legacy source missing asset: {id}");
        assert_eq!(actual.keys().cloned().collect::<std::collections::BTreeSet<_>>(), names, "unexpected staged assets: {id}");
        assert_eq!(expected, actual, "{id}");
    }
}

#[test]
fn silver_pines_requires_both_steam_install_markers() {
    let registry = crate::services::registry_service::load_registry().unwrap();
    let game = registry.games.iter().find(|g| g.scummvm_id == "silver-pines").unwrap();
    assert_eq!(game.steam_app_id, Some(2333000));
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("Carpenter.exe"), []).unwrap();
    assert!(crate::services::game_adapters::detect(game, root).is_none());
    let strings = root.join("Carpenter_Data/StreamingAssets/aa/StandaloneWindows64/localization-string-tables-english(en)_assets_all.bundle");
    std::fs::create_dir_all(strings.parent().unwrap()).unwrap();
    std::fs::write(strings, []).unwrap();
    assert_eq!(crate::services::game_adapters::detect(game, root), Some(root.canonicalize().unwrap()));
}
