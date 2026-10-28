use super::*;
fn catalog() -> Catalog {
    Catalog {
        schema_version: 1,
        catalog_revision: 1,
        patches: vec![Release {
            scummvm_id: "sq1".into(),
            language: "he".into(),
            version: "RC2-kitor1".into(),
            release_revision: 1,
            download_url: format!(
                "https://github.com/{}/releases/download/patch-sq1-he-vRC2-kitor1/patch.zip",
                repository()
            ),
            filename: "patch.zip".into(),
            checksum: "a".repeat(64),
            file_size: 123,
            author: "Translator".into(),
            description: "Corrected dialogue".into(),
            platforms: vec!["windows".into(), "osx".into()],
        }],
    }
}
#[test]
fn rejects_unsafe_or_ambiguous_catalogs() {
    let good = catalog();
    assert!(parse(&serde_json::to_vec(&good).unwrap()).is_ok());
    for case in 0..5 {
        let mut bad = good.clone();
        match case {
            0 => bad.patches[0].download_url = "https://example.com/patch.zip".into(),
            1 => bad.patches[0].filename = "../patch.zip".into(),
            2 => bad.patches[0].release_revision = 0,
            3 => bad.patches.push(bad.patches[0].clone()),
            _ => bad.schema_version = 2,
        }
        assert!(parse(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
#[test]
fn rejects_catalog_rollback_and_same_revision_mutation() {
    let old = catalog();
    assert!(validate_advance(&old, &old).is_ok());
    let mut new = old.clone();
    new.catalog_revision = 0;
    assert!(validate_advance(&old, &new).is_err());
    new.catalog_revision = 2;
    new.patches[0].checksum = "b".repeat(64);
    assert!(validate_advance(&old, &new).is_err());
    new.patches[0].release_revision = 2;
    assert!(validate_advance(&old, &new).is_err());
    new.patches[0].version = "RC2-kitor2".into();
    assert!(validate_advance(&old, &new).is_ok());
    new.patches.clear();
    assert!(validate_advance(&old, &new).is_err());
}
#[test]
fn catalog_never_replaces_launch_or_detection_configuration() {
    let mut registry: Registry =
        serde_json::from_str(include_str!("../../../../patches/registry.json")).unwrap();
    let before = registry
        .games
        .iter()
        .find(|g| g.scummvm_id == "sq1")
        .unwrap()
        .clone();
    merge(&mut registry, &catalog());
    let after = registry
        .games
        .iter()
        .find(|g| g.scummvm_id == "sq1")
        .unwrap();
    assert_eq!(
        serde_json::to_value(&before.launch_config).unwrap(),
        serde_json::to_value(&after.launch_config).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&before.detection_patterns).unwrap(),
        serde_json::to_value(&after.detection_patterns).unwrap()
    );
    assert_eq!(serde_json::to_value(&before.behavior).unwrap(), serde_json::to_value(&after.behavior).unwrap());
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        assert_eq!(after.patches[0].version, "RC2-kitor1");
        assert_eq!(after.patches[0].release_revision, 1);
        assert!(after.patches[0].post_install_script.is_none());
    }
}
