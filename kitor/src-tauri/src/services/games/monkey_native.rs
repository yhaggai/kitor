//! Native Windows Special Edition candidates, separate from classic ScummVM patches.
#[cfg(any(test, feature = "standalone-patch-tools"))]
use std::{fs, path::Path};

#[cfg(any(test, feature = "standalone-patch-tools"))]
pub const VERSION: &str = "0.2.0-native-candidate";
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn supported(id: &str) -> bool { matches!(id, "monkey-se-native" | "monkey2-se-native") }

#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn stage(destination: &Path, id: &str, language: &str, version: &str) -> Result<(), String> {
    if !cfg!(target_os = "windows") { return Err("Native Monkey Island patches currently support Windows only".into()); }
    if language != "he" || version != VERSION { return Err("Unknown native Monkey Island patch version".into()); }
    let resources: &[(&str, &[u8])] = match id {
        "monkey-se-native" => &[
            ("patch.json", include_bytes!("../../../../../patches-workspace/monkey-se-native/he/patch.windows.candidate.json")),
            ("recipes/0.json", include_bytes!("../../../../../patches-workspace/monkey-se-native/he/recipes/0.json")),
            ("recipes/1.json", include_bytes!("../../../../../patches-workspace/monkey-se-native/he/recipes/1.json")),
            ("recipes/2.json", include_bytes!("../../../../../patches-workspace/monkey-se-native/he/recipes/2.json")),
            ("recipes/3.json", include_bytes!("../../../../../patches-workspace/monkey-se-native/he/recipes/3.json")),
        ],
        "monkey2-se-native" => &[
            ("patch.json", include_bytes!("../../../../../patches-workspace/monkey2-se-native/he/patch.windows.candidate.json")),
            ("recipes/0.json", include_bytes!("../../../../../patches-workspace/monkey2-se-native/he/recipes/0.json")),
        ],
        _ => return Err("Unknown native Monkey Island edition".into()),
    };
    for (name, bytes) in resources {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().ok_or("Invalid patch path")?).map_err(|e|e.to_string())?;
        fs::write(path, bytes).map_err(|e|e.to_string())?;
    }
    fs::create_dir_all(destination.join("licenses")).map_err(|e|e.to_string())?;
    fs::write(destination.join("licenses/CREDITS.txt"), include_bytes!("../../../../../patches-workspace/monkey-se-native/he/licenses/CREDITS.txt")).map_err(|e|e.to_string())?;
    fs::write(destination.join("licenses/FONT-OFL.txt"), include_bytes!("../../../../../patches-workspace/grim-fandango-remastered/he/licenses/FONT-OFL.txt")).map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
#[path = "monkey_native/tests.rs"]
mod tests;
