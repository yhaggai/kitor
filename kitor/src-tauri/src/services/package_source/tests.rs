use super::*;

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

#[test]
fn all_catalog_patches_are_downloadable() {
    let registry = crate::services::registry_service::load_registry().unwrap();
    for game in registry.games {
        for patch in &game.patches {
            PackageSource::resolve(patch, &game.behavior).unwrap();
            if patch.version.rsplit_once("-download").is_some_and(|(_, suffix)| suffix.parse::<u64>().is_ok()) {
                assert!(patch.checksum.as_ref().is_some_and(|s| s.len() == 64), "{}", game.scummvm_id);
            }
        }
    }
}

#[test]
fn downloaded_platform_package_uses_selective_lifecycle() {
    use std::io::Write;
    use crate::services::patch_lifecycle as lifecycle;
    let d = tempfile::tempdir().unwrap();
    let game = d.path().join("game");
    let staging = d.path().join("staging");
    fs::create_dir(&game).unwrap();
    fs::create_dir(&staging).unwrap();
    fs::write(game.join("text.txt"), b"original").unwrap();
    fs::write(game.join("save.dat"), b"save").unwrap();
    let payload = d.path().join("payload");
    fs::write(&payload, b"translated").unwrap();
    let manifest = serde_json::json!({
        "schema_version": 1, "scummvm_id": "fixture", "language": "he", "version": "1",
        "filename": "fixture.zip", "platforms": {lifecycle::platform(): {"files": [
            {"path": "text.txt", "operation": "modify", "source": {"kind": "copy", "path": "payload.txt", "sha256": lifecycle::hash(&payload).unwrap()}},
            {"path": "added.txt", "operation": "install", "source": {"kind": "copy", "path": "payload.txt", "sha256": lifecycle::hash(&payload).unwrap()}}
        ]}}
    });
    let archive = d.path().join("fixture.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&archive).unwrap());
    for (name, data) in [("patch.json", serde_json::to_vec(&manifest).unwrap()), ("payload.txt", b"translated".to_vec())] {
        zip.start_file(format!("_platforms/{}/default/{name}", lifecycle::platform()), zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(&data).unwrap();
    }
    zip.finish().unwrap();
    PackageSource::Download.stage(&staging, &archive, "fixture", "he", "1", &game).unwrap();
    assert!(!staging.join("_platforms").exists());
    let journal = lifecycle::install(&staging, "patch.json", &game, &d.path().join("backups"), "fixture", "he", "1", lifecycle::platform()).unwrap();
    assert_eq!(fs::read(game.join("text.txt")).unwrap(), b"translated");
    assert_eq!(fs::read_dir(journal.join("originals")).unwrap().count(), 1);
    fs::write(game.join("later.dat"), b"later").unwrap();
    lifecycle::uninstall(&journal, &game).unwrap();
    assert_eq!(fs::read(game.join("text.txt")).unwrap(), b"original");
    assert!(!game.join("added.txt").exists());
    assert_eq!(fs::read(game.join("save.dat")).unwrap(), b"save");
    assert_eq!(fs::read(game.join("later.dat")).unwrap(), b"later");
}

#[test]
fn platform_and_edition_selection_never_falls_back_to_other_payloads() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("_platforms/windows/remastered");
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("patch.json"), b"{}").unwrap();
    assert_eq!(selected_root(d.path(), "tentacle", d.path(), "windows").unwrap(), path);
    assert!(selected_root(d.path(), "tentacle", d.path(), "osx").is_err());
    assert!(selected_root(d.path(), "other", d.path(), "windows").is_err());
    fs::write(d.path().join("patch.json"), b"{}").unwrap();
    assert!(selected_root(d.path(), "tentacle", d.path(), "windows").is_err());
}

#[test]
#[cfg(debug_assertions)]
fn arbitrary_local_packages_are_not_enabled_by_catalog_urls() {
    let mut patch=crate::services::registry_service::load_registry().unwrap().games.into_iter().flat_map(|g|g.patches).next().unwrap();
    patch.source_type="local-preview".into();patch.download_url="file:///tmp/other.zip".into();
    assert!(PackageSource::resolve(&patch,&GameBehavior::default()).is_err());
}

#[test]
#[cfg(debug_assertions)]
#[ignore = "Requires KITOR_DISCO_PREVIEW; validates actual private ZIP without touching game"]
fn disco_local_preview_resolves_and_stages_verified_archive() {
    let registry=crate::services::registry_service::load_registry().unwrap();
    let game=registry.games.iter().find(|g|g.scummvm_id=="disco-elysium").unwrap();
    let patch=game.patches.first().expect("Local preview must be enabled");
    let source=PackageSource::resolve(patch,&game.behavior).unwrap();
    let PackageSource::LocalPreview(path)=&source else {panic!("Expected local source")};
    let tmp=tempfile::tempdir().unwrap();let staging=tmp.path().join("stage");fs::create_dir(&staging).unwrap();
    source.stage(&staging,path,"disco-elysium","he",&patch.version,tmp.path()).unwrap();
    let manifest:serde_json::Value=serde_json::from_slice(&fs::read(staging.join("patch.json")).unwrap()).unwrap();
    assert_eq!(manifest["scummvm_id"],"disco-elysium");assert_eq!(manifest["version"],patch.version);
    let mut altered=patch.clone();altered.checksum=Some("0".repeat(64));assert!(PackageSource::resolve(&altered,&game.behavior).is_err());
}
