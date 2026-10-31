use kitor_patch_lifecycle_harness::{owlboy, patch_lifecycle as lifecycle};
use std::path::PathBuf;

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: owlboy_patch install GAME_FOLDER RECOVERY_FOLDER | uninstall GAME_FOLDER JOURNAL_FOLDER".into());
    }
    let root = PathBuf::from(&args[1]);
    let storage = PathBuf::from(&args[2]);
    if args[0] == "uninstall" {
        return lifecycle::uninstall(&storage, &root);
    }
    if args[0] != "install" { return Err("Expected install or uninstall".into()); }
    let root = owlboy::detect(&root).ok_or("Owlboy FNA installation not found")?;
    let stage = tempfile::tempdir().map_err(|e| e.to_string())?;
    owlboy::stage(stage.path(), "he", owlboy::VERSION)?;
    let journal = lifecycle::install(stage.path(), "patch.json", &root, &storage,
        "owlboy", "he", owlboy::VERSION, lifecycle::platform())?;
    println!("{}", journal.display());
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); }
}
