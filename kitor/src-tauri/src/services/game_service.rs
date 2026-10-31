use sea_orm::*;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::models::game::{self, Entity as Game};
use crate::services::registry_service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPatchInfo {
    pub language: Option<String>,
    pub release_revision: u64,
    pub id: i32,
    pub scummvm_id: String,
    pub version: String,
    pub installed_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePatchInfo {
    pub release_revision: u64,
    pub language: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWithPatches {
    pub id: i32,
    pub scummvm_id: String,
    pub title: String,
    pub install_path: String,
    pub cover_url: Option<String>,
    pub detected_version: Option<String>,
    pub added_date: String,
    pub last_played: Option<String>,
    pub requires_scummvm: bool,
    pub is_patched: bool,
    pub health: Option<super::game_refresh_service::GameHealth>,
    pub has_available_patch: bool,
    pub installed_patches: Vec<InstalledPatchInfo>,
    pub available_patches: Vec<AvailablePatchInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameInfo {
    pub install_path: String,
    pub scummvm_id: String,
    pub title: String,
    pub available_patches_count: usize,
}


pub async fn get_all_games(db: &DatabaseConnection) -> Result<Vec<game::Model>, String> {
    Game::find()
        .all(db)
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))
}

pub async fn get_game_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<game::Model>, String> {
    Game::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch game: {}", e))
}

pub async fn scan_directory(
    db: &DatabaseConnection,
    scan_path: &str,
    _app_data_dir: PathBuf,
) -> Result<Vec<game::Model>, String> {
    let path = Path::new(scan_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    let mut games = Vec::new();

    // A selected game/app is itself a candidate; otherwise scan its children.
    let candidates = if detect_scummvm_game(path).is_some() {
        vec![path.to_path_buf()]
    } else {
        std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read entry: {}", e))?
    };

    for entry_path in candidates {

        if entry_path.is_dir() {
            for game_info in detect_library_games(&entry_path) {
                // Check if game already exists
                let existing = Game::find()
                    .filter(game::Column::ScummvmId.eq(&game_info.scummvm_id))
                    .one(db)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?;

                if existing.is_none() {
                    let (launch_subdirectory, launch_args, cover_url) =
                        get_game_metadata_from_registry(&game_info.scummvm_id)?;
                    
                    let game = game::ActiveModel {
                        scummvm_id: Set(game_info.scummvm_id),
                        title: Set(game_info.title),
                        install_path: Set(game_info.install_path),
                        cover_url: Set(cover_url),
                        detected_version: Set(None),
                        added_date: Set(chrono::Utc::now().naive_utc()),
                        last_played: Set(None),
                        launch_subdirectory: Set(launch_subdirectory),
                        launch_args: Set(launch_args),
                        ..Default::default()
                    };

                    let result = game.insert(db).await
                        .map_err(|e| format!("Failed to insert game: {}", e))?;

                    games.push(result);
                }
            }
        }
    }

    Ok(games)
}

pub async fn add_game_manually(
    db: &DatabaseConnection,
    scummvm_id: String,
    title: String,
    install_path: String,
    _app_data_dir: PathBuf,
) -> Result<game::Model, String> {
    let install_path = detect_game_for_id(Path::new(&install_path), &scummvm_id)?
        .filter(|detected| detected.scummvm_id == scummvm_id)
        .map(|detected| detected.install_path)
        .unwrap_or(install_path);

    // Idempotent add: if game exists (unique scummvm_id), update metadata + return it.
    if let Some(existing) = Game::find()
        .filter(game::Column::ScummvmId.eq(&scummvm_id))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
    {
        let mut active: game::ActiveModel = existing.into();
        active.title = Set(title);
        active.install_path = Set(install_path);

        let (subdirectory, args, cover_url) = get_game_metadata_from_registry(&scummvm_id)?;
        active.launch_subdirectory = Set(subdirectory);
        active.launch_args = Set(args);
        if cover_url.is_some() {
            active.cover_url = Set(cover_url);
        }

        return active
            .update(db)
            .await
            .map_err(|e| format!("Failed to update existing game: {}", e));
    }

    let (launch_subdirectory, launch_args, cover_url) =
        get_game_metadata_from_registry(&scummvm_id)?;

    let game = game::ActiveModel {
        scummvm_id: Set(scummvm_id),
        title: Set(title),
        install_path: Set(install_path),
        cover_url: Set(cover_url),
        detected_version: Set(None),
        added_date: Set(chrono::Utc::now().naive_utc()),
        last_played: Set(None),
        launch_subdirectory: Set(launch_subdirectory),
        launch_args: Set(launch_args),
        ..Default::default()
    };

    let result = game.insert(db)
        .await
        .map_err(|e| format!("Failed to add game: {}", e))?;

    Ok(result)
}

pub async fn remove_game(db: &DatabaseConnection, id: i32) -> Result<(), String> {
    Game::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to remove game: {}", e))?;
    
    Ok(())
}

struct GameInfo {
    install_path: String,
    scummvm_id: String,
    title: String,
}

fn get_game_metadata_from_registry(
    scummvm_id: &str,
) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let registry = registry_service::load_registry()?;
    
    if let Some(game_def) = registry.games.iter().find(|g| g.scummvm_id == scummvm_id) {
        let cover_url = game_def.cover_url.clone();

        let profile = game_def.behavior.launch_profile(true);
        let args = if profile.args.is_empty() { None } else {
            Some(serde_json::to_string(&profile.args).map_err(|e| e.to_string())?)
        };
        return Ok((profile.subdirectory, args, cover_url));
    }
    
    Ok((None, None, None))
}

// AGI/SCI filenames are shared across titles. Fingerprints identify the
// supported Steam editions and patched AGI directories; SCI patches leave
// RESOURCE.MAP intact. Values come from the import recipes and patch payloads.
fn detected(def: &registry_service::GameDefinition, path: &Path) -> Option<DetectedGameInfo> {
    super::game_adapters::detect(def, path).map(|root| DetectedGameInfo {
        install_path: root.to_string_lossy().into_owned(), scummvm_id: def.scummvm_id.clone(),
        title: def.title.clone(), available_patches_count: def.patches.len(),
    })
}

pub fn detect_game_in_folder(path: &Path) -> Result<Option<DetectedGameInfo>, String> {
    if !path.is_dir() { return Err("Invalid directory path".into()); }
    let mut registry = registry_service::load_registry()?;
    registry.games.sort_by_key(|g| std::cmp::Reverse(g.behavior.detection.priority));
    Ok(registry.games.iter().find_map(|def| detected(def, path)))
}

pub fn detect_game_for_id(path: &Path, id: &str) -> Result<Option<DetectedGameInfo>, String> {
    let registry = registry_service::load_registry()?;
    Ok(registry.games.iter().find(|g| g.scummvm_id == id).and_then(|def| detected(def, path)))
}
fn detect_library_games(path: &Path) -> Vec<GameInfo> {
    let Ok(mut registry) = registry_service::load_registry() else { return Vec::new(); };
    registry.games.sort_by_key(|g| std::cmp::Reverse(g.behavior.detection.priority));
    // A native installation may expose several independently patchable editions.
    for def in &registry.games {
        if !def.behavior.library_editions.is_empty() && super::game_adapters::detect(def, path).is_some() {
            return def.behavior.library_editions.iter().filter_map(|id|
                registry.games.iter().find(|g| &g.scummvm_id == id).and_then(|g| detected(g, path)))
                .map(|d| GameInfo { install_path: d.install_path, scummvm_id: d.scummvm_id, title: d.title }).collect();
        }
    }
    registry.games.iter().find_map(|g| detected(g, path)).into_iter()
        .map(|d| GameInfo { install_path: d.install_path, scummvm_id: d.scummvm_id, title: d.title }).collect()
}

fn detect_scummvm_game(path: &Path) -> Option<GameInfo> {
    let detected = detect_game_in_folder(path).ok()??;
    Some(GameInfo {
        install_path: detected.install_path,
        scummvm_id: detected.scummvm_id,
        title: detected.title,
    })
}

pub async fn get_games_with_patches(
    db: &DatabaseConnection,
    app_data_dir: PathBuf,
) -> Result<Vec<GameWithPatches>, String> {
    // Get all games from DB
    let games = Game::find()
        .all(db)
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))?;
    
    // Load registry
    let registry = registry_service::load_registry()?;

    
    let mut games_with_patches = Vec::new();
    
    for game in games {
        // Find game definition in registry
        let game_def = registry.games.iter()
            .find(|g| g.scummvm_id == game.scummvm_id);
        
        // Get installed patches for this game
        let behavior = game_def.map(|def| def.behavior.clone()).unwrap_or_default();
        let state = super::patch_state::resolve(db, &game, &app_data_dir, &behavior).await?;
        let patched = state.is_patched();
        let installed = state.recorded;
        
        let installed_patches: Vec<InstalledPatchInfo> = installed.iter().map(|p| {
            let identity = p.backup_path.as_ref().and_then(|path|
                super::patch_lifecycle::recover_installation(Path::new(path)).ok());
            InstalledPatchInfo {
                language: identity.as_ref().map(|i| i.1.clone()),
                release_revision: identity.as_ref().map_or(0, |i| i.3),
                id: p.id,
                scummvm_id: p.scummvm_id.clone(),
                version: p.version.clone(),
                installed_date: p.installed_date.format("%Y-%m-%d").to_string(),
                status: p.status.clone(),
            }
        }).collect();
        
        // Get available patches from registry
        let available_patches: Vec<AvailablePatchInfo> = if let Some(def) = game_def.filter(|def| {
            def.platforms.as_ref().is_none_or(|p| if cfg!(target_os="windows") {p.windows} else if cfg!(target_os="macos") {p.osx} else {p.linux})
        }) {
            def.patches.iter().map(|p| {
                AvailablePatchInfo {
                    release_revision: p.release_revision,
                    language: p.language.clone(),
                    version: p.version.clone(),
                    author: p.author.clone(),
                    description: p.description.clone(),
                }
            }).collect()
        } else {
            Vec::new()
        };
        
        let has_available_patch = !available_patches.is_empty();
        
        let cover_url = game_def.and_then(|d| d.cover_url.clone()).or(game.cover_url);
        let requires_scummvm = !behavior.launch_profile(patched).is_native();

        games_with_patches.push(GameWithPatches {
            id: game.id,
            scummvm_id: game.scummvm_id,
            title: game.title,
            install_path: game.install_path,
            cover_url,
            detected_version: game.detected_version,
            added_date: game.added_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            last_played: game.last_played.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
            requires_scummvm,
            is_patched: patched,
            health: state.manual_health,
            has_available_patch,
            installed_patches,
            available_patches,
        });
    }
    
    Ok(games_with_patches)
}




#[cfg(test)]
mod tests;
