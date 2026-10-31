//! Native Windows/macOS FNA Owlboy support and the bundled Hebrew beta.
use std::{fs, path::{Path,PathBuf}, process::Command};
pub const VERSION: &str = "0.2.0-beta.2-kitor2";
const MANIFEST: &[u8] = include_bytes!("../../../../patches-workspace/owlboy/he/patch.osx.candidate.json");
const WINDOWS_MANIFEST: &[u8] = include_bytes!("../../../../patches-workspace/owlboy/he/patch.windows.candidate.json");
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
mod windows_assets { include!("../../../../patches-workspace/owlboy/he/windows/embedded.rs"); }
fn child(path: &Path,name: &str) -> Option<PathBuf> {
 fs::read_dir(path).ok()?.flatten().find(|e|e.file_name().to_string_lossy().eq_ignore_ascii_case(name)).map(|e|e.path())
}
fn descend(path: &Path,names: &[&str]) -> Option<PathBuf> {
 let mut p=path.to_path_buf();for name in names {p=child(&p,name)?;}Some(p)
}
pub fn detect(path: &Path) -> Option<PathBuf> {
 // A macOS Resources directory also contains Owlboy.exe/FNA.dll. Resolve its
 // bundle first so it cannot be mistaken for a Windows installation.
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
 }
 if windows_root(path) {return path.canonicalize().ok();}
 if path.file_name().is_some_and(|n|n.eq_ignore_ascii_case("Data")) {
  if let Some(parent)=path.parent(){if windows_root(parent){return parent.canonicalize().ok();}}
 }
 None
}
fn windows_root(path:&Path)->bool {
 [vec!["Owlboy.exe"],vec!["FNA.dll"],vec!["FNA3D.dll"],vec!["SDL2.dll"],vec!["Data","versioninfo.txt"],vec!["Data","Localizations","de.csv"]]
  .iter().all(|parts|descend(path,parts).is_some_and(|p|p.is_file())) &&
  descend(path,&["Data","Maps"]).is_some_and(|p|p.is_dir())
}
pub fn launch_command(path: &Path) -> Result<Command,String> {
 let data=detect(path).ok_or("Complete native Owlboy FNA installation not found")?;
 if windows_root(&data) {
  if !cfg!(target_os="windows"){return Err("The Windows Owlboy edition requires Windows".into());}
  let mut cmd=Command::new(child(&data,"Owlboy.exe").ok_or("Owlboy.exe not found")?);
  cmd.current_dir(data);return Ok(cmd);
 }
 if !cfg!(target_os="macos"){return Err("The macOS Owlboy edition requires macOS".into());}
 let bundle=data.parent().and_then(Path::parent).ok_or("Invalid Owlboy bundle")?;
 let mut cmd=Command::new("open");cmd.arg(bundle).current_dir(data);Ok(cmd)
}
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
   if cfg!(target_os="macos"){assert_eq!(launch_command(selected).unwrap().get_program(),"open");}
   else {assert!(launch_command(selected).is_err());}
  }
  fs::remove_file(data.join("FNA.dll")).unwrap();assert!(detect(t.path()).is_none());
 }
 #[test] fn bundled_payload_matches_manifest(){
  use sha2::{Digest,Sha256};let t=tempfile::tempdir().unwrap();stage(t.path(),"he",VERSION).unwrap();
  for (manifest,platform) in [(MANIFEST,"osx"),(WINDOWS_MANIFEST,"windows")] {
  let m:serde_json::Value=serde_json::from_slice(manifest).unwrap();
  for f in m["platforms"][platform]["files"].as_array().unwrap(){
   let bytes=fs::read(t.path().join(f["source"]["path"].as_str().unwrap())).unwrap();
   assert_eq!(format!("{:x}",Sha256::digest(bytes)),f["source"]["sha256"].as_str().unwrap());
  }}assert!(stage(t.path(),"he","unknown").is_err());
 }
 #[test] fn catalog_embeds_platform_specific_payloads(){
  use sha2::{Digest,Sha256};
  let catalog:serde_json::Value=serde_json::from_str(include_str!("../../../patches/bundled-assets.json")).unwrap();
  for (manifest,platform) in [(MANIFEST,"osx"),(WINDOWS_MANIFEST,"windows")] {
   let assets=&catalog[format!("owlboy/{platform}")];
   let workspace=Path::new(file!()).canonicalize().unwrap().ancestors().nth(5).unwrap().to_path_buf();
   assert_eq!(fs::read(workspace.join(assets["patch.json"].as_str().unwrap())).unwrap(),manifest);
   let m:serde_json::Value=serde_json::from_slice(manifest).unwrap();
   for f in m["platforms"][platform]["files"].as_array().unwrap(){
    let path=assets[f["source"]["path"].as_str().unwrap()].as_str().unwrap();
    let bytes=fs::read(workspace.join(path)).unwrap();
    assert_eq!(format!("{:x}",Sha256::digest(bytes)),f["source"]["sha256"].as_str().unwrap());
   }
  }
 }
 #[test] fn windows_detection_and_launch(){
  let t=tempfile::tempdir().unwrap();let root=t.path().join("Owlboy");
  fs::create_dir_all(root.join("Data/Localizations")).unwrap();fs::create_dir_all(root.join("Data/Maps")).unwrap();
  for name in ["oWlBoY.ExE","FNA.dll","FNA3D.dll","SDL2.dll","Data/versioninfo.txt","Data/Localizations/de.csv"]{fs::write(root.join(name),b"fixture").unwrap();}
  assert_eq!(detect(&root),Some(root.canonicalize().unwrap()));
  assert_eq!(detect(&root.join("Data")),detect(&root));
  if cfg!(target_os="windows") {
   let cmd=launch_command(&root).unwrap();assert_eq!(Path::new(cmd.get_program()),root.canonicalize().unwrap().join("oWlBoY.ExE"));
   assert_eq!(cmd.get_current_dir(),Some(root.canonicalize().unwrap().as_path()));
   // Exercise the catalog route as well: it must not reject Windows before
   // reaching this platform-aware launcher (for either language edition).
   let behavior=super::super::game_behavior::for_game("owlboy").unwrap();
   let profiles=behavior.launch.unwrap();
   for profile in [profiles.original,profiles.patched] {
    let command=super::super::game_adapters::native_command(&profile,&root,"owlboy").unwrap().unwrap();
    assert_eq!(command.get_program(),cmd.get_program());
    assert_eq!(command.get_current_dir(),cmd.get_current_dir());
   }
  }else{assert!(launch_command(&root).is_err());}
  fs::remove_file(root.join("FNA.dll")).unwrap();assert!(detect(&root).is_none());
 }
 #[test]
 #[ignore = "requires KITOR_OWLBOY_PATH pointing to pristine Windows FNA game files"]
 fn windows_owned_files_roundtrip(){
  use super::super::patch_lifecycle as lifecycle;
  let original=PathBuf::from(std::env::var_os("KITOR_OWLBOY_PATH").unwrap());
  let t=tempfile::tempdir().unwrap();let game=t.path().join("game");let package=t.path().join("package");
  fs::create_dir_all(&game).unwrap();fs::create_dir_all(&package).unwrap();stage(&package,"he",VERSION).unwrap();
  // Exercise the Windows plan explicitly, including when run on another host.
  fs::write(package.join("patch.json"),WINDOWS_MANIFEST).unwrap();
  let m:serde_json::Value=serde_json::from_slice(WINDOWS_MANIFEST).unwrap();
  let files=m["platforms"]["windows"]["files"].as_array().unwrap();
  for f in files {
   if let Some(input)=f["source"]["input"].as_str(){
    let dest=game.join(input);fs::create_dir_all(dest.parent().unwrap()).unwrap();fs::copy(original.join(input),dest).unwrap();
   }
  }
  let sentinel=game.join("user-save.sav");fs::write(&sentinel,b"keep save").unwrap();
  let recovery=t.path().join("recovery");
  let journal=lifecycle::install(&package,"patch.json",&game,&recovery,"owlboy","he",VERSION,"windows").unwrap();
  for f in files {assert_eq!(lifecycle::hash(&game.join(f["path"].as_str().unwrap())).unwrap(),f["installed_sha256"].as_str().unwrap());}
  lifecycle::uninstall(&journal,&game).unwrap();
  for f in files {
   let path=game.join(f["path"].as_str().unwrap());
   if let Some(hash)=f["original_sha256"].as_str(){assert_eq!(lifecycle::hash(&path).unwrap(),hash);}
   else{assert!(!path.exists());}
  }
  assert_eq!(fs::read(&sentinel).unwrap(),b"keep save");
  // An unsupported executable must be refused before any translation is installed.
  fs::write(game.join("Owlboy.exe"),b"unsupported edition").unwrap();
  assert!(lifecycle::install(&package,"patch.json",&game,&t.path().join("rejected"),"owlboy","he",VERSION,"windows").is_err());
  assert!(!game.join("Data/Localizations/he.csv").exists());
  assert_eq!(fs::read(game.join("Owlboy.exe")).unwrap(),b"unsupported edition");
 }
}
