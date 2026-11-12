//! Curated game depot downloads through the user's authenticated Steam client.
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::Mutex;
static DOWNLOAD: Mutex<()> = Mutex::const_new(());

fn completion(line: &str, app: u32, depot: u32) -> bool {
    line.contains("Depot download complete :")
        && line
            .replace('\\', "/")
            .contains(&format!("/app_{app}/depot_{depot}\""))
}
fn copy_checked(source: &Path, dest: &Path) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(source).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Steam data contains a symbolic link".into());
    }
    if meta.is_dir() {
        std::fs::create_dir(dest).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            copy_checked(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        std::fs::copy(source, dest).map_err(|e| e.to_string())?;
        fn hash(path: &Path) -> Result<Vec<u8>, String> {
            let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut h = Sha256::new();
            let mut buf = [0u8; 65536];
            loop {
                let n = file.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                h.update(&buf[..n]);
            }
            Ok(h.finalize().to_vec())
        }
        if hash(source)? != hash(dest)? {
            return Err("Imported file verification failed".into());
        }
    } else {
        return Err("Unexpected Steam file type".into());
    }
    Ok(())
}

// Only consume Kitor-owned temporary content after Steam verification succeeds.
// External client installations and reusable caches must remain untouched.
fn transfer_verified(source: &Path, dest: &Path, consume: bool) -> Result<(), String> {
    if consume {
        reject_links(source)?;
        if dest.exists() {
            return Err(format!("Import destination already exists: {}", dest.display()));
        }
        match std::fs::rename(source, dest) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::CrossesDevices => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    copy_checked(source, dest)
}

async fn native_source(
    product: &SteamProduct,
    state: &crate::AppState,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<Option<(tempfile::TempDir, PathBuf)>, String> {
    native_depot_source(product.app_id, product.depot_id, state, progress_app).await
}

async fn native_depot_source(
    app_id: u32,
    depot_id: u32,
    state: &crate::AppState,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<Option<(tempfile::TempDir, PathBuf)>, String> {
    if !kitor_steam::connected().await.map_err(|e| e.to_string())? { return Ok(None); }
    let staging = tempfile::tempdir_in(&state.app_data_dir).map_err(|e| e.to_string())?;
    let source = staging.path().join("content");
    kitor_steam::download(app_id, depot_id, &source, |event| {
        if let Some(app) = progress_app {
            use tauri::Emitter;
            let _ = app.emit("native-steam-download", &event);
        }
    }).await.map_err(|e| e.to_string())?;
    Ok(Some((staging, source)))
}

fn depot_candidates(steam: &Path, executable: &Path, product: &SteamProduct) -> Vec<PathBuf> {
    depot_paths(steam, executable, product.app_id, product.depot_id)
}

fn depot_paths(steam: &Path, executable: &Path, app_id: u32, depot_id: u32) -> Vec<PathBuf> {
    let relative = format!("steamapps/content/app_{app_id}/depot_{depot_id}");
    let mut candidates = vec![steam.join(&relative)];
    if let Some(runtime) = executable.parent() {
        let candidate = runtime.join(relative);
        if !candidates.contains(&candidate) {
            candidates.insert(0, candidate);
        }
    }
    candidates
}

async fn download_collection(
    product: &SteamProduct,
    state: &crate::AppState,
    install_root: &Path,
    overwrite: bool,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<Vec<crate::models::game::Model>, String> {
    reject_links(install_root)?;
    let destination = install_root.join(format!("steam-{}", product.app_id));
    reject_links(&destination)?;
    if destination.exists() && !overwrite {
        return Err(format!("OVERWRITE_REQUIRED:{}", destination.display()));
    }
    {
        let native_staging = native_source(product, state, progress_app).await?;
        let source = if let Some((_, source)) = &native_staging {
            for recipe in &product.games { verify_recipe(&source.join(&recipe.folder), recipe)?; }
            source.clone()
        } else {
            let (steam, executable) = layout_steam_client()?;
            let candidates = depot_candidates(&steam, &executable, product);
            let valid_source = || candidates.iter().find(|path| product.games.iter()
                .all(|recipe| verify_recipe(&path.join(&recipe.folder), recipe).is_ok())).cloned();
            if let Some(source) = valid_source() {
                source
            } else {
                download_depot(&steam, &executable, &state.app_data_dir, product.app_id, product.depot_id).await?;
                valid_source().ok_or("Steam completed, but the collection files could not be verified")?
            }
        };
        if let Some(app) = progress_app {
            use tauri::Emitter;
            for recipe in &product.games {
                let _ = app.emit("steam-game-importing", &recipe.id);
            }
        }
        let consume = native_staging.is_some();
        let root = install_root.to_path_buf();
        let target = destination.clone();
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
            let staging = tempfile::tempdir_in(&root).map_err(|e| e.to_string())?;
            let content = staging.path().join("collection");
            transfer_verified(&source, &content, consume)?;
            // Finish and verify the replacement before touching the existing installation.
            let previous = staging.path().join("previous");
            let replacing = target.exists();
            if replacing && !overwrite {
                return Err(format!("OVERWRITE_REQUIRED:{}", target.display()));
            }
            if replacing {
                std::fs::rename(&target, &previous).map_err(|e| e.to_string())?;
            }
            if let Err(error) = std::fs::rename(content, &target) {
                if replacing {
                    if let Err(restore) = std::fs::rename(&previous, &target) {
                        let retained = staging.keep();
                        return Err(format!("Replacement failed: {error}; restore failed: {restore}. Original files retained at {}", retained.join("previous").display()));
                    }
                }
                return Err(error.to_string());
            }
            Ok(())
        }).await.map_err(|e| e.to_string())??;
    }
    // Detect intact collection contents, including games outside the download recipes.
    // Existing imports may contain saves and patches and must never be overwritten.
    let mut pending = vec![destination];
    let mut games = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() { continue; }
        games.extend(super::game_service::scan_directory(
            &state.db, &path.to_string_lossy(), state.app_data_dir.clone(),
        ).await?);
        for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
            pending.push(entry.map_err(|e| e.to_string())?.path());
        }
    }
    Ok(games)
}

fn layout_steam_client() -> Result<(PathBuf, PathBuf), String> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").ok_or("Cannot locate your home folder")?;
        let steam = PathBuf::from(home).join("Library/Application Support/Steam");
        let executables = [
            steam.join("Steam.AppBundle/Steam/Contents/MacOS/steam_osx"),
            PathBuf::from("/Applications/Steam.app/Contents/MacOS/steam_osx"),
        ];
        if let Some(executable) = executables.into_iter().find(|path| path.is_file()) {
            return Ok((steam, executable));
        }
    }
    #[cfg(target_os = "windows")]
    {
        let roots = super::game_discovery_service::registry_paths(
            r"HKCU\Software\Valve\Steam", "SteamPath", false,
        );
        for steam in roots.into_iter().chain(
            ["ProgramFiles(x86)", "ProgramFiles"].into_iter()
                .filter_map(std::env::var_os)
                .map(|root| PathBuf::from(root).join("Steam")),
        ) {
            let executable = steam.join("steam.exe");
            if steam.is_absolute() && executable.is_file() {
                return Ok((steam, executable));
            }
        }
    }
    Err("Install Steam and sign in, then try again.".into())
}

// Shared and platform depots may contain the same directories. Reject conflicting
// files so a partial or mismatched build never overwrites another depot's files.
fn merge_depot(source: &Path, destination: &Path, consume: bool) -> Result<(), String> {
    reject_links(source)?;
    for entry in std::fs::read_dir(source).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if !to.exists() {
            if kind.is_dir() {
                std::fs::create_dir(&to).map_err(|error| error.to_string())?;
                merge_depot(&from, &to, consume)?;
            } else if kind.is_file() {
                transfer_verified(&from, &to, consume)?;
            } else {
                return Err(format!("Unexpected Steam depot file type: {}", from.display()));
            }
        } else if kind.is_dir() && to.is_dir() {
            merge_depot(&from, &to, consume)?;
        } else if !kind.is_file() || !to.is_file()
            || super::patch_lifecycle::hash(&from)? != super::patch_lifecycle::hash(&to)?
        {
            return Err(format!("Conflicting Steam depot file: {}", to.display()));
        }
    }
    Ok(())
}

async fn download_depot_set(
    product: &SteamProduct,
    state: &crate::AppState,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<(tempfile::TempDir, PathBuf), String> {
    let staging = tempfile::tempdir_in(&state.app_data_dir).map_err(|error| error.to_string())?;
    let destination = staging.path().join("content");
    std::fs::create_dir(&destination).map_err(|error| error.to_string())?;
    for depot in std::iter::once(product.depot_id).chain(product.additional_depots.iter().copied()) {
        let native = native_depot_source(product.app_id, depot, state, progress_app).await?;
        let source = if let Some((_, source)) = &native {
            source.clone()
        } else {
            let (steam, executable) = layout_steam_client()?;
            download_depot(&steam, &executable, &state.app_data_dir, product.app_id, depot).await?;
            depot_paths(&steam, &executable, product.app_id, depot)
                .into_iter()
                .find(|path| path.is_dir())
                .ok_or_else(|| format!("Steam did not provide depot {depot}"))?
        };
        let target = destination.clone();
        let consume = native.is_some();
        tokio::task::spawn_blocking(move || merge_depot(&source, &target, consume))
            .await.map_err(|error| error.to_string())??;
    }
    Ok((staging, destination))
}

async fn download_layout(
    product: &SteamProduct,
    state: &crate::AppState,
    install_root: &Path,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<crate::models::game::Model, String> {
    let recipe = product.games.first().ok_or("Missing import recipe")?;
    // Never move an existing installation out from under its patch backups or saves.
    if let Some(game) = super::game_service::get_all_games(&state.db)
        .await?
        .into_iter()
        .find(|g| g.scummvm_id == recipe.id)
    {
        if Path::new(&game.install_path).is_dir() {
            return Ok(game);
        }
        return Err("This game's registered folder is missing. Remove that library entry before downloading again.".into());
    }
    let native_staging = if product.additional_depots.is_empty() {
        native_source(product, state, progress_app).await?
    } else {
        Some(download_depot_set(product, state, progress_app).await?)
    };
    let source = if let Some((_, source)) = &native_staging {
        if !product.required_files.iter().all(|name| source.join(name).is_file()) {
            return Err("Downloaded Steam files do not match the supported game edition".into());
        }
        source.clone()
    } else {
        let (steam, executable) = layout_steam_client()?;
        download_depot(&steam, &executable, &state.app_data_dir, product.app_id, product.depot_id).await?;
        depot_candidates(&steam, &executable, product)
        .into_iter()
        .find(|p| {
            product.required_files.iter().all(|name| p.join(name).is_file())
        })
        .ok_or("Steam reported completion, but the expected game files were not found")?

    };
    if let Some(app) = progress_app {
        use tauri::Emitter;
        let _ = app.emit("steam-game-importing", &recipe.id);
    }
    let consume = native_staging.is_some();
    let games = install_root.to_path_buf();
    let destination = games.join(format!("{}-steam", recipe.id));
    let copy_paths = product.copy_paths.clone();
    let copy_depot = product.copy_depot;
    let lowercase_copy_paths = product.lowercase_copy_paths;
    #[cfg(unix)]
    let executable_paths = product.executable_paths.clone();
    let game_id = recipe.id.clone();
    let import = destination.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        std::fs::create_dir_all(&games).map_err(|e| e.to_string())?;
        if import.exists() { return Err("The managed game folder already exists. Add it to your library instead of overwriting it.".into()); }
        let staging = tempfile::tempdir_in(&games).map_err(|e| e.to_string())?;
        let staged_game = staging.path().join("game");
        if copy_depot {
            transfer_verified(&source, &staged_game, consume)?;
        } else {
            std::fs::create_dir(&staged_game).map_err(|error| error.to_string())?;
            for name in copy_paths {
                let destination_name = if lowercase_copy_paths {
                    name.to_ascii_lowercase()
                } else {
                    name.clone()
                };
                transfer_verified(&source.join(&name), &staged_game.join(destination_name), consume)?;
            }
        }
        #[cfg(unix)]
        for path in &executable_paths {
            restore_owner_execute(&staged_game, path)?;
        }
        if super::game_service::detect_game_for_id(&staged_game, &game_id)?.is_none() {
            return Err("Steam files do not match the supported game edition.".into());
        }
        if game_id == "owlboy" {
            // Validate the platform and prepare the native bundle's executable
            // permission before publishing the installation or applying a patch.
            super::games::owlboy::launch_command(&staged_game)?;
        }
        std::fs::rename(&staged_game, &import).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())??;
    super::game_service::add_game_manually(
        &state.db,
        recipe.id.clone(),
        recipe.title.clone(),
        destination.to_string_lossy().into_owned(),
        state.app_data_dir.clone(),
    )
    .await
}

// Some depots omit the executable flag on a native launcher. Repair only
// reviewed launcher paths inside the staged install before publishing it.
#[cfg(unix)]
fn restore_owner_execute(root: &Path, relative: &str) -> Result<(), String> {
    use std::{os::unix::fs::PermissionsExt, path::Component};

    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || !relative.components().all(|part| matches!(part, Component::Normal(_)))
    {
        return Err("Invalid executable path in Steam recipe".into());
    }
    let path = root.join(relative);
    reject_links(&path)?;
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| format!("Could not read Steam launcher {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("Steam launcher is not a regular file: {}", path.display()));
    }
    let mut permissions = metadata.permissions();
    if permissions.mode() & 0o100 == 0 {
        permissions.set_mode(permissions.mode() | 0o100);
        std::fs::set_permissions(&path, permissions)
            .map_err(|error| format!("Could not make {} executable: {error}", path.display()))?;
    }
    Ok(())
}

fn log_identity(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.ino()
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        0
    }
}

async fn download_depot(
    steam: &Path,
    executable: &Path,
    progress_dir: &Path,
    app: u32,
    depot: u32,
) -> Result<(), String> {
    let log = steam.join("logs/console_log.txt");
    std::fs::create_dir_all(progress_dir).map_err(|e| e.to_string())?;
    let request = progress_dir.join(format!("steam-download-{app}-{depot}.json"));
    // Keep the original offset after a timeout/restart. Retrying attaches to the
    // same Steam request instead of issuing a second in-flight download.
    let (mut offset, mut identity): (u64, u64) = if request.exists() {
        serde_json::from_slice(&std::fs::read(&request).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Cannot read pending Steam request: {e}"))?
    } else {
        let initial = std::fs::metadata(&log)
            .map(|m| (m.len(), log_identity(&m)))
            .unwrap_or((0, 0));
        std::fs::write(
            &request,
            serde_json::to_vec(&initial).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        match tokio::process::Command::new(executable)
            .args([
                "-console",
                "+download_depot",
                &app.to_string(),
                &depot.to_string(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait().await;
                });
            }
            Err(e) => {
                let _ = std::fs::remove_file(&request);
                return Err(format!("Could not start Steam: {e}"));
            }
        }
        initial
    };
    let mut pending = String::new();
    let mut complete = false;
    for _ in 0..1800 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if let Ok(mut file) = std::fs::File::open(&log) {
            let metadata = file.metadata().map_err(|e| e.to_string())?;
            let length = metadata.len();
            let current_identity = log_identity(&metadata);
            if length < offset || current_identity != identity {
                identity = current_identity;
                offset = 0;
                pending.clear();
            }
            file.seek(SeekFrom::Start(offset))
                .map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            file.take(1024 * 1024)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            offset += bytes.len() as u64;
            pending.push_str(&String::from_utf8_lossy(&bytes));
            while let Some(end) = pending.find('\n') {
                let line: String = pending.drain(..=end).collect();
                if completion(&line, app, depot) {
                    complete = true;
                    break;
                }
                if line.contains(&depot.to_string())
                    && (line.contains("Depot download failed")
                        || line.contains("not owned")
                        || line.contains("Access Denied"))
                {
                    let _ = std::fs::remove_file(&request);
                    return Err("Steam could not download this game. Check that Steam is online and signed into an account that owns this collection or game, then retry.".into());
                }
            }
            if pending.len() > 1024 * 1024 {
                return Err("Unexpected Steam log format. Check the Steam console.".into());
            }
        }
        if complete {
            break;
        }
    }
    if !complete {
        return Err("Timed out waiting for Steam. The download may still be running in Steam; check its console before retrying.".into());
    }
    std::fs::remove_file(&request).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_os = "macos"))]
mod live_test;

#[derive(serde::Deserialize)]
struct ImportFile {
    name: String,
    sha256: String,
}
#[derive(serde::Deserialize)]
struct ImportRecipe {
    id: String,
    title: String,
    folder: String,
    files: Vec<ImportFile>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Importer { Layout, VerifiedFiles }
#[derive(serde::Deserialize)]
struct SteamProduct {
    app_id: u32,
    depot_id: u32,
    #[serde(default)]
    additional_depots: Vec<u32>,
    #[serde(default)]
    platform: Option<String>,
    title: String,
    importer: Importer,
    cache_key: String,
    games: Vec<ImportRecipe>,
    #[serde(default)]
    required_files: Vec<String>,
    #[serde(default)]
    copy_paths: Vec<String>,
    #[serde(default)]
    lowercase_copy_paths: bool,
    #[serde(default)]
    copy_depot: bool,
    #[cfg(unix)]
    #[serde(default)]
    executable_paths: Vec<String>,
}
impl SteamProduct {
    fn supports_current_platform(&self) -> bool {
        match self.platform.as_deref() {
            Some("windows") => cfg!(target_os = "windows"),
            Some("desktop") => cfg!(any(target_os = "macos", target_os = "windows")),
            None => cfg!(target_os = "macos"),
            _ => false,
        }
    }
}
fn products() -> Vec<SteamProduct> {
    serde_json::from_str(include_str!("../../data/steam_products.json")).expect("reviewed Steam products")
}
// Steam libraries list collections as one installation. Probe the curated
// game subdirectories, then let each game's detector verify its own files.
pub(super) fn collection_game_folders(root: &Path) -> Vec<PathBuf> {
    use std::{collections::HashMap, sync::OnceLock};
    static FOLDERS: OnceLock<HashMap<String, Vec<PathBuf>>> = OnceLock::new();
    let Some(name) = root.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    let collections = FOLDERS.get_or_init(|| products().into_iter()
        .filter(|p| matches!(p.importer, Importer::VerifiedFiles))
        .map(|p| {
            let mut folders = Vec::new();
            for recipe in p.games {
                let folder = PathBuf::from(recipe.folder);
                folders.push(folder.clone());
                // Earlier Space Quest Collection releases install sq2-sq6
                // directly below the collection. Prefer the 2016 copy when both exist.
                if let Ok(legacy) = folder.strip_prefix("2016_SpaceQuestCollection") {
                    folders.push(legacy.to_owned());
                }
            }
            (p.title.to_ascii_lowercase(), folders)
        }).collect());
    collections.get(&name.to_ascii_lowercase()).into_iter().flatten()
        .map(|folder| root.join(folder))
        .filter(|path| path.is_dir())
        .collect()
}

#[cfg(test)]
fn recipes() -> Vec<ImportRecipe> {
    products().into_iter().find(|p| matches!(p.importer, Importer::VerifiedFiles)).unwrap().games
}
#[derive(serde::Serialize)]
pub struct DownloadCapabilities {
    supported: bool,
    title: String,
    games: Vec<DownloadGame>,
}
#[derive(serde::Serialize)]
struct DownloadGame { id: String, title: String }
pub fn capabilities(app_id: u32, game_id: &str) -> DownloadCapabilities {
    let product = products().into_iter().find(|p| p.app_id == app_id && p.supports_current_platform() && p.games.iter().any(|g| g.id == game_id));
    match product {
        Some(product) => DownloadCapabilities { supported: product.supports_current_platform(), title: product.title,
            games: product.games.into_iter().map(|g| DownloadGame { id: g.id, title: g.title }).collect() },
        None => DownloadCapabilities { supported: false, title: String::new(), games: Vec::new() },
    }
}
fn reject_links(path: &Path) -> Result<(), String> {
    // Resolve the system temp/home prefix first at call sites where necessary;
    // explicit source components must never redirect outside the Steam tree.
    for ancestor in path.ancestors().take_while(|p| p.parent().is_some()) {
        if std::fs::symlink_metadata(ancestor)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(format!(
                "Symbolic link in game path: {}",
                ancestor.display()
            ));
        }
    }
    Ok(())
}
fn verify_recipe(root: &Path, recipe: &ImportRecipe) -> Result<(), String> {
    reject_links(root)?;
    for file in &recipe.files {
        let path = root.join(&file.name);
        let meta =
            std::fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() == 0 {
            return Err(format!("Invalid game file: {}", path.display()));
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        if format!("{:x}", Sha256::digest(bytes)) != file.sha256 {
            return Err(format!(
                "Unverified edition or modified file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}
fn import_recipe(source: &Path, destination: &Path, recipe: &ImportRecipe) -> Result<(), String> {
    import_recipe_transfer(source, destination, recipe, false)
}

fn import_recipe_transfer(source: &Path, destination: &Path, recipe: &ImportRecipe, consume: bool) -> Result<(), String> {
    verify_recipe(source, recipe)?;
    reject_links(destination)?;
    let parent = destination.parent().ok_or("Missing import parent")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    if destination.exists() {
        return Err(format!(
            "{} already exists; add it manually instead of overwriting it",
            destination.display()
        ));
    }
    let staging = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    for file in &recipe.files {
        transfer_verified(&source.join(&file.name), &staging.path().join(&file.name), consume)?;
    }
    if !consume { verify_recipe(staging.path(), recipe)?; }
    std::fs::rename(staging.path(), destination).map_err(|e| e.to_string())
}

/// A store product is a download source, not a Kitor game identity.
/// Serialize imports too, so two cards cannot replace one another's installation.
#[cfg(test)]
pub async fn download(
    app_id: u32,
    game_ids: Vec<String>,
    state: &crate::AppState,
) -> Result<Vec<crate::models::game::Model>, String> {
    download_to(app_id, game_ids, state, None).await
}

#[cfg(test)]
pub async fn download_to(
    app_id: u32,
    game_ids: Vec<String>,
    state: &crate::AppState,
    install_root: Option<PathBuf>,
) -> Result<Vec<crate::models::game::Model>, String> {
    download_to_confirmed(app_id, game_ids, state, install_root, false, None).await
}

pub async fn download_to_confirmed(
    app_id: u32,
    game_ids: Vec<String>,
    state: &crate::AppState,
    install_root: Option<PathBuf>,
    overwrite: bool,
    progress_app: Option<&tauri::AppHandle>,
) -> Result<Vec<crate::models::game::Model>, String> {
    let install_root = install_root.unwrap_or_else(|| state.app_data_dir.join("games"));
    if !install_root.is_absolute() { return Err("Choose an absolute installation folder".into()); }
    let _guard = DOWNLOAD.try_lock().map_err(|_| {
        "A Steam download/import is already running in Kitor. Reopen its progress dialog."
    })?;
    let product = products()
        .into_iter()
        .find(|product| {
            product.app_id == app_id
                && product.supports_current_platform()
                && !game_ids.is_empty()
                && game_ids.iter().all(|id| product.games.iter().any(|game| &game.id == id))
        })
        .ok_or("Unsupported game edition for this Steam product")?;
    if product.games.len() > 1 {
        return download_collection(&product, state, &install_root, overwrite, progress_app).await;
    }
    if matches!(product.importer, Importer::Layout) {
        if product.games.len() != 1 || game_ids.len() != 1 { return Err("Layout imports require one game".into()); }
        return Ok(vec![download_layout(&product, state, &install_root, progress_app).await?]);
    }
    let all = &product.games;
    let existing = super::game_service::get_all_games(&state.db).await?;
    let mut result = Vec::new();
    let mut needed = Vec::new();
    for recipe in all.iter().filter(|r| game_ids.contains(&r.id)) {
        if let Some(game) = existing.iter().find(|g| g.scummvm_id == recipe.id) {
            if !Path::new(&game.install_path).is_dir() {
                return Err(format!(
                    "{} has a missing registered folder. Repair or remove its library entry first.",
                    recipe.title
                ));
            }
            result.push(game.clone());
        } else {
            needed.push(recipe);
        }
    }
    if needed.is_empty() {
        return Ok(result);
    }
    let cache = state
        .app_data_dir
        .join("steam-sources").join(&product.cache_key);
    if !needed
        .iter()
        .all(|r| verify_recipe(&cache.join(&r.id), r).is_ok())
    {
        // Exact known bytes can be reused even without an appmanifest or old console logs.
        let native_staging = native_source(&product, state, progress_app).await?;
        let source = if let Some((_, source)) = &native_staging {
            for recipe in all { verify_recipe(&source.join(&recipe.folder), recipe)?; }
            source.clone()
        } else {
            let (steam, executable) = layout_steam_client()?;
            let candidates = depot_candidates(&steam, &executable, &product);
            let valid_source = || candidates.iter().find(|path| all.iter()
                .all(|recipe| verify_recipe(&path.join(&recipe.folder), recipe).is_ok())).cloned();
            if let Some(source) = valid_source() {
                source
            } else {
                download_depot(&steam, &executable, &state.app_data_dir, product.app_id, product.depot_id).await?;
                valid_source().ok_or("Steam completed, but the files do not match the verified edition. No games were imported.")?
            }
        };
        for recipe in all {
            let destination = cache.join(&recipe.id);
            if verify_recipe(&destination, recipe).is_ok() {
                continue;
            }
            import_recipe_transfer(&source.join(&recipe.folder), &destination, recipe, native_staging.is_some())?;
        }
    }
    for recipe in needed {
        let destination = install_root.join(format!("{}-steam", recipe.id));
        // A previous registration failure leaves a pristine, recoverable import.
        if !destination.exists() {
            import_recipe(&cache.join(&recipe.id), &destination, recipe)?;
        } else {
            verify_recipe(&destination, recipe).map_err(|_| format!("{} already exists and differs from the original. Add it manually; Kitor will not overwrite saves or patches.", destination.display()))?;
        }
        let game = super::game_service::add_game_manually(
            &state.db,
            recipe.id.clone(),
            recipe.title.clone(),
            destination.to_string_lossy().into_owned(),
            state.app_data_dir.clone(),
        )
        .await
        .map_err(|e| {
            format!(
                "Files retained at {}. Add this folder manually if retry fails: {e}",
                destination.display()
            )
        })?;
        result.push(game);
    }
    Ok(result)
}

#[cfg(test)]
mod collection_tests;

#[cfg(all(test, target_os = "macos"))]
mod manual_import;

#[cfg(test)]
mod capability_tests;
