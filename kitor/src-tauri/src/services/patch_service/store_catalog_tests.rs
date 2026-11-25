use super::store_catalog_entry;

#[test]
fn store_only_game_preserves_gog_purchase_details() {
    let game = serde_json::from_value(serde_json::json!({
        "scummvm_id": "dig",
        "title": "The Dig",
        "add_to_cart_url": "https://www.gog.com/checkout/widget/52756712356612660/1425299904?price=undefined%2018.60",
        "gog": {
            "id": 1425299904,
            "installers": [{
                "id": "",
                "name": "The Dig",
                "path": "https://www.gog.com/downloads/the_dig/en1installer0"
            }]
        },
        "detection_patterns": { "unique_files": [], "file_patterns": [] },
        "patches": []
    }))
    .unwrap();
    let listing = store_catalog_entry(&game);
    assert_eq!(listing.language, "");
    assert_eq!(listing.download_url, "");
    assert_eq!(listing.source_type, crate::services::registry_service::CatalogSourceType::Store);
    assert_eq!(
        listing.gog_installer_path.as_deref(),
        Some("https://www.gog.com/downloads/the_dig/en1installer0")
    );
    assert_eq!(
        listing.add_to_cart_url.as_deref(),
        Some("https://www.gog.com/checkout/widget/52756712356612660/1425299904?price=undefined%2018.60")
    );
}
