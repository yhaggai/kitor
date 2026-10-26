use kitor_patch_lifecycle_harness::{
    patch_lifecycle as lifecycle, tentacle, tentacle_remastered as patch,
};
use std::path::PathBuf;
fn run() -> Result<(), String> {
    let a: Vec<_> = std::env::args_os().skip(1).collect();
    if a.len() != 3 {
        return Err("Usage: tentacle_remastered_patch install GAME_FOLDER RECOVERY_FOLDER | uninstall GAME_FOLDER JOURNAL_FOLDER".into());
    }
    let root = PathBuf::from(&a[1]);
    let storage = PathBuf::from(&a[2]);
    if a[0] == "uninstall" {
        return lifecycle::uninstall(&storage, &root);
    }
    if a[0] != "install" {
        return Err("Expected install or uninstall".into());
    }
    let root = tentacle::detect_remastered(&root).ok_or("Remastered installation not found")?;
    let stage = tempfile::tempdir().map_err(|e| e.to_string())?;
    patch::stage(stage.path(), "he", patch::VERSION)?;
    let j = lifecycle::install(
        stage.path(),
        "patch.json",
        &root,
        &storage,
        "day-of-the-tentacle-remastered",
        "he",
        patch::VERSION,
        lifecycle::platform(),
    )?;
    println!(
        "Remastered prototype installed. Recovery journal: {}",
        j.display()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
