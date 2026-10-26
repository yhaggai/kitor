//! Declarative patch staging and selective, journaled install/uninstall.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub schema_version: u32,
    pub scummvm_id: String,
    pub language: String,
    pub version: String,
    pub filename: String,
    pub platforms: BTreeMap<String, Plan>,
    #[serde(default)]
    pub files: Vec<Change>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(default)]
    pub files: Vec<Change>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub path: String,
    pub operation: Operation,
    pub source: Option<Source>,
    pub original_sha256: Option<String>,
    pub installed_sha256: Option<String>,
    pub original_md5: Option<String>,
    pub installed_md5: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Install,
    Modify,
    Create,
    Delete,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    Copy {
        path: String,
        sha256: String,
    },
    Bsdiff {
        path: String,
        sha256: String,
        input: String,
    },
    Reconstruct { path: String, sha256: String, input: String },
    Xdelta {
        path: String,
        sha256: String,
        input: String,
        windows_tool: String,
        windows_tool_sha256: String,
    },
}

// Legacy publishers provide MD5 as an edition identifier; payload integrity and
// recovery journals always use SHA-256. MD5 is not an authenticity guarantee.
fn check_md5(path: &Path, expected: &str) -> Result<()> {
    let mut file = fs::File::open(path).map_err(err)?;
    let mut digest = md5::Md5::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    if expected.len() != 32 || format!("{:x}", digest.finalize()) != expected {
        return Err(format!(
            "Legacy edition checksum mismatch: {}",
            path.display()
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Journal {
    schema_version: u32,
    target: PathBuf,
    game: String,
    language: String,
    version: String,
    status: String,
    files: Vec<Record>,
    created_dirs: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct Record {
    path: String,
    original_hash: Option<String>,
    installed_hash: Option<String>,
    started: bool,
    restored: bool,
}

pub fn platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}
pub fn hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(err)?;
    let mut digest = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn current_hash(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_file() => hash(path).map(Some),
        Ok(_) => Err(format!("Expected a regular file: {}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err(e)),
    }
}

/// Resolve case-insensitively for Windows-origin game assets, reject ambiguous
/// aliases and symlinks on every platform, and never allow a path to escape.
pub fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.is_empty() || relative.contains(['\\', ':']) {
        return Err(format!("Unsafe relative path: {relative}"));
    }
    let mut result = root.to_path_buf();
    for part in relative.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(format!("Unsafe relative path: {relative}"));
        }
        let mut matches = Vec::new();
        if result.is_dir() {
            for entry in fs::read_dir(&result).map_err(err)? {
                let entry = entry.map_err(err)?;
                if entry
                    .file_name()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(part)
                {
                    matches.push(entry.path());
                }
            }
        }
        if matches.len() > 1 {
            return Err(format!("Ambiguous path: {relative}"));
        }
        result = matches.pop().unwrap_or_else(|| result.join(part));
        match fs::symlink_metadata(&result) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("Symlink not allowed: {relative}"))
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(err(e)),
            _ => (),
        }
    }
    Ok(result)
}
fn check_hash(path: &Path, expected: &str) -> Result<()> {
    if hash(path)? != expected {
        return Err(format!("Checksum mismatch: {}", path.display()));
    }
    Ok(())
}
fn write_json(path: &Path, value: &Journal) -> Result<()> {
    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing journal parent")?)
        .map_err(err)?;
    tmp.write_all(&serde_json::to_vec_pretty(value).map_err(err)?)
        .map_err(err)?;
    tmp.as_file().sync_all().map_err(err)?;
    tmp.persist(path).map_err(err)?;
    sync_dir(path.parent().unwrap())
}
fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(err)?;
    let _ = path;
    Ok(())
}
fn save(dir: &Path, journal: &Journal) -> Result<()> {
    write_json(&dir.join("manifest.json"), journal)
}
fn load(dir: &Path) -> Result<Journal> {
    let journal: Journal = serde_json::from_slice(&fs::read(dir.join("manifest.json")).map_err(|_| {
        "This backup has no selective installation manifest. Legacy folder backups are preserved; automatic full-folder restore is disabled.".to_string()
    })?).map_err(err)?;
    if journal.schema_version != 1 {
        return Err("Unsupported installation manifest version".into());
    }
    Ok(journal)
}
/// Inspect installed bytes without changing game files, backups, or records.
pub fn verify_installed(dir: &Path, target: &Path, game: &str, version: &str) -> Result<Vec<String>> {
    let journal = load(dir)?;
    if journal.target != target.canonicalize().map_err(err)? || journal.game != game || journal.version != version {
        return Err("Patch manifest belongs to a different installation or version".into());
    }
    if journal.status != "active" || journal.files.is_empty() {
        return Err(format!("Patch installation is not active ({})", journal.status));
    }
    let mut problems = Vec::new();
    for record in journal.files {
        if !record.started || record.restored { return Err("Patch installation is incomplete".into()); }
        let actual = current_hash(&safe_path(target, &record.path)?)?;
        if actual != record.installed_hash {
            problems.push(format!("{}: {}", record.path, if actual.is_none() { "missing" } else { "changed" }));
        }
    }
    Ok(problems)
}

fn copy_durable(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to.parent().ok_or("Missing destination parent")?).map_err(err)?;
    // Windows FlushFileBuffers requires a writable handle. Reopening the copy
    // with File::open creates a read-only handle and fails even for normal files.
    // replace retains the writable temporary handle opened before fs::copy also
    // copies source permissions, so read-only source files remain supported.
    replace(from, to)
}
fn replace(from: &Path, to: &Path) -> Result<()> {
    let parent = to.parent().ok_or("Missing target parent")?;
    let temp = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    fs::copy(from, temp.path()).map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    temp.persist(to).map_err(err)?;
    sync_dir(parent)
}
fn lock(dir: &Path) -> Result<fs::File> {
    fs::create_dir_all(dir).map_err(err)?;
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(".lock"))
        .map_err(err)?;
    file.try_lock()
        .map_err(|_| "Another patch operation is running for this installation".to_string())?;
    Ok(file)
}
fn installation_dir(storage: &Path, target: &Path) -> PathBuf {
    storage.join(format!(
        "{:x}",
        Sha256::digest(target.to_string_lossy().as_bytes())
    ))
}
fn run(mut command: Command) -> Result<()> {
    let output = command.output().map_err(|e| {
        format!(
            "Patch tool could not run: {e}. Install the required Python 3 or xdelta3 dependency."
        )
    })?;
    if !output.status.success() {
        return Err(format!(
            "Patch tool failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[cfg(any(target_os = "windows", test))]
fn trusted_windows_decoder() -> Result<(tempfile::TempDir, PathBuf)> {
    // Compiled into Kitor from reviewed repository bytes. Neither the program
    // nor its trust anchor comes from the downloaded package manifest.
    const DECODER: &[u8] = include_bytes!("../../../../patches-workspace/pajama/he/payload/xdelta3.exe");
    let dir = tempfile::Builder::new().prefix("kitor-decoder-").tempdir().map_err(err)?;
    let path = dir.path().join("xdelta3.exe");
    fs::write(&path, DECODER).map_err(err)?;
    Ok((dir, path))
}

fn stage_source(source: &Source, package: &Path, target: &Path, output: &Path) -> Result<()> {
    let (path, checksum) = match source {
        Source::Copy { path, sha256 }
        | Source::Bsdiff { path, sha256, .. }
        | Source::Reconstruct { path, sha256, .. }
        | Source::Xdelta { path, sha256, .. } => (path, sha256),
    };
    let asset = safe_path(package, path)?;
    check_hash(&asset, checksum)?;
    match source {
        Source::Copy { .. } => {
            copy_durable(&asset, output)?;
        }
        Source::Reconstruct { input, .. } => {
            super::reconstruction::apply(&asset, &safe_path(target, input)?, output)?;
        }
        Source::Bsdiff { input, .. } => {
            // Only the trusted backend decoder runs, never a package installer.
            let input = safe_path(target, input)?;
            let mut command = if cfg!(target_os = "windows") {
                let mut c = Command::new("py");
                c.arg("-3");
                c
            } else {
                Command::new("python3")
            };
            command
                .arg("-I")
                .arg("-c")
                .arg(include_str!("apply_bsdiff.py"))
                .arg(&input)
                .arg(&asset)
                .arg(output)
                .current_dir(output.parent().unwrap());
            run(command)?;
        }
        Source::Xdelta { input, .. } => {
            let input = safe_path(target, input)?;
            #[cfg(target_os = "windows")]
            let (_decoder_dir, tool) = trusted_windows_decoder()?;
            #[cfg(not(target_os = "windows"))]
            let tool = PathBuf::from("xdelta3");
            let mut command = Command::new(tool);
            command
                .args(["-d", "-s"])
                .arg(&input)
                .arg(&asset)
                .arg(output)
                .current_dir(output.parent().unwrap());
            run(command)?;
        }
    }
    if !output.is_file() {
        return Err("Patch tool did not produce its declared output".into());
    }
    Ok(())
}

/// Read a package manifest, stage only declared outputs, then apply a durable
/// transaction. The journal is authoritative even if the database write fails.
pub fn install(
    package_dir: &Path,
    manifest_name: &str,
    target: &Path,
    storage: &Path,
    game: &str,
    language: &str,
    version: &str,
    platform: &str,
) -> Result<PathBuf> {
    let package: Package =
        serde_json::from_slice(&fs::read(safe_path(package_dir, manifest_name)?).map_err(err)?)
            .map_err(err)?;
    if package.schema_version != 1
        || package.scummvm_id != game
        || package.language != language
        || package.version != version
    {
        return Err("Patch manifest identity/version does not match the catalog".into());
    }
    let plan = package
        .platforms
        .get(platform)
        .ok_or("Patch has no file plan for this platform")?;
    let files = if plan.files.is_empty() {
        &package.files
    } else {
        &plan.files
    };
    if files.is_empty() {
        return Err("Patch file plan is empty".into());
    }
    let target = target.canonicalize().map_err(err)?;
    fs::create_dir_all(storage).map_err(err)?;
    let storage = storage.canonicalize().map_err(err)?;
    if storage.starts_with(&target) {
        return Err("Recovery storage must be outside the game directory".into());
    }
    let parent = installation_dir(&storage, &target);
    let _lock = lock(&parent)?;
    let mut paths = HashSet::new();
    for file in files {
        safe_path(&target, &file.path)?;
        if !paths.insert(file.path.to_lowercase()) {
            return Err("Duplicate patch target path".into());
        }
    }
    // Reject file/directory collisions before any writes.
    for path in &paths {
        for other in &paths {
            if other.starts_with(&format!("{path}/")) {
                return Err("Overlapping patch target paths".into());
            }
        }
    }
    for entry in fs::read_dir(&parent).map_err(err)? {
        let dir = entry.map_err(err)?.path();
        if !dir.is_dir() || !dir.join("manifest.json").exists() {
            continue;
        }
        let mut existing = load(&dir)?;
        if existing.target != target {
            return Err("Recovery manifest target mismatch".into());
        }
        if existing.status != "active" && existing.status != "restored" {
            restore_inner(&dir, &mut existing)?;
        }
        if existing.status == "active" {
            if existing.game == game && existing.language == language && existing.version == version
            {
                for record in &existing.files {
                    if current_hash(&safe_path(&target, &record.path)?)? != record.installed_hash {
                        return Err("Installed patch files changed; resolve the conflict before reinstalling".into());
                    }
                }
                return Ok(dir);
            }
            if existing
                .files
                .iter()
                .any(|r| paths.contains(&r.path.to_lowercase()))
            {
                return Err("Another active patch owns these files. Uninstall it before upgrading or replacing it.".into());
            }
        }
    }
    let temp = tempfile::Builder::new()
        .prefix("install-")
        .tempdir_in(&parent)
        .map_err(err)?;
    let dir = temp.path();
    fs::create_dir(dir.join("originals")).map_err(err)?;
    fs::create_dir(dir.join("outputs")).map_err(err)?;
    let mut journal = Journal {
        schema_version: 1,
        target: target.clone(),
        game: game.into(),
        language: language.into(),
        version: version.into(),
        status: "prepared".into(),
        files: vec![],
        created_dirs: vec![],
    };
    for (index, change) in files.iter().enumerate() {
        let path = safe_path(&target, &change.path)?;
        let original = current_hash(&path)?;
        match change.operation {
            Operation::Create if original.is_some() => {
                return Err(format!(
                    "Creation conflicts with existing file: {}",
                    change.path
                ))
            }
            Operation::Modify | Operation::Delete if original.is_none() => {
                return Err(format!("Required original is missing: {}", change.path))
            }
            _ => (),
        }
        if let Some(expected) = &change.original_sha256 {
            if original.as_ref() != Some(expected) {
                return Err(format!("Unsupported original version: {}", change.path));
            }
        }
        if let Some(expected) = &change.original_md5 {
            check_md5(&path, expected)?;
        }
        let output = dir.join("outputs").join(index.to_string());
        let installed_hash = if change.operation == Operation::Delete {
            if change.source.is_some() {
                return Err("Delete operations cannot have an output source".into());
            }
            None
        } else {
            stage_source(
                change
                    .source
                    .as_ref()
                    .ok_or("Missing patch output source")?,
                package_dir,
                &target,
                &output,
            )?;
            if let Some(expected) = &change.installed_sha256 {
                check_hash(&output, expected)?;
            }
            if let Some(expected) = &change.installed_md5 {
                check_md5(&output, expected)?;
            }
            if original.is_some() {
                fs::set_permissions(&output, fs::metadata(&path).map_err(err)?.permissions())
                    .map_err(err)?;
            }
            Some(hash(&output)?)
        };
        // Recheck after the tool runs; never back up a file that changed meanwhile.
        if current_hash(&path)? != original {
            return Err("Game changed while staging the patch".into());
        }
        if let Some(expected) = &original {
            let backup = dir.join("originals").join(index.to_string());
            copy_durable(&path, &backup)?;
            check_hash(&backup, expected)?;
        }
        let relative = path
            .strip_prefix(&target)
            .map_err(err)?
            .to_string_lossy()
            .replace('\\', "/");
        let mut ancestor = path.parent();
        while let Some(folder) = ancestor {
            if folder == target {
                break;
            }
            if !folder.exists() {
                let rel = folder
                    .strip_prefix(&target)
                    .map_err(err)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if !journal.created_dirs.contains(&rel) {
                    journal.created_dirs.push(rel);
                }
            }
            ancestor = folder.parent();
        }
        journal.files.push(Record {
            path: relative,
            original_hash: original,
            installed_hash,
            started: false,
            restored: false,
        });
    }
    // Keep recovery information before the first game mutation.
    save(dir, &journal)?;
    let dir = temp.keep();
    let result = apply(&dir, &mut journal);
    if let Err(failure) = result {
        return match restore_inner(&dir, &mut journal) {
            Ok(()) => Err(format!("{failure}. Changes rolled back.")),
            Err(recovery) => Err(format!(
                "{failure}. Recovery required at {}: {recovery}",
                dir.display()
            )),
        };
    }
    // Staged output is disposable; originals and journal remain until uninstall.
    let _ = fs::remove_dir_all(dir.join("outputs"));
    Ok(dir)
}
fn apply(dir: &Path, journal: &mut Journal) -> Result<()> {
    journal.status = "applying".into();
    save(dir, journal)?;
    for index in 0..journal.files.len() {
        let record = &journal.files[index];
        let path = safe_path(&journal.target, &record.path)?;
        if current_hash(&path)? != record.original_hash {
            return Err(format!("Game file changed: {}", record.path));
        }
        journal.files[index].started = true;
        save(dir, journal)?;
        let record = &journal.files[index];
        if let Some(expected) = &record.installed_hash {
            fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
            replace(&dir.join("outputs").join(index.to_string()), &path)?;
            check_hash(&path, expected)?;
        } else {
            fs::remove_file(&path).map_err(err)?;
            sync_dir(path.parent().unwrap())?;
        }
    }
    journal.status = "active".into();
    save(dir, journal)
}

fn restore_inner(dir: &Path, journal: &mut Journal) -> Result<()> {
    if journal.status == "restored" {
        return Ok(());
    }
    // Preflight every backup and every touched file before restoring anything.
    for (index, record) in journal.files.iter().enumerate() {
        if record.restored || !record.started {
            continue;
        }
        if let Some(expected) = &record.original_hash {
            check_hash(&dir.join("originals").join(index.to_string()), expected)
                .map_err(|e| format!("Missing or invalid original backup: {e}"))?;
        }
        let current = current_hash(&safe_path(&journal.target, &record.path)?)?;
        if current != record.original_hash && current != record.installed_hash {
            return Err(format!(
                "File changed after patching; refusing to overwrite: {}",
                record.path
            ));
        }
    }
    journal.status = "restoring".into();
    save(dir, journal)?;
    for index in (0..journal.files.len()).rev() {
        let record = &journal.files[index];
        if record.restored || !record.started {
            continue;
        }
        let path = safe_path(&journal.target, &record.path)?;
        let current = current_hash(&path)?;
        if current != record.original_hash {
            if current != record.installed_hash {
                return Err(format!("File changed during restoration: {}", record.path));
            }
            if let Some(expected) = &record.original_hash {
                replace(&dir.join("originals").join(index.to_string()), &path)?;
                check_hash(&path, expected)?;
            } else if path.exists() {
                fs::remove_file(&path).map_err(err)?;
                sync_dir(path.parent().unwrap())?;
            }
        }
        journal.files[index].restored = true;
        save(dir, journal)?;
    }
    journal
        .created_dirs
        .sort_by_key(|p| std::cmp::Reverse(p.len()));
    for folder in &journal.created_dirs {
        let folder = safe_path(&journal.target, folder)?;
        if folder.is_dir() && fs::read_dir(&folder).map_err(err)?.next().is_none() {
            fs::remove_dir(&folder).map_err(err)?;
        }
    }
    journal.status = "restored".into();
    save(dir, journal)
}

pub fn uninstall(dir: &Path, target: &Path) -> Result<()> {
    // Deliberately do not interpret old full-directory backups as manifests.
    let _lock = lock(dir.parent().ok_or("Missing transaction parent")?)?;
    let mut journal = load(dir)?;
    let target = target.canonicalize().map_err(err)?;
    if journal.target != target {
        return Err("Backup belongs to a different game installation".into());
    }
    restore_inner(dir, &mut journal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn windows_decoder_is_bundled_and_isolated_from_package_tools() {
        let (dir, decoder) = trusted_windows_decoder().unwrap();
        assert_eq!(decoder.parent(), Some(dir.path()));
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert_eq!(hash(&decoder).unwrap(), "d81f59b2fe5e8589c0ee9782e231c805084f4d23dfade413903a4cad63b4e342");
        // Legacy manifest fields remain parseable but cannot choose the decoder.
        let malicious: Source = serde_json::from_value(json!({
            "kind":"xdelta", "path":"delta", "sha256":"0".repeat(64),
            "input":"game.dat", "windows_tool":"../../malware.exe",
            "windows_tool_sha256":"0".repeat(64)
        })).unwrap();
        assert!(matches!(malicious, Source::Xdelta { .. }));
        drop(dir);
        assert!(!decoder.exists());
    }
    struct Fixture {
        _root: tempfile::TempDir,
        game: PathBuf,
        package: PathBuf,
        storage: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = tempfile::tempdir().unwrap();
            let game = root.path().join("game");
            let package = root.path().join("package");
            let storage = root.path().join("transactions");
            fs::create_dir(&game).unwrap();
            fs::create_dir(&package).unwrap();
            Self {
                _root: root,
                game,
                package,
                storage,
            }
        }
        fn plan(&self, changes: serde_json::Value, version: &str) {
            fs::write(
                self.package.join("patch.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema_version":1,"scummvm_id":"test","language":"he","version":version,
                    "filename":"patch.zip","platforms":{"windows":{},"osx":{}},"files":changes
                }))
                .unwrap(),
            )
            .unwrap();
        }
        fn copy(&self, target: &str, operation: &str, contents: &[u8]) -> serde_json::Value {
            let name = format!("{:x}", Sha256::digest(contents));
            fs::write(self.package.join(&name), contents).unwrap();
            json!({"path":target,"operation":operation,"source":{"kind":"copy","path":name,"sha256":hash(&self.package.join(&name)).unwrap()}})
        }
        fn install(&self, platform: &str, version: &str) -> Result<PathBuf> {
            install(
                &self.package,
                "patch.json",
                &self.game,
                &self.storage,
                "test",
                "he",
                version,
                platform,
            )
        }
    }
    #[test]
    fn durable_copy_flushes_writable_handle_and_preserves_readonly_permissions() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let destination = root.path().join("nested/copy");
        fs::write(&source, b"original readonly bytes").unwrap();
        let mut permissions = fs::metadata(&source).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&source, permissions).unwrap();
        copy_durable(&source, &destination).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"original readonly bytes");
        assert!(fs::metadata(&destination).unwrap().permissions().readonly());
        #[cfg(windows)]
        for path in [&source, &destination] {
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_readonly(false);
            fs::set_permissions(path, permissions).unwrap();
        }
    }
    #[test]
    fn selective_roundtrip_both_layouts_preserves_unrelated_files_and_permissions() {
        for platform in ["windows", "osx"] {
            let f = Fixture::new();
            fs::write(f.game.join("data.bin"), b"original").unwrap();
            fs::write(f.game.join("deleted.bin"), b"restore me").unwrap();
            fs::write(f.game.join("save.dat"), b"untouched").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(f.game.join("data.bin"), fs::Permissions::from_mode(0o755))
                    .unwrap();
            }
            f.plan(json!([f.copy("DATA.bin","modify",b"translated"),f.copy("new/font.bin","create",b"glyphs"),
                {"path":"deleted.bin","operation":"delete"}]),"1");
            let dir = f.install(platform, "1").unwrap();
            assert_eq!(fs::read_dir(dir.join("originals")).unwrap().count(), 2);
            assert!(!f.game.join("patch.json").exists());
            assert!(!f.game.join("deleted.bin").exists());
            assert_eq!(fs::read(f.game.join("data.bin")).unwrap(), b"translated");
            assert_eq!(f.install(platform, "1").unwrap(), dir);
            fs::write(f.game.join("after-install.txt"), b"keep this").unwrap();
            uninstall(&dir, &f.game).unwrap();
            uninstall(&dir, &f.game).unwrap();
            assert_eq!(fs::read(f.game.join("data.bin")).unwrap(), b"original");
            assert_eq!(fs::read(f.game.join("deleted.bin")).unwrap(), b"restore me");
            assert_eq!(fs::read(f.game.join("save.dat")).unwrap(), b"untouched");
            assert!(f.game.join("after-install.txt").exists());
            assert!(!f.game.join("new").exists());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(f.game.join("data.bin"))
                        .unwrap()
                        .permissions()
                        .mode()
                        & 0o777,
                    0o755
                );
            }
            let second = f.install(platform, "1").unwrap();
            assert_ne!(dir, second);
        }
    }
    #[test]
    fn rejects_version_conflicts_missing_backups_and_later_changes_without_mutation() {
        let f = Fixture::new();
        fs::write(f.game.join("data"), b"original").unwrap();
        f.plan(json!([f.copy("data", "modify", b"patched")]), "1");
        let dir = f.install("osx", "1").unwrap();
        f.plan(json!([f.copy("data", "modify", b"upgrade")]), "2");
        assert!(f.install("osx", "2").unwrap_err().contains("active patch"));
        fs::write(f.game.join("data"), b"later mod").unwrap();
        assert!(uninstall(&dir, &f.game)
            .unwrap_err()
            .contains("changed after"));
        assert_eq!(fs::read(f.game.join("data")).unwrap(), b"later mod");
        fs::write(f.game.join("data"), b"patched").unwrap();
        let backup = dir.join("originals/0");
        let original = fs::read(&backup).unwrap();
        fs::remove_file(&backup).unwrap();
        assert!(uninstall(&dir, &f.game)
            .unwrap_err()
            .contains("Missing or invalid"));
        assert_eq!(fs::read(f.game.join("data")).unwrap(), b"patched");
        fs::write(backup, original).unwrap();
        uninstall(&dir, &f.game).unwrap();
    }
    #[test]
    fn interrupted_apply_and_restore_recover_without_replacing_originals() {
        let f = Fixture::new();
        fs::write(f.game.join("data"), b"original").unwrap();
        f.plan(
            json!([
                f.copy("data", "modify", b"patched"),
                f.copy("new", "create", b"new")
            ]),
            "1",
        );
        let dir = f.install("osx", "1").unwrap();
        let mut journal = load(&dir).unwrap();
        // Simulate death between the first file replacement and the next write.
        fs::remove_file(f.game.join("new")).unwrap();
        journal.status = "applying".into();
        journal.files[1].started = false;
        save(&dir, &journal).unwrap();
        let recovered = f.install("osx", "1").unwrap();
        assert_ne!(dir, recovered);
        assert_eq!(
            fs::read(recovered.join("originals/0")).unwrap(),
            b"original"
        );
        let mut journal = load(&recovered).unwrap();
        journal.status = "restoring".into();
        // Simulate death after restoring bytes but before journaling completion.
        fs::copy(recovered.join("originals/0"), f.game.join("data")).unwrap();
        save(&recovered, &journal).unwrap();
        uninstall(&recovered, &f.game).unwrap();
        assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
        assert!(!f.game.join("new").exists());
    }
    #[test]
    fn failed_apply_rolls_back_only_started_operations() {
        let f = Fixture::new();
        for name in ["first", "second"] {
            fs::write(f.game.join(name), b"original").unwrap();
        }
        fs::write(f.game.join("unrelated"), b"keep").unwrap();
        let dir = f._root.path().join("transaction");
        fs::create_dir_all(dir.join("originals")).unwrap();
        fs::create_dir(dir.join("outputs")).unwrap();
        let mut journal = Journal {
            schema_version: 1,
            target: f.game.canonicalize().unwrap(),
            game: "test".into(),
            language: "he".into(),
            version: "1".into(),
            status: "prepared".into(),
            files: vec![],
            created_dirs: vec![],
        };
        for (i, name) in ["first", "second"].iter().enumerate() {
            fs::copy(f.game.join(name), dir.join("originals").join(i.to_string())).unwrap();
            journal.files.push(Record {
                path: name.to_string(),
                original_hash: Some(hash(&f.game.join(name)).unwrap()),
                installed_hash: Some(format!("{:x}", Sha256::digest(b"patched"))),
                started: false,
                restored: false,
            });
        }
        // Missing second output simulates a disk/tool failure after one commit.
        fs::write(dir.join("outputs/0"), b"patched").unwrap();
        assert!(apply(&dir, &mut journal).is_err());
        assert_eq!(fs::read(f.game.join("first")).unwrap(), b"patched");
        restore_inner(&dir, &mut journal).unwrap();
        for name in ["first", "second"] {
            assert_eq!(fs::read(f.game.join(name)).unwrap(), b"original");
        }
        assert_eq!(fs::read(f.game.join("unrelated")).unwrap(), b"keep");
    }

    #[test]
    fn legacy_md5_guards_reject_wrong_input_and_output_before_mutation() {
        for wrong_input in [true, false] {
            let f = Fixture::new();
            fs::write(f.game.join("existing"), b"original").unwrap();
            let mut change = f.copy("existing", "modify", b"patched");
            change["original_md5"] = json!(format!(
                "{:x}",
                md5::Md5::digest(if wrong_input {
                    b"wrong".as_slice()
                } else {
                    b"original".as_slice()
                })
            ));
            change["installed_md5"] = json!("00000000000000000000000000000000");
            f.plan(json!([change]), "1");
            assert!(f
                .install("osx", "1")
                .unwrap_err()
                .contains("Legacy edition checksum mismatch"));
            assert_eq!(fs::read(f.game.join("existing")).unwrap(), b"original");
        }
        let f = Fixture::new();
        fs::write(f.game.join("existing"), b"original").unwrap();
        let mut change = f.copy("existing", "modify", b"patched");
        change["original_md5"] = json!(format!("{:x}", md5::Md5::digest(b"original")));
        change["installed_md5"] = json!(format!("{:x}", md5::Md5::digest(b"patched")));
        f.plan(json!([change]), "1");
        let journal = f.install("osx", "1").unwrap();
        uninstall(&journal, &f.game).unwrap();
        assert_eq!(fs::read(f.game.join("existing")).unwrap(), b"original");
    }

    #[test]
    fn preflight_rejects_paths_creation_collisions_and_invalid_payloads() {
        for path in ["../escape", "/absolute", "C:/file", "a\\b", "a/../b"] {
            let f = Fixture::new();
            f.plan(json!([f.copy(path, "create", b"new")]), "1");
            assert!(f.install("osx", "1").is_err());
        }
        let f = Fixture::new();
        fs::write(f.game.join("existing"), b"keep").unwrap();
        f.plan(json!([f.copy("existing", "create", b"new")]), "1");
        assert!(f.install("osx", "1").is_err());
        let mut change = f.copy("existing", "modify", b"new");
        change["original_sha256"] = json!("wrong");
        f.plan(json!([change]), "1");
        assert!(f
            .install("osx", "1")
            .unwrap_err()
            .contains("Unsupported original"));
        let mut change = f.copy("existing", "modify", b"new");
        change["source"]["sha256"] = json!("wrong");
        f.plan(json!([change]), "1");
        assert!(f
            .install("osx", "1")
            .unwrap_err()
            .contains("Checksum mismatch"));
        assert_eq!(fs::read(f.game.join("existing")).unwrap(), b"keep");
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlink_targets_and_preserves_nonempty_created_directories() {
        let f = Fixture::new();
        std::os::unix::fs::symlink(&f.package, f.game.join("escape")).unwrap();
        f.plan(json!([f.copy("escape/data", "create", b"new")]), "1");
        assert!(f.install("osx", "1").is_err());
        f.plan(json!([f.copy("new/data", "create", b"new")]), "1");
        let dir = f.install("osx", "1").unwrap();
        fs::write(f.game.join("new/user-file"), b"keep").unwrap();
        uninstall(&dir, &f.game).unwrap();
        assert_eq!(fs::read(f.game.join("new/user-file")).unwrap(), b"keep");
    }
    #[test]
    fn legacy_backups_and_wrong_installations_are_not_restored() {
        let f = Fixture::new();
        assert!(uninstall(&f.package, &f.game)
            .unwrap_err()
            .contains("Legacy"));
        f.plan(json!([f.copy("new", "create", b"new")]), "1");
        let dir = f.install("osx", "1").unwrap();
        assert!(uninstall(&dir, &f.package)
            .unwrap_err()
            .contains("different game"));
    }
    #[test]
    fn all_catalog_packages_have_matching_plans_and_verified_assets() {
        // Fixtures use the source registry in both debug and optimized builds;
        // the production loader intentionally uses a different release path.
        let registry: crate::services::registry_service::Registry = serde_json::from_slice(
            &fs::read("../patches/registry.json").unwrap(),
        ).unwrap();
        for game in registry.games {
            if game.patches.is_empty() {
                continue;
            }
            let root = Path::new("../../patches-workspace")
                .join(&game.scummvm_id)
                .join("he");
            let manifest = if root.join("patch.json").is_file() {
                root.join("patch.json")
            } else if root.join("patch.osx.candidate.json").is_file() {
                root.join("patch.osx.candidate.json")
            } else {
                root.join("patch.windows.candidate.json")
            };
            let package: Package =
                serde_json::from_slice(&fs::read(manifest).unwrap()).unwrap();
            assert_eq!(package.version, game.patches[0].version);
            assert_eq!(
                game.patches[0].install_manifest.as_deref(),
                Some("patch.json")
            );
            for platform in ["windows", "osx"] {
                let supported = game.platforms.as_ref().map(|p| match platform {
                    "windows" => p.windows,
                    _ => p.osx,
                }).unwrap_or(false);
                assert_eq!(supported, package.platforms.contains_key(platform),
                    "Registry platform mismatch for {}: {}", game.scummvm_id, platform);
                if !supported {
                    continue;
                }
                let plan = package.platforms.get(platform).unwrap();
                let files = if plan.files.is_empty() {
                    &package.files
                } else {
                    &plan.files
                };
                assert!(!files.is_empty());
                let mut seen = HashSet::new();
                for file in files {
                    assert!(seen.insert(file.path.to_lowercase()));
                    safe_path(&root, &file.path).unwrap();
                    match file.source.as_ref().unwrap() {
                        Source::Copy { path, sha256 } | Source::Bsdiff { path, sha256, .. } | Source::Reconstruct { path, sha256, .. } => {
                            check_hash(&safe_path(&root, path).unwrap(), sha256).unwrap()
                        }
                        Source::Xdelta {
                            path,
                            sha256,
                            windows_tool,
                            windows_tool_sha256,
                            ..
                        } => {
                            check_hash(&safe_path(&root, path).unwrap(), sha256).unwrap();
                            check_hash(
                                &safe_path(&root, windows_tool).unwrap(),
                                windows_tool_sha256,
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn resource_packages_roundtrip_windows_and_macos_plans() {
        for game in ["kyra1-cd", "sq1sci"] {
            let root = Path::new("../../patches-workspace")
                .join(game)
                .join("he")
                .canonicalize()
                .unwrap();
            let package: Package =
                serde_json::from_slice(&fs::read(root.join("patch.json")).unwrap()).unwrap();
            for platform in ["windows", "osx"] {
                let f = Fixture::new();
                let first = &package.files[0].path;
                fs::write(f.game.join(first), b"original resource").unwrap();
                fs::write(f.game.join("save.dat"), b"save").unwrap();
                let dir = install(
                    &root,
                    "patch.json",
                    &f.game,
                    &f.storage,
                    game,
                    "he",
                    &package.version,
                    platform,
                )
                .unwrap();
                assert_eq!(fs::read_dir(dir.join("originals")).unwrap().count(), 1);
                fs::write(f.game.join("later.txt"), b"later").unwrap();
                uninstall(&dir, &f.game).unwrap();
                assert_eq!(fs::read(f.game.join(first)).unwrap(), b"original resource");
                assert_eq!(fs::read_dir(&f.game).unwrap().count(), 3);
            }
        }
    }
    #[test]
    #[ignore = "requires KITOR_UNDERTALE_ARCHIVE pointing to a supported original archive and Python 3"]
    fn real_undertale_archive_roundtrip_on_disposable_copy() {
        let original = PathBuf::from(std::env::var_os("KITOR_UNDERTALE_ARCHIVE").unwrap());
        let platform = if original.extension().unwrap() == "win" {
            "windows"
        } else {
            "osx"
        };
        let name = if platform == "windows" {
            "data.win"
        } else {
            "game.ios"
        };
        let f = Fixture::new();
        fs::copy(original, f.game.join(name)).unwrap();
        fs::write(f.game.join("music.ogg"), b"untouched").unwrap();
        let before = hash(&f.game.join(name)).unwrap();
        let root = Path::new("../../patches-workspace/undertale/he")
            .canonicalize()
            .unwrap();
        let package: Package =
            serde_json::from_slice(&fs::read(root.join("patch.json")).unwrap()).unwrap();
        let dir = install(
            &root,
            "patch.json",
            &f.game,
            &f.storage,
            "undertale",
            "he",
            &package.version,
            platform,
        )
        .unwrap();
        assert_eq!(fs::read_dir(dir.join("originals")).unwrap().count(), 1);
        assert_ne!(hash(&f.game.join(name)).unwrap(), before);
        assert_eq!(fs::read_dir(&f.game).unwrap().count(), 2);
        uninstall(&dir, &f.game).unwrap();
        assert_eq!(hash(&f.game.join(name)).unwrap(), before);
        assert!(f.game.join("music.ogg").exists());
    }
}
