use sea_orm::*;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::models::game::{self, Entity as Game};
use crate::models::installed_patch;
use crate::services::image_service;
use crate::services::registry_service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPatchInfo {
    pub id: i32,
    pub scummvm_id: String,
    pub version: String,
    pub installed_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePatchInfo {
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
    pub has_available_patch: bool,
    pub installed_patches: Vec<InstalledPatchInfo>,
    pub available_patches: Vec<AvailablePatchInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameInfo {
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
    app_data_dir: PathBuf,
) -> Result<Vec<game::Model>, String> {
    let path = Path::new(scan_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    let mut games = Vec::new();

    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if let Some(game_info) = detect_scummvm_game(&entry_path) {
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
                        install_path: Set(entry_path.to_string_lossy().to_string()),
                        cover_url: Set(cover_url.clone()),
                        detected_version: Set(None),
                        added_date: Set(chrono::Utc::now().naive_utc()),
                        last_played: Set(None),
                        launch_subdirectory: Set(launch_subdirectory),
                        launch_args: Set(launch_args),
                        ..Default::default()
                    };

                    let result = game.insert(db).await
                        .map_err(|e| format!("Failed to insert game: {}", e))?;

                    if let Some(url) = cover_url {
                        let scummvm_id = result.scummvm_id.clone();
                        let app_data_dir = app_data_dir.clone();
                        tokio::spawn(async move {
                            let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
                        });
                    }
                    
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
    app_data_dir: PathBuf,
) -> Result<game::Model, String> {
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

        let (_, _, cover_url) = get_game_metadata_from_registry(&scummvm_id)?;
        if cover_url.is_some() {
            active.cover_url = Set(cover_url.clone());
        }

        if let Some(url) = cover_url {
            let scummvm_id = scummvm_id.clone();
            let app_data_dir = app_data_dir.clone();
            tokio::spawn(async move {
                let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
            });
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
        cover_url: Set(cover_url.clone()),
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

    if let Some(url) = cover_url {
        let scummvm_id = result.scummvm_id.clone();
        tokio::spawn(async move {
            let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
        });
    }

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
    scummvm_id: String,
    title: String,
}

fn get_game_metadata_from_registry(
    scummvm_id: &str,
) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let registry = registry_service::load_registry()?;
    
    if let Some(game_def) = registry.games.iter().find(|g| g.scummvm_id == scummvm_id) {
        let cover_url = game_def.cover_url.clone();

        if let Some(launch_config) = &game_def.launch_config {
            let subdirectory = launch_config.subdirectory.clone();
            let args_json = if !launch_config.args.is_empty() {
                Some(serde_json::to_string(&launch_config.args)
                    .map_err(|e| format!("Failed to serialize launch args: {}", e))?)
            } else {
                None
            };
            return Ok((subdirectory, args_json, cover_url));
        }

        return Ok((None, None, cover_url));
    }
    
    Ok((None, None, None))
}

pub fn detect_game_in_folder(path: &Path) -> Result<Option<DetectedGameInfo>, String> {
    if !path.exists() || !path.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    // Load registry and try to match against game definitions
    let registry = registry_service::load_registry()?;

    // Get list of files in directory (case-insensitive)
    let entries = std::fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut files_in_dir_lower: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if let Some(file_name) = entry.file_name().to_str() {
            files_in_dir_lower.push(file_name.to_ascii_lowercase());
        }
    }

    for game_def in registry.games {
        // Only allow games that actually have patches in registry
        if game_def.patches.is_empty() {
            continue;
        }

        let matches = game_def
            .detection_patterns
            .unique_files
            .iter()
            .any(|unique_file| files_in_dir_lower.iter().any(|f| f == &unique_file.to_ascii_lowercase()));

        if matches {
            return Ok(Some(DetectedGameInfo {
                scummvm_id: game_def.scummvm_id,
                title: game_def.title,
                available_patches_count: game_def.patches.len(),
            }));
        }
    }

    Ok(None)
}

fn detect_scummvm_game(path: &Path) -> Option<GameInfo> {
    let detected = detect_game_in_folder(path).ok()??;
    Some(GameInfo {
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

    let _ = image_service::ensure_covers_dir(&app_data_dir);
    
    let mut games_with_patches = Vec::new();
    
    for game in games {
        // Find game definition in registry
        let game_def = registry.games.iter()
            .find(|g| g.scummvm_id == game.scummvm_id);
        
        // Get installed patches for this game
        let installed = installed_patch::Entity::find()
            .filter(installed_patch::Column::GameId.eq(game.id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch installed patches: {}", e))?;
        
        let installed_patches: Vec<InstalledPatchInfo> = installed.iter().map(|p| {
            InstalledPatchInfo {
                id: p.id,
                scummvm_id: p.scummvm_id.clone(),
                version: p.version.clone(),
                installed_date: p.installed_date.format("%Y-%m-%d").to_string(),
                status: p.status.clone(),
            }
        }).collect();
        
        // Get available patches from registry
        let available_patches: Vec<AvailablePatchInfo> = if let Some(def) = game_def {
            def.patches.iter().map(|p| {
                AvailablePatchInfo {
                    language: p.language.clone(),
                    version: p.version.clone(),
                    author: p.author.clone(),
                    description: p.description.clone(),
                }
            }).collect()
        } else {
            Vec::new()
        };
        
        // Only include games that have patches (available or installed)
        let has_available_patch = !available_patches.is_empty();
        let has_installed_patch = !installed_patches.is_empty();
        
        if has_available_patch || has_installed_patch {
            let registry_cover_url = game_def.and_then(|d| d.cover_url.clone());
            let cached_cover_path = image_service::get_cached_cover_path(&game.scummvm_id, &app_data_dir)?;

            if cached_cover_path.is_none() {
                if let Some(url) = registry_cover_url.clone().or_else(|| game.cover_url.clone()) {
                    let scummvm_id = game.scummvm_id.clone();
                    let app_data_dir = app_data_dir.clone();
                    tokio::spawn(async move {
                        let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
                    });
                }
            }

            let cover_url = if let Some(path) = cached_cover_path {
                Some(path.to_string_lossy().to_string())
            } else {
                registry_cover_url.or(game.cover_url.clone())
            };

            games_with_patches.push(GameWithPatches {
                id: game.id,
                scummvm_id: game.scummvm_id,
                title: game.title,
                install_path: game.install_path,
                cover_url,
                detected_version: game.detected_version,
                added_date: game.added_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_played: game.last_played.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                has_available_patch,
                installed_patches,
                available_patches,
            });
        }
    }
    
    Ok(games_with_patches)
}




