//! Reusable native launch capabilities selected by trusted registry profiles.
use super::game_behavior::{
    LaunchMethod, LaunchProfile, LaunchRoot, LaunchWorkingDirectory, NativeCommand,
    NativePreparation,
};
use super::patch_lifecycle::safe_path;
use super::registry_service::Platform;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

pub fn native_command(profile: &LaunchProfile, root: &Path) -> Result<Option<Command>, String> {
    if profile.commands.is_empty() {
        return Ok(None);
    }
    let platform = Platform::current();
    let config = profile
        .commands
        .get(&platform)
        .ok_or("Native launch is not supported on this platform")?;
    command_for_platform(config, root, platform).map(Some)
}

fn command_root(
    config: &NativeCommand,
    root: &Path,
    platform: Platform,
) -> Result<PathBuf, String> {
    if config.root == LaunchRoot::GameDirectory {
        // A macOS bundle can contain Windows-named data executables, as in FNA.
        // Their presence must not make that edition launchable on Windows.
        if platform == Platform::Windows && root.ancestors().any(is_bundle) {
            return Err("A macOS app bundle cannot be launched as a Windows installation".into());
        }
        return Ok(root.to_path_buf());
    }
    if platform != Platform::MacOS {
        return Err("App-bundle roots require macOS".into());
    }
    // Detection may record the bundle itself or its Resources directory. Only
    // the containing .app is eligible; arbitrary parent paths are never accepted.
    if let Some(bundle) = root.ancestors().find(|path| is_bundle(path)) {
        let contents = safe_path(bundle, "Contents")?;
        let relative = root.strip_prefix(bundle).map_err(|e| e.to_string())?;
        if relative.as_os_str().is_empty() || root.starts_with(&contents) {
            return bundle.canonicalize().map_err(|e| e.to_string());
        }
    }
    let bundle = safe_path(
        root,
        config.bundle_name.as_deref().ok_or("Missing bundle name")?,
    )?;
    if !bundle.is_dir() {
        return Err("Native app bundle was not found".into());
    }
    bundle.canonicalize().map_err(|e| e.to_string())
}

fn is_bundle(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
}

fn regular_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = safe_path(root, relative)?;
    if !path.is_file() {
        return Err(format!("Required native game file not found: {relative}"));
    }
    Ok(path)
}

fn command_for_platform(
    config: &NativeCommand,
    root: &Path,
    platform: Platform,
) -> Result<Command, String> {
    config.validate_for_platform(platform)?;
    let root = command_root(config, root, platform)?;
    let program = regular_file(&root, &config.program)?;
    for path in &config.required_files {
        regular_file(&root, path)?;
    }
    for path in &config.required_directories {
        if !safe_path(&root, path)?.is_dir() {
            return Err(format!("Required native game directory not found: {path}"));
        }
    }
    for signature in &config.signatures {
        let path = regular_file(&root, &signature.path)?;
        let mut prefix = vec![0; signature.prefix.len()];
        let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
        file.read_exact(&mut prefix)
            .map_err(|e| format!("Could not check {}: {e}", signature.path))?;
        if prefix != signature.prefix {
            return Err(format!("Unsupported native game file: {}", signature.path));
        }
    }
    let working_directory = match config.working_directory {
        LaunchWorkingDirectory::Root => root.clone(),
        LaunchWorkingDirectory::BundleParent => root
            .parent()
            .ok_or("Invalid app bundle parent")?
            .to_path_buf(),
        LaunchWorkingDirectory::BundleResources => safe_path(&root, "Contents/Resources")?,
    };
    if !working_directory.is_dir() {
        return Err("Native working directory was not found".into());
    }
    // Validate the entire plan before any permission or settings mutation.
    let executable_files = config
        .executable_files
        .iter()
        .map(|path| regular_file(&root, path))
        .collect::<Result<Vec<_>, _>>()?;
    for path in executable_files {
        ensure_owner_execute(&path)?;
    }
    match config.preparation {
        Some(NativePreparation::GrimHebrewPreferences) => {
            super::games::grim_remastered::prepare_launch(&root)?
        }
        None => {}
    }
    let mut command = match config.method {
        LaunchMethod::Executable => Command::new(program),
        LaunchMethod::AppBundle => {
            if platform != Platform::MacOS {
                return Err("App-bundle launching requires macOS".into());
            }
            let mut command = Command::new("/usr/bin/open");
            command.arg(&root);
            if !config.args.is_empty() {
                command.arg("--args");
            }
            command
        }
    };
    command
        .current_dir(working_directory)
        .args(&config.args)
        .envs(&config.env);
    Ok(command)
}

fn ensure_owner_execute(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        if permissions.mode() & 0o100 == 0 {
            permissions.set_mode(permissions.mode() | 0o100);
            fs::set_permissions(path, permissions).map_err(|e| e.to_string())?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

#[cfg(test)]
#[path = "native_launch/tests.rs"]
mod tests;
