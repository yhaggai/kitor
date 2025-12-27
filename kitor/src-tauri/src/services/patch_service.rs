use sea_orm::*;
use std::path::{Path, PathBuf};
use crate::models::installed_patch;
use crate::services::{download_service, file_service, registry_service};
use serde::{Deserialize, Serialize};

// Patch representation for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInfo {
    pub scummvm_id: String,
    pub game_title: String,
    pub language: String,
    pub version: String,
    pub author: String,
    pub completion: i32,
    pub download_url: String,
    pub installer_url: Option<String>,
    pub source_type: String,
    pub checksum: Option<String>,
    pub file_size: Option<i64>,
    pub description: String,
}

pub async fn get_patches_for_game(
    db: &DatabaseConnection,
    game_id: i32,
) -> Result<Vec<PatchInfo>, String> {
    // Get game from DB to find its scummvm_id
    let game = crate::models::game::Entity::find_by_id(game_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Game not found")?;
    
    // Load patches from registry for this game
    let registry = registry_service::load_registry()?;
    
    let game_def = registry.games
        .into_iter()
        .find(|g| g.scummvm_id == game.scummvm_id)
        .ok_or("Game not found in registry")?;
    
    let patches: Vec<PatchInfo> = game_def.patches
        .into_iter()
        .map(|p| PatchInfo {
            scummvm_id: game_def.scummvm_id.clone(),
            game_title: game_def.title.clone(),
            language: p.language,
            version: p.version,
            author: p.author,
            completion: p.completion,
            download_url: p.download_url,
            installer_url: p.installer_url,
            source_type: p.source_type,
            checksum: p.checksum,
            file_size: p.file_size,
            description: p.description,
        })
        .collect();
    
    Ok(patches)
}

pub async fn get_all_patches(db: &DatabaseConnection) -> Result<Vec<PatchInfo>, String> {
    // Load all games from DB to include game_id in response
    let games = crate::models::game::Entity::find()
        .all(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    
    // Load patches from registry
    let registry = registry_service::load_registry()?;
    
    let mut all_patches = Vec::new();
    
    for game_def in registry.games {
        // Check if this game exists in user's library
        let _game_in_library = games.iter().find(|g| g.scummvm_id == game_def.scummvm_id);
        
        for patch in game_def.patches {
            all_patches.push(PatchInfo {
                scummvm_id: game_def.scummvm_id.clone(),
                game_title: game_def.title.clone(),
                language: patch.language,
                version: patch.version,
                author: patch.author,
                completion: patch.completion,
                download_url: patch.download_url,
                installer_url: patch.installer_url,
                source_type: patch.source_type,
                checksum: patch.checksum,
                file_size: patch.file_size,
                description: patch.description,
            });
        }
    }
    
    Ok(all_patches)
}

pub async fn download_and_install_patch(
    db: &DatabaseConnection,
    scummvm_id: String,
    language: String,
    version: String,
    app_handle: tauri::AppHandle,
    app_data_dir: PathBuf,
) -> Result<(), String> {
    // Get game from DB
    let game = crate::models::game::Entity::find()
        .filter(crate::models::game::Column::ScummvmId.eq(&scummvm_id))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Game not found in library")?;
    
    // Get patch from registry
    let registry = registry_service::load_registry()?;
    let game_def = registry.games
        .into_iter()
        .find(|g| g.scummvm_id == scummvm_id)
        .ok_or("Game not found in registry")?;
    
    let patch = game_def.patches
        .into_iter()
        .find(|p| p.language == language && p.version == version)
        .ok_or("Patch not found in registry")?;

    // Create downloads directory
    let downloads_dir = app_data_dir.join("downloads");
    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to create downloads directory: {}", e))?;

    // Download patch
    let file_name = patch.download_url
        .split('/')
        .last()
        .unwrap_or("patch.zip");
    let download_path = downloads_dir.join(file_name);

    download_service::download_file(&patch.download_url, &download_path, &app_handle)
        .await?;

    // Create backup
    let backup_dir = app_data_dir.join("backups").join(&game.scummvm_id);
    let backup_path = file_service::create_backup(
        Path::new(&game.install_path),
        &backup_dir,
    )?;

    // Extract and apply patch
    if download_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        file_service::extract_zip(&download_path, Path::new(&game.install_path))?;
    } else {
        // Copy file directly
        let dest = Path::new(&game.install_path).join(file_name);
        std::fs::copy(&download_path, dest)
            .map_err(|e| format!("Failed to copy patch file: {}", e))?;
    }

    // Record installation
    let installed = installed_patch::ActiveModel {
        game_id: Set(game.id),
        scummvm_id: Set(scummvm_id),
        language: Set(language),
        version: Set(version),
        installed_date: Set(chrono::Utc::now().naive_utc()),
        status: Set("active".to_string()),
        backup_path: Set(Some(backup_path)),
        ..Default::default()
    };

    installed.insert(db)
        .await
        .map_err(|e| format!("Failed to record installation: {}", e))?;

    // Clean up download
    let _ = std::fs::remove_file(download_path);

    Ok(())
}

pub async fn get_installed_patches(
    db: &DatabaseConnection,
    game_id: i32,
) -> Result<Vec<installed_patch::Model>, String> {
    installed_patch::Entity::find()
        .filter(installed_patch::Column::GameId.eq(game_id))
        .all(db)
        .await
        .map_err(|e| format!("Failed to fetch installed patches: {}", e))
}

pub async fn restore_backup(
    db: &DatabaseConnection,
    installed_patch_id: i32,
) -> Result<(), String> {
    let installed = installed_patch::Entity::find_by_id(installed_patch_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Installed patch not found")?;

    let backup_path = installed.backup_path.clone()
        .ok_or("No backup available")?;

    let game = crate::models::game::Entity::find_by_id(installed.game_id)
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Game not found")?;

    // Restore from backup
    let src = Path::new(&backup_path);
    let dst = Path::new(&game.install_path);

    if src.is_dir() {
        // Remove current files and restore
        std::fs::remove_dir_all(dst)
            .map_err(|e| format!("Failed to remove current files: {}", e))?;
        
        copy_dir_recursive(src, dst)?;
    } else {
        std::fs::copy(src, dst)
            .map_err(|e| format!("Failed to restore file: {}", e))?;
    }

    // Update status
    let mut active: installed_patch::ActiveModel = installed.into();
    active.status = Set("disabled".to_string());
    active.update(db)
        .await
        .map_err(|e| format!("Failed to update status: {}", e))?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let file_type = entry.file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }

    Ok(())
}
