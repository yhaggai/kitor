//! PQ1 EGA: publisher fingerprints guard both supported editions and Hebrew outputs.
use std::{fs, path::{Path, PathBuf}};
use sha2::Digest;
use super::patch_lifecycle::{Package, Source};
pub const VERSION: &str = "1.0-kitor2";
const PLANS: &[&[u8]] = &[
 include_bytes!("../../../../patches-workspace/pq1/he/patch.json"),
 include_bytes!("../../../../patches-workspace/pq1/he/patch.steam.json"),
];
const ASSETS: &[(&str, &[u8])] = &[
 ("payload/LOGDIR.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/LOGDIR.bsdiff")),
 ("payload/LOGDIR.steam.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/LOGDIR.steam.bsdiff")),
 ("payload/OBJECT.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/OBJECT.bsdiff")),
 ("payload/PICDIR.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/PICDIR.bsdiff")),
 ("payload/PQ1.WAG", include_bytes!("../../../../patches-workspace/pq1/he/payload/PQ1.WAG")),
 ("payload/SNDDIR.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/SNDDIR.bsdiff")),
 ("payload/VIEWDIR.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/VIEWDIR.bsdiff")),
 ("payload/VOL.0.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/VOL.0.bsdiff")),
 ("payload/VOL.0.steam.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/VOL.0.steam.bsdiff")),
 ("payload/WORDS.TOK.EXTENDED", include_bytes!("../../../../patches-workspace/pq1/he/payload/WORDS.TOK.EXTENDED")),
 ("payload/WORDS.TOK.bsdiff", include_bytes!("../../../../patches-workspace/pq1/he/payload/WORDS.TOK.bsdiff")),
 ("payload/agi-font-dos.bin", include_bytes!("../../../../patches-workspace/pq1/he/payload/agi-font-dos.bin")),
 ];
fn child(root: &Path, name: &str) -> Option<PathBuf> {
 fs::read_dir(root).ok()?.flatten().find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name) && !e.file_type().is_ok_and(|t|t.is_symlink())).map(|e|e.path())
}
fn descend(root: &Path, parts: &[&str]) -> Option<PathBuf> {
 let mut p=root.to_path_buf(); for part in parts {p=child(&p,part)?;} Some(p)
}
fn matches(root: &Path, package: &Package, installed: bool) -> bool {
 package.files.iter().filter(|c| c.original_md5.is_some()).all(|c| {
  let Some(file)=child(root,&c.path) else {return false};
  let Ok(data)=fs::read(file) else {return false};
  let expected=if installed {&c.installed_md5} else {&c.original_md5};
  expected.as_ref().is_some_and(|h|format!("{:x}",md5::Md5::digest(data))==*h)
 })
}
fn engine_identifies(root: &Path) -> Option<bool> {
 let home=std::env::var_os("HOME").map(PathBuf::from);
 let executable=if cfg!(target_os="macos") {
  home?.join("Library/Application Support/haggai/scummvm/ScummVM.app/Contents/MacOS/scummvm")
 } else if cfg!(target_os="windows") {
  PathBuf::from(std::env::var_os("APPDATA")?).join("haggai/scummvm/scummvm.exe")
 } else {return None};
 if !executable.is_file() {return None;}
 let output=std::process::Command::new(executable).arg("--detect").arg(format!("--path={}",root.display())).output().ok()?;
 if !output.status.success() {return None;}
 Some(String::from_utf8_lossy(&output.stdout).lines().any(|l|l.split_whitespace().next()==Some("agi:pq1")))
}
pub fn detect(root: &Path) -> Option<PathBuf> {
 // Use ScummVM's detection-only CLI when installed; exact publisher fingerprints
 // provide the offline fallback and independently restrict supported patch editions.
 for parts in [vec![],vec!["PQ1","EGA"],vec!["EGA"],vec!["Contents","Resources","game","game"],vec!["Contents","Resources","game"]] {
  let Some(path)=descend(root,&parts) else {continue};
  if child(&path,"LOGDIR").is_none() {continue;}
  if engine_identifies(&path)==Some(false) {continue;}
  for bytes in PLANS {
   let package: Package=serde_json::from_slice(bytes).ok()?;
   if matches(&path,&package,false)||matches(&path,&package,true) {return path.canonicalize().ok();}
  }
 } None
}
pub fn stage(dir: &Path, language: &str, version: &str, root: &Path) -> Result<(),String> {
 if language!="he" || version!=VERSION {return Err("Unknown PQ1 patch version".into());}
 let bytes=PLANS.iter().find(|bytes| {
  let p:Package=serde_json::from_slice(bytes).unwrap(); matches(root,&p,false)||matches(root,&p,true)
 }).ok_or("Unsupported PQ1 EGA edition; original files were not changed")?;
 let mut package:Package=serde_json::from_slice(bytes).map_err(|e|e.to_string())?;
 // Retain the actual on-disk spelling on case-sensitive filesystems.
 for change in &mut package.files {
  if let Some(p)=child(root,&change.path) {change.path=p.file_name().unwrap().to_string_lossy().into_owned();}
  if let Some(Source::Bsdiff {input,..})=&mut change.source {
   if let Some(p)=child(root,input) {*input=p.file_name().unwrap().to_string_lossy().into_owned();}
  }
 }
 fs::create_dir_all(dir.join("payload")).map_err(|e|e.to_string())?;
 fs::write(dir.join("patch.json"),serde_json::to_vec_pretty(&package).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
 for (name,data) in ASSETS {fs::write(dir.join(name),data).map_err(|e|e.to_string())?;}
 Ok(())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn rejects_lookalikes_and_unknown_versions() {
  let t=tempfile::tempdir().unwrap();fs::create_dir_all(t.path().join("PQ1/EGA")).unwrap();
  fs::write(t.path().join("PQ1/EGA/LOGDIR"),b"other game").unwrap();
  assert!(detect(t.path()).is_none());assert!(stage(t.path(),"he",VERSION,t.path()).is_err());
  assert!(stage(t.path(),"he","wrong",t.path()).is_err());
 }
 #[test] fn owned_steam_layout_roundtrip() {
  let Ok(source)=std::env::var("KITOR_PQ1_TEST_DATA") else {return};
  let source=Path::new(&source);let root=detect(source).expect("owned EGA edition");
  let temp=tempfile::tempdir().unwrap();let game=temp.path().join("pq1/ega");fs::create_dir_all(&game).unwrap();
  for entry in fs::read_dir(&root).unwrap().flatten().filter(|e|e.path().is_file()) {fs::copy(entry.path(),game.join(entry.file_name().to_string_lossy().to_lowercase())).unwrap();}
  assert_eq!(detect(temp.path()).unwrap(),game.canonicalize().unwrap());
  let before:Vec<_>=fs::read_dir(&game).unwrap().flatten().map(|e|(e.path(),fs::read(e.path()).unwrap())).collect();
  let package=temp.path().join("package");stage(&package,"he",VERSION,&game).unwrap();
  let journal=super::super::patch_lifecycle::install(&package,"patch.json",&game,&temp.path().join("recovery"),"pq1","he",VERSION,super::super::patch_lifecycle::platform()).unwrap();
  assert_eq!(detect(temp.path()).unwrap(),game.canonicalize().unwrap());
  fs::write(game.join("later.sav"),b"save").unwrap();
  super::super::patch_lifecycle::uninstall(&journal,&game).unwrap();
  for (p,data) in before {assert_eq!(fs::read(p).unwrap(),data);}
  assert_eq!(fs::read(game.join("later.sav")).unwrap(),b"save");
  assert!(!game.join("WORDS.TOK.EXTENDED").exists());
 }
}
