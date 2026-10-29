use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    // Only explicitly reviewed assets are embedded; never sweep game folders.
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo = manifest.parent().unwrap().parent().unwrap();
    let index = manifest.join("../patches/bundled-assets.json");
    println!("cargo:rerun-if-changed={}", index.display());
    let packages: BTreeMap<String, BTreeMap<String, String>> =
        serde_json::from_slice(&fs::read(index).unwrap()).unwrap();
    let mut generated = String::from("pub fn embedded_assets(key: &str) -> Option<&'static [(&'static str, &'static [u8])]> { match key {\n");
    for (key, assets) in packages {
        generated.push_str(&format!(
            "{key:?} => {{ const ASSETS: &[(&str, &[u8])] = &[\n"
        ));
        for (name, source) in assets {
            let source = repo
                .join(source)
                .canonicalize()
                .expect("Missing bundled patch asset");
            assert!(
                source.starts_with(repo.canonicalize().unwrap()),
                "Bundled asset escapes repository"
            );
            println!("cargo:rerun-if-changed={}", source.display());
            generated.push_str(&format!(
                "({name:?}, include_bytes!({:?}).as_slice()),\n",
                source.to_str().unwrap()
            ));
        }
        generated.push_str("]; Some(ASSETS) },\n");
    }
    generated.push_str("_ => None, } }\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("bundled_assets.rs"),
        generated,
    )
    .unwrap();
    tauri_build::build()
}
