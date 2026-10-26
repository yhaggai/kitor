#[path = "../../../src-tauri/src/services/owlboy.rs"]
pub mod owlboy;
#[path = "../../../src-tauri/src/services/drifter_patch.rs"]
pub mod drifter_patch;
#[path = "../../../src-tauri/src/services/patch_lifecycle.rs"]
pub mod patch_lifecycle;
#[path = "../../../src-tauri/src/services/package_auth.rs"]
pub mod package_auth;
#[path = "../../../src-tauri/src/services/registry_service.rs"]
pub mod registry_service;
pub mod services {
    pub use crate::registry_service;
}

#[cfg(test)]
mod payload_tests {
    use crate::patch_lifecycle::{self, Operation, Package, Source};
    use std::{fs, path::Path};

    #[test]
    fn all_copy_packages_restore_only_affected_originals() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../patches-workspace");
        let mut exercised = 0;
        let filter = std::env::var("KITOR_PAYLOAD_GAME").ok();
        for game in fs::read_dir(workspace).unwrap() {
            let game = game.unwrap();
            if filter
                .as_ref()
                .is_some_and(|name| !name.split(',').any(|item| game.file_name().to_string_lossy() == item))
            {
                continue;
            }
            let root = game.path().join("he");
            for manifest_name in ["patch.json", "patch.windows.candidate.json", "patch.osx.candidate.json"] {
                if manifest_name != "patch.json" && std::env::var_os("KITOR_SKIP_CANDIDATES").is_some() { continue; }
                let manifest = root.join(manifest_name);
                if !manifest.is_file() {
                    continue;
                }
                let package: Package =
                    serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
                for platform in ["windows", "osx"] {
                    let Some(plan) = package.platforms.get(platform) else {
                        continue;
                    };
                    let files = if plan.files.is_empty() {
                        &package.files
                    } else {
                        &plan.files
                    };
                    if files.is_empty()
                        || !files.iter().all(|f| {
                            matches!(f.source, Some(Source::Copy { .. }))
                                && f.original_sha256.is_none()
                                && f.original_md5.is_none()
                                && f.operation != Operation::Delete
                        })
                    {
                        continue;
                    }
                    eprintln!(
                        "Starting copy payload roundtrip: {} {platform}",
                        package.scummvm_id
                    );
                    let fixture = tempfile::tempdir().unwrap();
                    let target = fixture.path().join("game");
                    let storage = fixture.path().join("backups");
                    fs::create_dir(&target).unwrap();
                    let first_install = files.iter().find(|f| f.operation == Operation::Install);
                    let originals: std::collections::HashSet<_> = files.iter()
                        .filter(|f| f.operation == Operation::Modify || first_install.map(|first| &first.path) == Some(&f.path))
                        .map(|f| f.path.clone()).collect();
                    for change in files.iter().filter(|f| originals.contains(&f.path)) {
                        let path = target.join(&change.path);
                        fs::create_dir_all(path.parent().unwrap()).unwrap();
                        fs::write(path, b"original fixture bytes").unwrap();
                    }
                    let save = target.join("__kitor_unrelated_save_fixture__");
                    fs::write(&save, b"save before installation").unwrap();
                    let journal = patch_lifecycle::install(
                        &root,
                        manifest_name,
                        &target,
                        &storage,
                        &package.scummvm_id,
                        &package.language,
                        &package.version,
                        platform,
                    )
                    .unwrap();
                    assert_eq!(
                        fs::read_dir(journal.join("originals")).unwrap().count(),
                        originals.len()
                    );
                    for change in files {
                        let Source::Copy { path, .. } = change.source.as_ref().unwrap() else {
                            unreachable!()
                        };
                        assert_eq!(
                            patch_lifecycle::hash(&target.join(&change.path)).unwrap(),
                            patch_lifecycle::hash(&root.join(path)).unwrap()
                        );
                    }
                    let later = target.join("__kitor_added_after_install_fixture__");
                    fs::write(&later, b"added later").unwrap();
                    patch_lifecycle::uninstall(&journal, &target).unwrap();
                    for change in files {
                        if originals.contains(&change.path) {
                            assert_eq!(
                                fs::read(target.join(&change.path)).unwrap(),
                                b"original fixture bytes"
                            );
                        } else {
                            assert!(!target.join(&change.path).exists());
                        }
                    }
                    assert_eq!(fs::read(save).unwrap(), b"save before installation");
                    assert_eq!(fs::read(later).unwrap(), b"added later");
                    eprintln!("Copy payload roundtrip: {} {platform}", package.scummvm_id);
                    exercised += 1;
                }
            }
        }
        assert!(exercised > 0);
    }
}

#[path = "../../../src-tauri/src/services/reconstruction.rs"]
pub mod reconstruction;

#[path = "../../../src-tauri/src/services/tentacle.rs"]
pub mod tentacle;

#[path = "../../../src-tauri/src/services/tentacle_remastered.rs"]
pub mod tentacle_remastered;
