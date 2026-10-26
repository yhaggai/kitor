use kitor_patch_lifecycle_harness::patch_lifecycle;
use std::path::Path;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("install") if args.len() == 6 => patch_lifecycle::install(
            Path::new(&args[2]), "patch.osx.candidate.json", Path::new(&args[3]),
            Path::new(&args[4]), "owlboy", "he", "0.2.0-beta.2-kitor1", &args[5]
        ).map(|p| println!("Recovery journal: {}", p.display())),
        Some("uninstall") if args.len() == 4 => patch_lifecycle::uninstall(Path::new(&args[2]),Path::new(&args[3])),
        _ => Err("Usage: owlboy-font-test install PACKAGE RESOURCES RECOVERY osx|windows\n       owlboy-font-test uninstall JOURNAL RESOURCES".into())
    };
    if let Err(e) = result { eprintln!("{e}"); std::process::exit(1); }
}
