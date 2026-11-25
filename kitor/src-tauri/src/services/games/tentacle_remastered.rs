//! Native Remastered prototype; independent from classic/ScummVM support.
#[cfg(feature = "standalone-patch-tools")]
use std::fs;
#[cfg(feature = "standalone-patch-tools")]
use std::path::Path;
#[cfg(feature = "standalone-patch-tools")]
pub const VERSION: &str = "0.1.1-ofl-kitor1";
#[cfg(feature = "standalone-patch-tools")]
pub fn stage(destination: &Path, language: &str, version: &str) -> Result<(), String> {
    if language != "he" || version != VERSION {
        return Err("Unknown Remastered patch version".into());
    }
    for (name,bytes) in [
  ("patch.json",if cfg!(target_os = "windows") {
      include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/patch.windows.candidate.json").as_slice()
  } else {
      include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/patch.osx.candidate.json").as_slice()
  }),
  ("recipes/windows-tenta.cle.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/windows-tenta.cle.json").as_slice()),
  ("recipes/tenta.cle.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/tenta.cle.json").as_slice()),
  ("recipes/signature-0.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-0.json").as_slice()),
  ("recipes/signature-1.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-1.json").as_slice()),
  ("recipes/signature-2.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-2.json").as_slice()),
  ("recipes/signature-3.json",include_bytes!("../../../../../patches-workspace/day-of-the-tentacle-remastered/he/recipes/signature-3.json").as_slice()),
 ] {let p=destination.join(name);fs::create_dir_all(p.parent().unwrap()).map_err(|e|e.to_string())?;fs::write(p,bytes).map_err(|e|e.to_string())?;}
    Ok(())
}
