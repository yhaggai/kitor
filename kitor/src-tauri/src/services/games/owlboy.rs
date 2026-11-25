//! Native Windows/macOS FNA Owlboy support and the bundled Hebrew beta.
#[cfg(any(test, feature = "standalone-patch-tools"))]
use std::{fs, path::Path};
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub const VERSION: &str = "0.2.0-beta.2-kitor2";
#[cfg(any(test, feature = "standalone-patch-tools"))]
const MANIFEST: &[u8] = include_bytes!("../../../../../patches-workspace/owlboy/he/patch.osx.candidate.json");
#[cfg(any(test, feature = "standalone-patch-tools"))]
const WINDOWS_MANIFEST: &[u8] = include_bytes!("../../../../../patches-workspace/owlboy/he/patch.windows.candidate.json");
#[cfg(any(test, feature = "standalone-patch-tools"))]
const ASSETS: &[(&str,&[u8])] = &[
    ("deltas/Data/Localizations/en/Arial/Arial.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/Arial.png.bsdiff")),
    ("deltas/Data/Localizations/en/Arial/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Arial/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/Gulim.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/Gulim.png.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/GulimBold.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/GulimBold.png.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/info.txt.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/SpriteFont1.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/SpriteFont1.png.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/Unlocalized.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/Unlocalized.png.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/WarningFont.png.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/WarningFont.png.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/charmap.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/info.txt.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/info.txt.bsdiff")),
    ("deltas/Data/Localizations/he.csv.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he.csv.bsdiff")),
    ("deltas/Data/Localizations/Translations.csv.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/Translations.csv.bsdiff")),
    ("assets/Data/Localizations/he/speechBubbleConfig.ini",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/speechBubbleConfig.ini")),
    ("assets/Data/Localizations/Flags/he.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/Flags/he.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelVellie_58x37.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelVellie_58x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTitleAdvent_183x37.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTitleAdvent_183x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTitleOwlTemple_117x48.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTitleOwlTemple_117x48.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelSvevendeKontinent_232x37.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelSvevendeKontinent_232x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelPiratmoderskipet_175x45.png",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelPiratmoderskipet_175x45.png")),
    ("assets/Data/Localizations/he/layouts.tsv",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/layouts.tsv")),
    ("assets/OwlboyHebrew.dll",include_bytes!("../../../../../patches-workspace/owlboy/he/assets/OwlboyHebrew.dll")),
    ("deltas/Owlboy.exe.bsdiff",include_bytes!("../../../../../patches-workspace/owlboy/he/deltas/Owlboy.exe.bsdiff")),
 ];
#[cfg(any(test, feature = "standalone-patch-tools"))]
mod windows_assets { include!("../../../../../patches-workspace/owlboy/he/windows/embedded.rs"); }
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn stage(dir: &Path,language: &str,version: &str)->Result<(),String>{
 if language!="he" || version!=VERSION{return Err("Unknown bundled Owlboy patch version".into());}
 let manifest=if cfg!(target_os="windows"){WINDOWS_MANIFEST}else{MANIFEST};
 fs::write(dir.join("patch.json"),manifest).map_err(|e|e.to_string())?;
 for (name,bytes) in ASSETS {
  let target=dir.join(name);fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;
  fs::write(target,bytes).map_err(|e|e.to_string())?;
 }
 for (name,bytes) in windows_assets::ASSETS {
  let target=dir.join(name);fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;
  fs::write(target,bytes).map_err(|e|e.to_string())?;
 }Ok(())
}
#[cfg(test)]
#[path = "owlboy/tests.rs"]
mod tests;
