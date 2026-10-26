//! Standalone native installer sharing Kitor's exact staged transaction engine.
use kitor_patch_lifecycle_harness::{patch_lifecycle as lifecycle, tentacle};
use std::path::PathBuf;
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: tentacle_patch install GAME_FOLDER RECOVERY_FOLDER\n       tentacle_patch uninstall GAME_FOLDER JOURNAL_FOLDER".into());
    }
    let root = PathBuf::from(&args[1]);
    let storage = PathBuf::from(&args[2]);
    if args[0] == "uninstall" {
        lifecycle::uninstall(&storage, &root)?;
        println!("Classic Hebrew patch removed; unrelated files preserved.");
        return Ok(());
    }
    if args[0] != "install" {
        return Err("Expected install or uninstall".into());
    }
    let root = tentacle::detect(&root).ok_or(
        "Supported classic files or complete Day of the Tentacle Remastered installation not found",
    )?;
    let stage = tempfile::tempdir().map_err(|e| e.to_string())?;
    tentacle::stage(stage.path(), "he", tentacle::VERSION, &root)?;
    let journal = lifecycle::install(
        stage.path(),
        "patch.json",
        &root,
        &storage,
        "tentacle",
        "he",
        tentacle::VERSION,
        lifecycle::platform(),
    )?;
    println!(
        "Installed classic Hebrew data: {}\nRecovery journal: {}",
        if tentacle::detect_classic(&root).is_some() {
            root.clone()
        } else {
            root.join("hebrew")
        }
        .display(),
        journal.display()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
