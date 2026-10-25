//! Disposable validation only: owned game directories are read-only sources.
use kitor_patch_lifecycle_harness::patch_lifecycle::{self as lifecycle, Package, Source};
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Component, Path, PathBuf}};

type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn relative(name: &str) -> Result<&Path> {
    let path = Path::new(name);
    if name.is_empty() || name.contains('\\') || name.contains(':') || path.components().any(|p| !matches!(p, Component::Normal(_))) {
        return Err(format!("Unsafe declared path: {name}"));
    }
    Ok(path)
}
fn source_path(root: &Path, name: &str) -> Result<PathBuf> {
    let mut path = root.to_path_buf();
    for component in relative(name)?.components() {
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => return Err(format!("Symlink input refused: {name}")),
            Ok(_) => {
                if !path.canonicalize().map_err(err)?.starts_with(root) { return Err(format!("Input escapes configured game: {name}")); }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(err(e)),
        }
    }
    Ok(path)
}
fn validate_game(package_dir: &Path, manifest_name: &str, owned: &Path) -> Result<Value> {
    let python = std::env::var("KITOR_VALIDATION_PYTHON").unwrap_or_else(|_| if cfg!(target_os="windows") { "python".into() } else { "python3".into() });
    let validation = std::process::Command::new(python)
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../build_patch_packages.py"))
        .args(["--check", "--platform", "windows"])
        .arg(package_dir.join(manifest_name)).output().map_err(err)?;
    if !validation.status.success() {
        return Err(format!("Package preflight failed: {}", String::from_utf8_lossy(&validation.stderr).trim()));
    }
    let package: Package = serde_json::from_slice(&fs::read(package_dir.join(manifest_name)).map_err(err)?).map_err(err)?;
    let plan = package.platforms.get("windows").ok_or("No Windows plan")?;
    let changes = if plan.files.is_empty() { &package.files } else { &plan.files };
    if changes.is_empty() { return Err("Empty file plan".into()); }
    let owned = owned.canonicalize().map_err(err)?;
    if !owned.is_dir() { return Err("Game path must be a directory".into()); }
    let mut inputs = BTreeSet::new();
    let mut affected = BTreeSet::new();
    for change in changes {
        relative(&change.path)?;
        inputs.insert(change.path.clone());
        affected.insert(change.path.clone());
        match &change.source {
            Some(Source::Bsdiff { input, .. } | Source::Xdelta { input, .. }) => { relative(input)?; inputs.insert(input.clone()); }
            _ => (),
        }
    }
    let fixture = tempfile::tempdir().map_err(err)?;
    let target = fixture.path().join("game-subset");
    fs::create_dir(&target).map_err(err)?;
    let mut originals = BTreeMap::new();
    for name in &inputs {
        let source = source_path(&owned, name)?;
        if !source.exists() { continue; }
        if !source.is_file() { return Err(format!("Declared input is not a file: {name}")); }
        let before = lifecycle::hash(&source)?;
        let destination = target.join(relative(name)?);
        fs::create_dir_all(destination.parent().unwrap()).map_err(err)?;
        fs::copy(&source, &destination).map_err(err)?;
        if lifecycle::hash(&destination)? != before || lifecycle::hash(&source)? != before {
            return Err(format!("Owned input changed during copying: {name}"));
        }
        originals.insert(name.clone(), before);
    }
    let sentinel = format!("__kitor_validation_{}__", fixture.path().file_name().unwrap().to_string_lossy());
    let later = format!("{sentinel}_later");
    if inputs.contains(&sentinel) || inputs.contains(&later) { return Err("Sentinel collision".into()); }
    fs::write(target.join(&sentinel), b"untouched sentinel").map_err(err)?;
    let journal = lifecycle::install(package_dir, manifest_name, &target, &fixture.path().join("recovery"), &package.scummvm_id, &package.language, &package.version, "windows")?;
    let expected_backups = originals.keys().filter(|name| affected.contains(*name)).count();
    let actual_backups = fs::read_dir(journal.join("originals")).map_err(err)?.count();
    if expected_backups != actual_backups { return Err(format!("Backup count mismatch: expected {expected_backups}, got {actual_backups}")); }
    // Lifecycle verifies installed SHA256/MD5 guards. Also compare copy outputs.
    for change in changes {
        if let Some(Source::Copy { path, .. }) = &change.source {
            if lifecycle::hash(&target.join(&change.path))? != lifecycle::hash(&package_dir.join(relative(path)?))? {
                return Err(format!("Installed copy mismatch: {}", change.path));
            }
        }
    }
    fs::write(target.join(&later), b"added later sentinel").map_err(err)?;
    lifecycle::uninstall(&journal, &target)?;
    for (name, original_hash) in &originals {
        if lifecycle::hash(&target.join(relative(name)?))? != *original_hash { return Err(format!("Restoration mismatch: {name}")); }
        if lifecycle::hash(&source_path(&owned, name)?)? != *original_hash { return Err(format!("Owned input changed externally during validation: {name}")); }
    }
    for name in &affected {
        if !originals.contains_key(name) && target.join(relative(name)?).exists() { return Err(format!("Created file remains: {name}")); }
    }
    if fs::read(target.join(&sentinel)).map_err(err)? != b"untouched sentinel" || fs::read(target.join(&later)).map_err(err)? != b"added later sentinel" {
        return Err("Unrelated sentinel changed".into());
    }
    let guarded_files = changes.iter().filter(|f| f.original_sha256.is_some() || f.original_md5.is_some()).count();
    Ok(json!({"guardedFileCount":guarded_files,"editionValidation":if guarded_files == changes.len() { "declared_original_fingerprints_matched" } else { "incomplete_or_absent_original_fingerprints" },"warning":"Pass means disposable install/uninstall roundtrip only; game ownership, complete edition compatibility, and gameplay are not verified", "copiedOriginalFiles":originals.len(), "backedUpFiles":expected_backups, "affectedFiles":affected.len(), "selectiveRoundtrip":"passed", "gameplay":"not_run"}))
}
fn run() -> Result<bool> {
    let mut args = std::env::args().skip(1);
    let config = PathBuf::from(args.next().ok_or("Usage: validate_owned_games CONFIG REPORT [--include-candidates] [--workspace PATH] [--allow-non-windows]")?);
    let report = PathBuf::from(args.next().ok_or("Missing report path")?);
    let mut workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../patches-workspace");
    let mut candidates = false;
    let mut allow_other = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--include-candidates" => candidates = true,
            "--allow-non-windows" => allow_other = true,
            "--workspace" => workspace = PathBuf::from(args.next().ok_or("Missing workspace")?),
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }
    if !cfg!(target_os="windows") && !allow_other { return Err("Native Windows required; --allow-non-windows is for developer fixture tests only".into()); }
    let config: BTreeMap<String, String> = serde_json::from_slice(&fs::read(config).map_err(err)?).map_err(err)?;
    let report_parent = report.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let resolved_parent = report_parent.canonicalize().map_err(err)?;
    for path in config.values().filter(|p| !p.trim().is_empty()) {
        if let Ok(root) = Path::new(path).canonicalize() {
            if resolved_parent.starts_with(root) { return Err("Report must be outside every configured game directory".into()); }
        }
    }
    if report.exists() { return Err("Report already exists; choose a new output filename".into()); }
    let mut results = Vec::new();
    let mut passed = true;
    for (id, path) in config {
        let mut result = json!({"gameId":id, "gamePath":path, "nativeWindows":cfg!(target_os="windows"), "gameplay":"not_run"});
        if path.trim().is_empty() { result["status"] = json!("skip"); result["reason"] = json!("No game directory configured"); results.push(result); continue; }
        let outcome = (|| {
            if relative(&id)?.components().count() != 1 { return Err("Invalid game ID".into()); }
            let package_dir = workspace.join(&id).join("he").canonicalize().map_err(err)?;
            let manifest = if package_dir.join("patch.json").is_file() { "patch.json" } else if candidates && package_dir.join("patch.windows.candidate.json").is_file() { "patch.windows.candidate.json" } else { return Ok(None); };
            validate_game(&package_dir, manifest, Path::new(&path)).map(Some)
        })();
        match outcome {
            Ok(Some(details)) => { result["status"] = json!("pass"); result["details"] = details; }
            Ok(None) => { result["status"] = json!("skip"); result["reason"] = json!("No enabled manifest (candidate requires --include-candidates)"); }
            Err(error) => { passed = false; result["status"] = json!("fail"); result["error"] = json!(error); }
        }
        eprintln!("{}: {}", id, result["status"]);
        results.push(result);
    }
    let passed_count = results.iter().filter(|r| r["status"] == "pass").count();
    let failed_count = results.iter().filter(|r| r["status"] == "fail").count();
    let skipped_count = results.len() - passed_count - failed_count;
    if passed_count == 0 { passed = false; eprintln!("No games validated successfully. Fill at least one configured game/data directory."); }
    let summary = json!({"passed":passed_count,"failed":failed_count,"skipped":skipped_count});
    let bytes = serde_json::to_vec_pretty(&json!({"schemaVersion":1,"nativeWindows":cfg!(target_os="windows"),"scope":"Disposable subset install/uninstall only; gameplay not tested", "summary":summary, "results":results})).map_err(err)?;
    use std::io::Write;
    fs::OpenOptions::new().write(true).create_new(true).open(&report).map_err(err)?.write_all(&bytes).map_err(err)?;
    Ok(passed)
}
fn main() {
    match run() { Ok(true) => (), Ok(false) => std::process::exit(1), Err(error) => { eprintln!("{error}"); std::process::exit(2); } }
}
