use super::*;

#[test]
fn old_registry_recovers_reviewed_behavior_without_trusting_saved_args() {
    let bundled: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut old = bundled.clone();
    for game in &mut old.games {
        game.behavior = Default::default();
        game.launch_config = Some(LaunchConfig {
            launcher: None,
            subdirectory: Some("obsolete".into()),
            args: vec!["obsolete".into()],
        });
    }
    prepare_behaviors(&mut old, &bundled).unwrap();
    for (restored, expected) in old.games.iter().zip(&bundled.games) {
        assert_eq!(
            serde_json::to_value(&restored.behavior).unwrap(),
            serde_json::to_value(&expected.behavior).unwrap()
        );
    }
}

#[test]
fn saved_bundled_entry_migrates_to_reviewed_download() {
    let built: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut external = built.clone();
    let game = external
        .games
        .iter_mut()
        .find(|g| g.scummvm_id == "jenny-leclue")
        .unwrap();
    let original = game.patches[0].clone();
    game.patches[0].source_type = "bundled".into();
    game.patches[0].download_url = "bundled://jenny-leclue/he".into();
    game.patches[0].version = "old".into();
    game.patches[0].release_revision += 1;
    migrate_bundled_packages(&mut external, &built);
    let resolved = &external
        .games
        .iter()
        .find(|g| g.scummvm_id == "jenny-leclue")
        .unwrap()
        .patches[0];
    assert_eq!(resolved.source_type, "github");
    assert_eq!(resolved.download_url, original.download_url);
    assert_eq!(resolved.version, original.version);
    assert_eq!(resolved.release_revision, original.release_revision);
}

#[test]
fn moomin_download_remains_available_in_the_loaded_registry() {
    let registry = load_registry().unwrap();
    let game = registry
        .games
        .iter()
        .find(|game| game.scummvm_id == "moomin-midsummer-madness")
        .expect("Moomin game is present");
    assert!(game.behavior.bundled_resource.is_none());
    assert!(game.patches.iter().any(|patch| {
        patch.language == "he" && patch.source_type == "github" && patch.checksum.is_some()
    }));
}

#[test]
fn saved_moomin_bundle_migrates_to_download_and_clears_embedded_behavior() {
    let built: Registry = serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut saved = built.clone();
    let game = saved.games.iter_mut().find(|g| g.scummvm_id == "moomin-midsummer-madness").unwrap();
    game.patches[0].source_type = "bundled".into();
    game.patches[0].download_url = "bundled://moomin-midsummer-madness/windows".into();
    game.behavior.bundled_resource = Some("moomin-midsummer-madness".into());
    migrate_bundled_packages(&mut saved, &built);
    let game = saved.games.iter().find(|g| g.scummvm_id == "moomin-midsummer-madness").unwrap();
    assert_eq!(game.patches[0].source_type, "github");
    assert!(game.patches[0].download_url.starts_with("https://"));
    assert!(game.behavior.bundled_resource.is_none());
}

#[cfg(not(debug_assertions))]
#[test]
fn production_registry_ignores_external_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let external = dir.path().join("registry.json");
    std::fs::write(&external, r#"{"games":[],"attacker":"override"}"#).unwrap();
    assert_eq!(registry_content(&external).unwrap(),
        include_str!("../../../../patches/registry.json"));
}

#[test]
fn tlj_catalog_groups_variants_and_keeps_the_best_release_for_each_build() {
    let registry: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut game = registry.games.into_iter().find(|game| game.scummvm_id == "tlj").unwrap();
    let gog = game.patches[0].clone();
    let steam = game.patches[1].clone();
    assert_eq!(game.steam_app_id, Some(6310));
    assert!(game.gog.is_some());
    assert_eq!(game.catalog_patches().count(), 1);
    assert_eq!(game.compatible_patches(|path| gog.match_files.get(path).cloned())[0].version, gog.version);
    assert_eq!(game.compatible_patches(|path| steam.match_files.get(path).cloned())[0].version, steam.version);
    assert!(game.compatible_patches(|path| {
        (path == "x.xarc").then(|| steam.match_files[path].clone())
    }).is_empty());

    let mut update = steam.clone();
    update.version = "new-steam-translation".into();
    update.release_revision = 1;
    game.patches.push(update);
    let mut another_build = steam.clone();
    another_build.version = "unrelated-new-game-build".into();
    another_build.release_revision = 2;
    another_build.match_files.insert("x.xarc".into(), "00".repeat(32));
    game.patches.push(another_build);
    assert_eq!(game.compatible_patches(|path| steam.match_files.get(path).cloned())[0].version,
        "new-steam-translation");
    assert_eq!(game.compatible_patches(|path| gog.match_files.get(path).cloned())[0].version, gog.version);
}

#[test]
fn active_patch_fingerprints_follow_verified_journal_bytes() {
    use super::super::patch_lifecycle;
    use std::fs;

    let registry: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let mut game = registry.games.into_iter().find(|game| game.scummvm_id == "tlj").unwrap();
    game.patches.truncate(1);
    let patch = &mut game.patches[0];
    let fixture = tempfile::tempdir().unwrap();
    let target = fixture.path().join("game");
    let package = fixture.path().join("package");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&package).unwrap();
    let original = target.join("x.xarc");
    let payload = package.join("replacement");
    fs::write(&original, "original").unwrap();
    fs::write(&payload, "patched").unwrap();
    let original_hash = patch_lifecycle::hash(&original).unwrap();
    let installed_hash = patch_lifecycle::hash(&payload).unwrap();
    patch.match_files = BTreeMap::from([("x.xarc".into(), original_hash.clone())]);
    let manifest = serde_json::json!({
        "schema_version": 1, "scummvm_id": "tlj", "language": "he",
        "version": patch.version, "filename": "test.zip", "platforms": {"osx": {}},
        "files": [{
            "path": "x.xarc", "operation": "modify", "original_sha256": original_hash,
            "installed_sha256": installed_hash,
            "source": {"kind": "copy", "path": "replacement", "sha256": installed_hash}
        }]
    });
    fs::write(package.join("patch.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
    let journal = patch_lifecycle::install(
        &package, "patch.json", &target, &fixture.path().join("backups"),
        "tlj", "he", &game.patches[0].version, "osx",
    ).unwrap();
    assert!(game.patches_for_target(&target, std::iter::empty::<&Path>()).is_empty());
    assert_eq!(game.patches_for_target(&target, [&journal]).len(), 1);
    fs::write(&original, "changed outside Kitor").unwrap();
    assert!(game.patches_for_target(&target, [&journal]).is_empty());
}
