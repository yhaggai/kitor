use sea_orm::*;
use std::path::Path;
use crate::models::game::{self, Entity as Game};
use crate::services::registry_service;

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



