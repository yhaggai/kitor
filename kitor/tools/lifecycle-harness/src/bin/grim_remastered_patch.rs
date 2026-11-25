use kitor_patch_lifecycle_harness::{games::grim_remastered as patch, patch_lifecycle as lifecycle};
use std::path::PathBuf;
use kitor_patch_lifecycle_harness::configured_native_command;

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: grim_remastered_patch install GAME RECOVERY | uninstall GAME JOURNAL | launch GAME LOG".into());
    }
    let root = PathBuf::from(&args[1]);
    let storage = PathBuf::from(&args[2]);
    if args[0] == "uninstall" { return lifecycle::uninstall(&storage, &root); }
    if args[0] == "launch" {
        let mut command = configured_native_command("grim-fandango-remastered", &root, true)?;
        let log = std::fs::File::create(storage).map_err(|e| e.to_string())?;
        command.stdout(log.try_clone().map_err(|e| e.to_string())?).stderr(log);
        let child = command.spawn().map_err(|e| e.to_string())?;
        println!("Native game started: PID {}", child.id());
        return Ok(());
    }
    if args[0] != "install" { return Err("Unknown action".into()); }
    configured_native_command("grim-fandango-remastered", &root, false)?;
    let staging = tempfile::tempdir().map_err(|e| e.to_string())?;
    patch::stage(staging.path(), "he", patch::VERSION)?;
    let journal = lifecycle::install(staging.path(), "patch.json", &root, &storage,
        "grim-fandango-remastered", "he", patch::VERSION, lifecycle::platform())?;
    println!("Recovery journal: {}", journal.display());
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(1); }
}
