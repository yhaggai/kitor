//! Native macOS Owlboy support and the bundled experimental Hebrew patch.
use std::{fs, path::{Path,PathBuf}, process::Command};
pub const VERSION: &str = "0.2.0-beta.2-kitor1";
const MANIFEST: &[u8] = include_bytes!("../../../../patches-workspace/owlboy/he/patch.osx.candidate.json");
const ASSETS: &[(&str,&[u8])] = &[
    ("deltas/Data/Localizations/en/Arial/Arial.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/Arial.png.bsdiff")),
    ("deltas/Data/Localizations/en/Arial/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Arial/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Arial/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/Gulim.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/Gulim.png.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Gulim/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Gulim/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/GulimBold.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/GulimBold.png.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/GulimBold/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/GulimBold/info.txt.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/SpriteFont1.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/SpriteFont1.png.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/he/SpriteFont1/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he/SpriteFont1/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/Unlocalized.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/Unlocalized.png.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/Unlocalized/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/Unlocalized/info.txt.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/WarningFont.png.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/WarningFont.png.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/charmap.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/charmap.txt.bsdiff")),
    ("deltas/Data/Localizations/en/WarningFont/info.txt.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/en/WarningFont/info.txt.bsdiff")),
    ("deltas/Data/Localizations/he.csv.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/he.csv.bsdiff")),
    ("deltas/Data/Localizations/Translations.csv.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Data/Localizations/Translations.csv.bsdiff")),
    ("assets/Data/Localizations/he/speechBubbleConfig.ini",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/speechBubbleConfig.ini")),
    ("assets/Data/Localizations/Flags/he.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/Flags/he.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelVellie_58x37.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelVellie_58x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTitleAdvent_183x37.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTitleAdvent_183x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTitleOwlTemple_117x48.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTitleOwlTemple_117x48.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelSvevendeKontinent_232x37.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelSvevendeKontinent_232x37.png")),
    ("assets/Data/Localizations/he/Titles/sprTittelPiratmoderskipet_175x45.png",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/Titles/sprTittelPiratmoderskipet_175x45.png")),
    ("assets/Data/Localizations/he/layouts.tsv",include_bytes!("../../../../patches-workspace/owlboy/he/assets/Data/Localizations/he/layouts.tsv")),
    ("assets/OwlboyHebrew.dll",include_bytes!("../../../../patches-workspace/owlboy/he/assets/OwlboyHebrew.dll")),
    ("deltas/Owlboy.exe.bsdiff",include_bytes!("../../../../patches-workspace/owlboy/he/deltas/Owlboy.exe.bsdiff")),
 ];
fn child(path: &Path,name: &str) -> Option<PathBuf> {
 fs::read_dir(path).ok()?.flatten().find(|e|e.file_name().to_string_lossy().eq_ignore_ascii_case(name)).map(|e|e.path())
}
fn descend(path: &Path,names: &[&str]) -> Option<PathBuf> {
 let mut p=path.to_path_buf();for name in names {p=child(&p,name)?;}Some(p)
}
pub fn detect(path: &Path) -> Option<PathBuf> {
 let mut bundles=vec![path.to_path_buf()];
 if let Some(bundle)=child(path,"Owlboy.app"){bundles.push(bundle);}
 if path.file_name().is_some_and(|n|n.eq_ignore_ascii_case("Resources")) {
  if let Some(bundle)=path.parent().and_then(Path::parent){bundles.push(bundle.to_path_buf());}
 }
 for bundle in bundles {
  let Some(data)=descend(&bundle,&["Contents","Resources"]) else {continue};
  let runner=descend(&bundle,&["Contents","MacOS","Owlboy"]);
  let valid=runner.is_some_and(|p|p.is_file()) &&
   [vec!["Owlboy.exe"],vec!["FNA.dll"],vec!["Data","versioninfo.txt"],vec!["Data","Localizations","de.csv"]]
    .iter().all(|parts|descend(&data,parts).is_some_and(|p|p.is_file())) &&
   descend(&data,&["Data","Maps"]).is_some_and(|p|p.is_dir());
  if valid {return data.canonicalize().ok();}
 }None
}
pub fn launch_command(path: &Path) -> Result<Command,String> {
 let data=detect(path).ok_or("Complete native Owlboy macOS installation not found")?;
 let bundle=data.parent().and_then(Path::parent).ok_or("Invalid Owlboy bundle")?;
 let mut cmd=Command::new("open");cmd.arg(bundle).current_dir(data);Ok(cmd)
}
pub fn stage(dir: &Path,language: &str,version: &str)->Result<(),String>{
 if language!="he" || version!=VERSION{return Err("Unknown bundled Owlboy patch version".into());}
 fs::write(dir.join("patch.json"),MANIFEST).map_err(|e|e.to_string())?;
 for (name,bytes) in ASSETS {
  let target=dir.join(name);fs::create_dir_all(target.parent().unwrap()).map_err(|e|e.to_string())?;
  fs::write(target,bytes).map_err(|e|e.to_string())?;
 }Ok(())
}
#[cfg(test)]
mod tests {
 use super::*;
 pub fn fixture(root:&Path)->PathBuf {
  let bundle=root.join("oWlBoY.ApP");let data=bundle.join("Contents/Resources");
  fs::create_dir_all(data.join("Data/Localizations")).unwrap();fs::create_dir_all(data.join("Data/Maps")).unwrap();
  fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
  for name in ["Owlboy.exe","FNA.dll","Data/versioninfo.txt","Data/Localizations/de.csv"] {fs::write(data.join(name),b"fixture").unwrap();}
  fs::write(bundle.join("Contents/MacOS/Owlboy"),b"fixture").unwrap();data
 }
 #[test] fn resolves_parent_bundle_data_and_rejects_incomplete(){
  let t=tempfile::tempdir().unwrap();assert!(detect(t.path()).is_none());let data=fixture(t.path());
  for selected in [t.path(),data.parent().unwrap().parent().unwrap(),&data]{
   assert_eq!(detect(selected),Some(data.canonicalize().unwrap()));
   assert_eq!(launch_command(selected).unwrap().get_program(),"open");
  }
  fs::remove_file(data.join("FNA.dll")).unwrap();assert!(detect(t.path()).is_none());
 }
 #[test] fn bundled_payload_matches_manifest(){
  use sha2::{Digest,Sha256};let t=tempfile::tempdir().unwrap();stage(t.path(),"he",VERSION).unwrap();
  let m:serde_json::Value=serde_json::from_slice(MANIFEST).unwrap();
  for f in m["platforms"]["osx"]["files"].as_array().unwrap(){
   let bytes=fs::read(t.path().join(f["source"]["path"].as_str().unwrap())).unwrap();
   assert_eq!(format!("{:x}",Sha256::digest(bytes)),f["source"]["sha256"].as_str().unwrap());
  }assert!(stage(t.path(),"he","unknown").is_err());
 }
}
