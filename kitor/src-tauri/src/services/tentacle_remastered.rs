//! Native Remastered prototype; independent from classic/ScummVM support.
use std::{fs, path::Path, process::Command};
pub const VERSION: &str = "0.1-local-items-kitor1";
pub fn stage(destination: &Path, language: &str, version: &str) -> Result<(), String> {
    if language != "he" || version != VERSION {
        return Err("Unknown Remastered patch version".into());
    }
    for (name,bytes) in [
  ("patch.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/patch.osx.candidate.json").as_slice()),
  ("recipes/tenta.cle.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/tenta.cle.json").as_slice()),
  ("recipes/signature-0.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-0.json").as_slice()),
  ("recipes/signature-1.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-1.json").as_slice()),
  ("recipes/signature-2.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-2.json").as_slice()),
  ("recipes/signature-3.json",include_bytes!("../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-3.json").as_slice()),
 ] {let p=destination.join(name);fs::create_dir_all(p.parent().unwrap()).map_err(|e|e.to_string())?;fs::write(p,bytes).map_err(|e|e.to_string())?;}
    Ok(())
}
pub fn launch_command(root: &Path) -> Result<Command, String> {
    let root = super::tentacle::detect_remastered(root)
        .ok_or("Native Day of the Tentacle Remastered installation not found")?;
    let binary = if cfg!(target_os = "macos") {
        root.join("Dott.app/Contents/MacOS/Dott")
    } else if cfg!(target_os = "windows") {
        root.join("Dott.exe")
    } else {
        return Err("Remastered launch supports macOS and Windows".into());
    };
    let mut cmd = Command::new(binary);
    cmd.current_dir(root).env("SteamAppId", "388210");
    Ok(cmd)
}
