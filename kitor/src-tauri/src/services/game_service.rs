use sea_orm::*;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::models::game::{self, Entity as Game};
use crate::models::installed_patch;
use crate::services::registry_service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPatchInfo {
    pub id: i32,
    pub scummvm_id: String,
    pub language: String,
    pub version: String,
    pub installed_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePatchInfo {
    pub language: String,
    pub version: String,
    pub author: String,
    pub completion: i32,
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
                    let game = game::ActiveModel {
                        scummvm_id: Set(game_info.scummvm_id),
                        title: Set(game_info.title),
                        install_path: Set(entry_path.to_string_lossy().to_string()),
                        cover_url: Set(None),
                        detected_version: Set(None),
                        added_date: Set(chrono::Utc::now().naive_utc()),
                        last_played: Set(None),
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
) -> Result<game::Model, String> {
    let game = game::ActiveModel {
        scummvm_id: Set(scummvm_id),
        title: Set(title),
        install_path: Set(install_path),
        cover_url: Set(None),
        detected_version: Set(None),
        added_date: Set(chrono::Utc::now().naive_utc()),
        last_played: Set(None),
        ..Default::default()
    };

    game.insert(db)
        .await
        .map_err(|e| format!("Failed to add game: {}", e))
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

fn detect_scummvm_game(path: &Path) -> Option<GameInfo> {
    // Load registry and try to match against game definitions
    let registry = registry_service::load_registry().ok()?;
    
    // Get list of files in directory
    let entries = std::fs::read_dir(path).ok()?;
    let mut files_in_dir: Vec<String> = Vec::new();
    
    for entry in entries.flatten() {
        if let Some(file_name) = entry.file_name().to_str() {
            files_in_dir.push(file_name.to_string());
        }
    }
    
    // Try to match against each game definition in registry
    for game_def in registry.games {
        let mut matches = false;
        
        // Check if any unique files match
        for unique_file in &game_def.detection_patterns.unique_files {
            if files_in_dir.iter().any(|f| f.eq_ignore_ascii_case(unique_file)) {
                matches = true;
                break;
            }
        }
        
        // If matched by unique files, return this game
        if matches {
            return Some(GameInfo {
                scummvm_id: game_def.scummvm_id,
                title: game_def.title,
            });
        }
    }
    
    None
}

pub async fn get_games_with_patches(db: &DatabaseConnection) -> Result<Vec<GameWithPatches>, String> {
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
        let installed = installed_patch::Entity::find()
            .filter(installed_patch::Column::GameId.eq(game.id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch installed patches: {}", e))?;
        
        let installed_patches: Vec<InstalledPatchInfo> = installed.iter().map(|p| {
            InstalledPatchInfo {
                id: p.id,
                scummvm_id: p.scummvm_id.clone(),
                language: p.language.clone(),
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
                    completion: p.completion,
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
            games_with_patches.push(GameWithPatches {
                id: game.id,
                scummvm_id: game.scummvm_id,
                title: game.title,
                install_path: game.install_path,
                cover_url: game.cover_url,
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




