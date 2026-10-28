use kitor_patch_lifecycle_harness::{monkey_native as patch, patch_lifecycle as lifecycle};
use std::path::PathBuf;

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 { return Err("Usage: monkey_native_patch install|update|verify|uninstall|launch GAME_ID GAME_ROOT RECOVERY_OR_LOG".into()); }
    let root = PathBuf::from(&args[2]);
    let storage = PathBuf::from(&args[3]);
    match args[0].as_str() {
        "verify" => {
            let problems = lifecycle::verify_installed(&storage, &root, &args[1], patch::VERSION)?;
            if !problems.is_empty() { return Err(problems.join("\n")); }
            println!("Installed files verified");
        },
        "update" => {
            let staging = tempfile::tempdir().map_err(|e|e.to_string())?;
            patch::stage(staging.path(), &args[1], "he", patch::VERSION)?;
            let journal = lifecycle::update(&storage, staging.path(), "patch.windows.candidate.json", &root,
                &args[1], "he", patch::VERSION, lifecycle::platform())?;
            println!("{}", journal.display());
        },
        "install" => {
            patch::launch_command(&root, &args[1])?;
            let staging = tempfile::tempdir().map_err(|e|e.to_string())?;
            patch::stage(staging.path(), &args[1], "he", patch::VERSION)?;
            let journal = lifecycle::install(staging.path(), "patch.windows.candidate.json", &root, &storage,
                &args[1], "he", patch::VERSION, lifecycle::platform())?;
            println!("{}", journal.display());
        },
        "uninstall" => lifecycle::uninstall(&storage, &root)?,
        "launch" => {
            let mut cmd = patch::launch_command(&root, &args[1])?;
            let log = std::fs::File::create(storage).map_err(|e|e.to_string())?;
            cmd.stdout(log.try_clone().map_err(|e|e.to_string())?).stderr(log);
            println!("PID {}", cmd.spawn().map_err(|e|e.to_string())?.id());
        },
        _ => return Err("Unknown action".into()),
    }
    Ok(())
}
fn main() { if let Err(e) = run() { eprintln!("{e}"); std::process::exit(1); } }
