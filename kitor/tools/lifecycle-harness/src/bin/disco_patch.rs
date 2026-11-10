//! Local preview entry point; uses the same transaction engine as Kitor.
use kitor_patch_lifecycle_harness::{games::disco_elysium, patch_lifecycle as lifecycle};
use std::path::PathBuf;
fn run() -> Result<(),String> {
    let args:Vec<_>=std::env::args_os().skip(1).collect();
    if args.len()==2 && args[0]=="detect" {
        let root=disco_elysium::detect(&PathBuf::from(&args[1])).ok_or("Complete Final Cut installation not found")?;
        println!("{}",root.display());return Ok(());
    }
    if args.len()==3 && args[0]=="uninstall" {
        lifecycle::uninstall(&PathBuf::from(&args[2]),&PathBuf::from(&args[1]))?;
        println!("Restored original files; saves and unrelated files preserved.");return Ok(());
    }
    if args.len()!=4 || args[0]!="install" {return Err("Usage: disco_patch detect GAME\n       disco_patch install PACKAGE_FOLDER GAME RECOVERY_FOLDER\n       disco_patch uninstall GAME JOURNAL_FOLDER".into());}
    let package=PathBuf::from(&args[1]);let root=disco_elysium::detect(&PathBuf::from(&args[2])).ok_or("Complete Final Cut installation not found")?;
    let manifest:lifecycle::Package=serde_json::from_slice(&std::fs::read(package.join("patch.json")).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    if manifest.scummvm_id!="disco-elysium" || manifest.language!="he" {return Err("Wrong game/language package".into());}
    let journal=lifecycle::install(&package,"patch.json",&root,&PathBuf::from(&args[3]),"disco-elysium","he",&manifest.version,lifecycle::platform())?;
    println!("Installed preview. Select Arabic in-game. Recovery journal: {}",journal.display());Ok(())
}
fn main(){if let Err(e)=run(){eprintln!("{e}");std::process::exit(1);}}
