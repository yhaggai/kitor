use super::*;

#[test]
fn frontend_capabilities_match_the_import_recipes() {
    let products = products();
    let os = std::env::consts::OS;
    for product in &products {
        for recipe in &product.games {
            let caps = capabilities(product.app_id, &recipe.id);
            let expected = products.iter().find(|candidate| {
                let platforms = recipe_platforms(candidate);
                platforms.contains(&os)
                    && candidate.app_id == product.app_id
                    && candidate.games.iter().any(|game| game.id == recipe.id)
            });
            assert_eq!(caps.supported, expected.is_some(), "{}", recipe.id);
            if let Some(expected) = expected {
                assert_eq!(caps.title, expected.title);
                assert_eq!(
                    caps.games.iter().map(|g| &g.id).collect::<Vec<_>>(),
                    expected.games.iter().map(|g| &g.id).collect::<Vec<_>>()
                );
            }
        }
        assert!(!capabilities(product.app_id, "not-in-this-product").supported);
    }
    assert!(!capabilities(u32::MAX, "unknown").supported);
}

fn recipe_platforms(product: &SteamProduct) -> &'static [&'static str] {
    match product.platform.as_deref() {
        None => &["macos"],
        Some("windows") => &["windows"],
        Some("desktop") => &["macos", "windows"],
        Some(platform) => panic!("Unknown Steam recipe platform: {platform}"),
    }
}

#[test]
fn all_product_paths_are_contained_and_layouts_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let mut ids = std::collections::HashSet::new();
    for product in products() {
        let platforms = recipe_platforms(&product);
        assert!(!product.games.is_empty());
        crate::services::patch_lifecycle::safe_path(root.path(), &product.cache_key).unwrap();
        for name in product.required_files.iter().chain(&product.copy_paths) {
            crate::services::patch_lifecycle::safe_path(root.path(), name).unwrap();
        }
        if matches!(product.importer, Importer::Layout) {
            assert_eq!(product.games.len(), 1);
            assert!(!product.required_files.is_empty());
            assert_ne!(product.copy_depot, !product.copy_paths.is_empty());
        }
        #[cfg(unix)]
        for path in &product.executable_paths {
            crate::services::patch_lifecycle::safe_path(root.path(), path).unwrap();
        }
        for game in product.games {
            for platform in platforms {
                assert!(
                    ids.insert((product.app_id, *platform, game.id.clone())),
                    "Overlapping Steam recipes for {} on {platform}",
                    game.id
                );
            }
            crate::services::patch_lifecycle::safe_path(root.path(), &game.id).unwrap();
            if !game.folder.is_empty() {
                crate::services::patch_lifecycle::safe_path(root.path(), &game.folder).unwrap();
            }
            for file in game.files {
                crate::services::patch_lifecycle::safe_path(root.path(), &file.name).unwrap();
            }
        }
    }
}
