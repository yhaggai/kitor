//! Curated Windows depot downloads through the user's authenticated Steam client.
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

async fn download_layout(
    product: &SteamProduct,
    state: &crate::AppState,
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
    let home = std::env::var_os("HOME").ok_or("Cannot locate your home folder")?;
    let steam = PathBuf::from(home).join("Library/Application Support/Steam");
    let runtime = steam.join("Steam.AppBundle/Steam/Contents/MacOS");
    let executable = runtime.join("steam_osx");
    if !executable.is_file() {
        return Err("Install Steam and sign in, then try again.".into());
    }
    download_depot(&steam, &executable, &state.app_data_dir, product.app_id, product.depot_id).await?;
    let source = [
        runtime.join(format!("steamapps/content/app_{}/depot_{}", product.app_id, product.depot_id)),
        steam.join(format!("steamapps/content/app_{}/depot_{}", product.app_id, product.depot_id)),
    ]
    .into_iter()
    .find(|p| {
        product.required_files.iter().all(|name| p.join(name).is_file())
    })
    .ok_or("Steam reported completion, but the expected game files were not found")?;
    let games = state.app_data_dir.join("games");
    let destination = games.join(format!("{}-steam", recipe.id));
    let copy_paths = product.copy_paths.clone();
    let import = destination.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        std::fs::create_dir_all(&games).map_err(|e| e.to_string())?;
        if import.exists() { return Err("The managed game folder already exists. Add it to your library instead of overwriting it.".into()); }
        let staging = tempfile::tempdir_in(&games).map_err(|e| e.to_string())?;
        // Only Steam game content; never import separately installed patches or saves.
        for name in copy_paths {
            copy_checked(&source.join(&name), &staging.path().join(&name))?;
        }
        std::fs::rename(staging.path(), &import).map_err(|e| e.to_string())?;
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
    title: String,
    importer: Importer,
    cache_key: String,
    games: Vec<ImportRecipe>,
    #[serde(default)]
    required_files: Vec<String>,
    #[serde(default)]
    copy_paths: Vec<String>,
}
fn products() -> Vec<SteamProduct> {
    serde_json::from_str(include_str!("../../data/steam_products.json")).expect("reviewed Steam products")
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
    let product = products().into_iter().find(|p| p.app_id == app_id && p.games.iter().any(|g| g.id == game_id));
    match product {
        Some(product) => DownloadCapabilities { supported: cfg!(target_os = "macos"), title: product.title,
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
        copy_checked(&source.join(&file.name), &staging.path().join(&file.name))?;
    }
    verify_recipe(staging.path(), recipe)?;
    std::fs::rename(staging.path(), destination).map_err(|e| e.to_string())
}

/// A store product is a download source, not a Kitor game identity.
/// Serialize imports too, so two cards cannot replace one another's installation.
pub async fn download(
    app_id: u32,
    game_ids: Vec<String>,
    state: &crate::AppState,
) -> Result<Vec<crate::models::game::Model>, String> {
    if !cfg!(target_os = "macos") {
        return Err("This download flow requires macOS".into());
    }
    let _guard = DOWNLOAD.try_lock().map_err(|_| {
        "A Steam download/import is already running in Kitor. Reopen its progress dialog."
    })?;
    let product = products().into_iter().find(|p| p.app_id == app_id)
        .ok_or("Unsupported Steam product")?;
    if game_ids.is_empty() || game_ids.iter().any(|id| !product.games.iter().any(|g| &g.id == id)) {
        return Err("Unsupported game edition for this Steam product".into());
    }
    if matches!(product.importer, Importer::Layout) {
        if product.games.len() != 1 || game_ids.len() != 1 { return Err("Layout imports require one game".into()); }
        return Ok(vec![download_layout(&product, state).await?]);
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
    let steam = PathBuf::from(std::env::var_os("HOME").ok_or("Cannot locate home folder")?)
        .join("Library/Application Support/Steam");
    let runtime = steam.join("Steam.AppBundle/Steam/Contents/MacOS");
    let cache = state
        .app_data_dir
        .join("steam-sources").join(&product.cache_key);
    if !needed
        .iter()
        .all(|r| verify_recipe(&cache.join(&r.id), r).is_ok())
    {
        let candidates = [
            runtime.join(format!("steamapps/content/app_{}/depot_{}", product.app_id, product.depot_id)),
            steam.join(format!("steamapps/content/app_{}/depot_{}", product.app_id, product.depot_id)),
        ];
        let valid_source = || {
            candidates
                .iter()
                .find(|p| {
                    all.iter()
                        .all(|r| verify_recipe(&p.join(&r.folder), r).is_ok())
                })
                .cloned()
        };
        // Exact known bytes can be reused even without an appmanifest or old console logs.
        let source = if let Some(source) = valid_source() {
            source
        } else {
            let executable = [
                runtime.join("steam_osx"),
                PathBuf::from("/Applications/Steam.app/Contents/MacOS/steam_osx"),
            ]
            .into_iter()
            .find(|p| p.is_file())
            .ok_or("Install Steam and sign in first")?;
            download_depot(&steam, &executable, &state.app_data_dir, product.app_id, product.depot_id).await?;
            valid_source().ok_or("Steam completed, but this collection's files do not match the verified edition. No games were imported.")?
        };
        for recipe in all {
            let destination = cache.join(&recipe.id);
            if verify_recipe(&destination, recipe).is_ok() {
                continue;
            }
            import_recipe(&source.join(&recipe.folder), &destination, recipe)?;
        }
    }
    for recipe in needed {
        let destination = state
            .app_data_dir
            .join("games")
            .join(format!("{}-steam", recipe.id));
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
