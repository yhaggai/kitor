//! Native Windows Special Edition candidates, separate from classic ScummVM patches.
use std::{fs, path::{Path, PathBuf}, process::Command};

pub const VERSION: &str = "0.2.0-native-candidate";
pub fn supported(id: &str) -> bool { matches!(id, "monkey-se-native" | "monkey2-se-native") }

fn names(id: &str) -> Result<(&'static str, &'static str, &'static str), String> {
    match id {
        "monkey-se-native" => Ok(("MISE.exe", "Monkey1.pak", "32360")),
        "monkey2-se-native" => Ok(("Monkey2.exe", "monkey2.pak", "32460")),
        _ => Err("Unknown native Monkey Island edition".into()),
    }
}

pub fn detect(root: &Path, id: &str) -> Option<PathBuf> {
    let (exe, pak, _) = names(id).ok()?;
    if !root.join(exe).is_file() || !root.join(pak).is_file() { return None; }
    use std::io::Read;
    let mut header = [0; 8];
    fs::File::open(root.join(pak)).ok()?.read_exact(&mut header).ok()?;
    if header != *b"KAPL\0\0\x80?" { return None; }
    root.canonicalize().ok()
}

pub fn launch_command(root: &Path, id: &str) -> Result<Command, String> {
    if !cfg!(target_os = "windows") { return Err("Native Monkey Island patches currently support Windows only".into()); }
    let root = detect(root, id).ok_or("Native Monkey Island Special Edition installation not found")?;
    let (exe, _, app) = names(id)?;
    if id == "monkey2-se-native" && !root.join("steam_api.dll").is_file() {
        return Err("Monkey Island 2 is missing steam_api.dll. Restore the game through Steam before launching the native edition.".into());
    }
    let mut command = Command::new(root.join(exe));
    command.current_dir(root).env("SteamAppId", app).env("SteamGameId", app);
    Ok(command)
}

pub fn stage(destination: &Path, id: &str, language: &str, version: &str) -> Result<(), String> {
    if !cfg!(target_os = "windows") { return Err("Native Monkey Island patches currently support Windows only".into()); }
    if language != "he" || version != VERSION { return Err("Unknown native Monkey Island patch version".into()); }
    let resources: &[(&str, &[u8])] = match id {
        "monkey-se-native" => &[
            ("patch.windows.candidate.json", include_bytes!("../../../../patches-workspace/monkey-se-native/he/patch.windows.candidate.json")),
            ("recipes/0.json", include_bytes!("../../../../patches-workspace/monkey-se-native/he/recipes/0.json")),
            ("recipes/1.json", include_bytes!("../../../../patches-workspace/monkey-se-native/he/recipes/1.json")),
            ("recipes/2.json", include_bytes!("../../../../patches-workspace/monkey-se-native/he/recipes/2.json")),
            ("recipes/3.json", include_bytes!("../../../../patches-workspace/monkey-se-native/he/recipes/3.json")),
        ],
        "monkey2-se-native" => &[
            ("patch.windows.candidate.json", include_bytes!("../../../../patches-workspace/monkey2-se-native/he/patch.windows.candidate.json")),
            ("recipes/0.json", include_bytes!("../../../../patches-workspace/monkey2-se-native/he/recipes/0.json")),
        ],
        _ => return Err("Unknown native Monkey Island edition".into()),
    };
    for (name, bytes) in resources {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().ok_or("Invalid patch path")?).map_err(|e|e.to_string())?;
        fs::write(path, bytes).map_err(|e|e.to_string())?;
    }
    fs::create_dir_all(destination.join("licenses")).map_err(|e|e.to_string())?;
    fs::write(destination.join("licenses/CREDITS.txt"), include_bytes!("../../../../patches-workspace/monkey-se-native/he/licenses/CREDITS.txt")).map_err(|e|e.to_string())?;
    fs::write(destination.join("licenses/FONT-OFL.txt"), include_bytes!("../../../../patches-workspace/grim-fandango-remastered/he/licenses/FONT-OFL.txt")).map_err(|e|e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detection_requires_title_specific_executable_and_archive_header() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("MISE.exe"), b"fixture").unwrap();
        fs::write(d.path().join("Monkey1.pak"), b"not a pak").unwrap();
        assert!(detect(d.path(), "monkey-se-native").is_none());
        fs::write(d.path().join("Monkey1.pak"), b"KAPL\0\0\x80?").unwrap();
        assert!(detect(d.path(), "monkey-se-native").is_some());
        assert!(detect(d.path(), "monkey2-se-native").is_none());
        assert!(!supported("monkey-se"));
    }
    #[test]
    fn staging_rejects_wrong_identity_and_version() {
        let d = tempfile::tempdir().unwrap();
        assert!(stage(d.path(), "monkey-se-native", "en", VERSION).is_err());
        assert!(stage(d.path(), "monkey-se-native", "he", "unknown").is_err());
        assert!(stage(d.path(), "monkey-se", "he", VERSION).is_err());
    }
}
