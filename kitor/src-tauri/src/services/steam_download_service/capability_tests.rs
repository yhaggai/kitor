use super::*;

#[test]
fn frontend_capabilities_match_the_import_recipes() {
    for product in products() {
        for recipe in &product.games {
            let caps = capabilities(product.app_id, &recipe.id);
            assert_eq!(caps.supported, cfg!(target_os = "macos"));
            assert_eq!(caps.title, product.title);
            assert_eq!(
                caps.games.iter().map(|g| &g.id).collect::<Vec<_>>(),
                product.games.iter().map(|g| &g.id).collect::<Vec<_>>()
            );
        }
        assert!(!capabilities(product.app_id, "not-in-this-product").supported);
    }
    assert!(!capabilities(u32::MAX, "unknown").supported);
}

#[test]
fn all_product_paths_are_contained_and_layouts_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    let mut ids = std::collections::HashSet::new();
    for product in products() {
        assert!(ids.insert(product.app_id));
        assert!(!product.games.is_empty());
        crate::services::patch_lifecycle::safe_path(root.path(), &product.cache_key).unwrap();
        for name in product.required_files.iter().chain(&product.copy_paths) {
            crate::services::patch_lifecycle::safe_path(root.path(), name).unwrap();
        }
        if matches!(product.importer, Importer::Layout) {
            assert_eq!(product.games.len(), 1);
            assert!(!product.required_files.is_empty());
            assert!(!product.copy_paths.is_empty());
        }
        for game in product.games {
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
