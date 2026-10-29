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
        ) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
            let mut found = std::collections::BTreeMap::new();
            for entry in std::fs::read_dir(root.join(relative)).unwrap() {
                let entry = entry.unwrap();
                let path = relative.join(entry.file_name());
                if entry.file_type().unwrap().is_dir() {
                    found.extend(files(root, &path));
                } else {
                    found.insert(path, std::fs::read(entry.path()).unwrap());
                }
            }
            found
        }
        assert_eq!(
            files(old.path(), Path::new("")),
            files(new.path(), Path::new("")),
            "{id}"
        );
    }
}
