use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let index = manifest.join("../patches/bundled-assets.json");
    let registry = manifest.join("../patches/registry.json");
    println!("cargo:rerun-if-changed={}", index.display());
    println!("cargo:rerun-if-changed={}", registry.display());
    let packages: serde_json::Value = serde_json::from_slice(&fs::read(index).unwrap()).unwrap();
    assert!(packages.as_object().is_some_and(|entries| entries.is_empty()),
        "Kitor must not embed patch payloads; publish a downloadable package instead");
    assert!(env::var_os("CARGO_FEATURE_STANDALONE_PATCH_TOOLS").is_none(),
        "Build standalone patch tools through lifecycle-harness, not the desktop app");
    let catalog: serde_json::Value = serde_json::from_slice(&fs::read(registry).unwrap()).unwrap();
    for game in catalog["games"].as_array().unwrap() {
        assert!(game["behavior"]["bundled_resource"].is_null() && game["behavior"]["bundled"].is_null(),
            "Catalog contains bundled patch behavior: {}", game["scummvm_id"]);
        for patch in game["patches"].as_array().unwrap() {
            assert!(patch["source_type"] != "bundled" && !patch["download_url"].as_str().unwrap_or("").starts_with("bundled:"),
                "Catalog contains a bundled patch: {}", game["scummvm_id"]);
        }
    }
    tauri_build::build()
}
