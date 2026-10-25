use sea_orm::*;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use crate::models::installed_patch;
use crate::services::{download_service, file_service, registry_service, image_service, patch_lifecycle};
use serde::{Deserialize, Serialize};

// Serialize lifecycle operations through database recording as well as file writes.
static PATCH_OPERATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

// Patch representation for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchInfo {
    pub scummvm_id: String,
    pub game_title: String,
    pub language: String,
    pub version: String,
    pub author: String,
    pub download_url: String,
    pub source_type: String,
    pub checksum: Option<String>,
    pub file_size: Option<i64>,
    pub description: String,
    pub filename: Option<String>,
    pub post_install_script: Option<registry_service::PostInstallScript>,
    pub cover_url: Option<String>,
    pub gog_id: Option<i64>,
    pub gog_installer_path: Option<String>,
    pub platforms: Option<registry_service::Platforms>,
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
            download_url: p.download_url,
            source_type: p.source_type,
            checksum: p.checksum,
            file_size: p.file_size,
            description: p.description,
            filename: p.filename,
            post_install_script: p.post_install_script,
            cover_url: game_def.cover_url.clone(),
            gog_id: game_def.gog.as_ref().map(|g| g.id),
            gog_installer_path: game_def.gog.as_ref().and_then(|g| g.installers.as_ref()).and_then(|i| i.first().map(|inst| inst.path.clone())),
            platforms: game_def.platforms.clone(),
        })
        .collect();
    
    Ok(patches)
}

pub async fn get_all_patches(db: &DatabaseConnection, app_data_dir: PathBuf) -> Result<Vec<PatchInfo>, String> {
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
        
        // Download and cache cover if URL exists
        let cached_cover = if let Some(ref cover_url) = game_def.cover_url {
            if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                // Try to cache, but don't fail if it doesn't work
                match image_service::download_and_cache_cover(&game_def.scummvm_id, cover_url, app_data_dir.clone()).await {
                    Ok(path) => Some(path.to_string_lossy().to_string()),
                    Err(e) => {
                        eprintln!("Failed to cache cover for {}: {}", game_def.scummvm_id, e);
                        Some(cover_url.clone())
                    }
                }
            } else {
                Some(cover_url.clone())
            }
        } else {
            None
        };
        
        for patch in game_def.patches {
            all_patches.push(PatchInfo {
                scummvm_id: game_def.scummvm_id.clone(),
                game_title: game_def.title.clone(),
                language: patch.language,
                version: patch.version,
                author: patch.author,
                download_url: patch.download_url,
                source_type: patch.source_type,
                checksum: patch.checksum,
                file_size: patch.file_size,
                description: patch.description,
                filename: patch.filename,
                post_install_script: patch.post_install_script,
                cover_url: cached_cover.clone(),
                gog_id: game_def.gog.as_ref().map(|g| g.id),
                gog_installer_path: game_def.gog.as_ref().and_then(|g| g.installers.as_ref()).and_then(|i| i.first().map(|inst| inst.path.clone())),
                platforms: game_def.platforms.clone(),
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
    let _operation = PATCH_OPERATION.lock().await;
    println!("[patch_service] Starting installation for {} (lang: {}, ver: {})", scummvm_id, language, version);
    
    // Emit progress: Starting
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 0,
        "step": "Starting installation..."
    }));

    // Get game from DB
    println!("[patch_service] Fetching game from database");
    let game = crate::models::game::Entity::find()
        .filter(crate::models::game::Column::ScummvmId.eq(&scummvm_id))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or("Game not found in library")?;
    
    println!("[patch_service] Game found: {} at {}", game.title, game.install_path);
    
    // Get patch from registry
    println!("[patch_service] Loading patch registry");
    let registry = registry_service::load_registry()?;
    let game_def = registry.games
        .into_iter()
        .find(|g| g.scummvm_id == scummvm_id)
        .ok_or("Game not found in registry")?;
    
    if let Some(platforms) = &game_def.platforms {
        let supported = if cfg!(target_os = "windows") { platforms.windows }
            else if cfg!(target_os = "macos") { platforms.osx }
            else { platforms.linux };
        if !supported {
            return Err("This patch does not support the current operating system".into());
        }
    }

    let patch = game_def.patches
        .into_iter()
        .find(|p| p.language == language && p.version == version)
        .ok_or("Patch not found in registry")?;

    println!("[patch_service] Patch found: {} by {}", patch.description, patch.author);

    // Emit progress: Preparing download
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 10,
        "step": "Preparing download..."
    }));

    // Create downloads directory
    let downloads_dir = app_data_dir.join("downloads");
    println!("[patch_service] Creating downloads directory: {}", downloads_dir.display());
    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to create downloads directory: {}", e))?;

    // Download patch
    let file_name = patch.filename
        .clone()
        .unwrap_or_else(|| {
            patch.download_url
                .split('/')
                .last()
                .unwrap_or("patch.zip")
                .to_string()
        });
    let download_path = downloads_dir.join(&file_name);

    // Emit progress: Downloading
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 20,
        "step": "Downloading patch..."
    }));

    println!("[patch_service] Downloading from: {}", patch.download_url);
    println!("[patch_service] Download destination: {}", download_path.display());
    download_service::download_file(&patch.download_url, &download_path, &app_handle)
        .await?;
    println!("[patch_service] Download complete");
    
    // Emit progress: Download complete
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 45,
        "step": "Download complete!"
    }));

    let install_target = crate::services::game_service::detect_game_in_folder(
        Path::new(&game.install_path),
    )?
    .filter(|detected| detected.scummvm_id == game.scummvm_id)
    .map(|detected| PathBuf::from(detected.install_path))
    .ok_or("Game data folder no longer matches the registered game")?;

    let manifest_name = patch.install_manifest.as_deref()
        .ok_or("Patch has no file-change manifest; whole-folder installation is no longer supported")?;
    if download_path.extension().and_then(|s| s.to_str()) != Some("zip") {
        return Err("Declarative patch packages must be ZIP archives".into());
    }
    // Package tools and source data never enter the game installation.
    let staging = tempfile::Builder::new().prefix("kitor-package-").tempdir().map_err(|e| e.to_string())?;
    file_service::validate_zip(&download_path)?;
    file_service::extract_zip(&download_path, staging.path())?;
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 55,
        "step": "Validating outputs and backing up affected files..."
    }));
    let previous = get_installed_patches(db, game.id).await?;
    if previous.iter().any(|p| p.backup_path.as_ref().is_none_or(|p| !Path::new(p).join("manifest.json").is_file())) {
        return Err("This installation has a legacy folder backup. Resolve the legacy installation before applying a new patch; its backup has been preserved.".into());
    }
    let backup_path = patch_lifecycle::install(
        staging.path(), manifest_name, &install_target,
        &app_data_dir.join("patch-transactions"), &scummvm_id, &language,
        &version, patch_lifecycle::platform(),
    )?;
    // Retrying the same version reuses the verified active transaction and its
    // originals. Do not insert a second database record for it.
    let already_recorded = previous.iter().any(|p| p.backup_path.as_deref() == backup_path.to_str());

    // Emit progress: Finalizing
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 90,
        "step": "Finalizing..."
    }));

    // Record installation
    println!("[patch_service] Recording installation in database");
    let installed = installed_patch::ActiveModel {
        game_id: Set(game.id),
        scummvm_id: Set(scummvm_id.clone()),
        version: Set(version),
        installed_date: Set(chrono::Utc::now().naive_utc()),
        status: Set("active".to_string()),
        backup_path: Set(Some(backup_path.to_string_lossy().into_owned())),
        ..Default::default()
    };

    if !already_recorded {
        if let Err(error) = installed.insert(db).await {
            let recovery = patch_lifecycle::uninstall(&backup_path, &install_target);
            return Err(format!("Failed to record installation: {error}; rollback: {recovery:?}. Recovery manifest: {}", backup_path.display()));
        }
    }
    println!("[patch_service] Installation recorded successfully");

    // Clean up download
    println!("[patch_service] Cleaning up download file");
    let _ = std::fs::remove_file(&download_path);

    // Emit progress: Complete
    let _ = app_handle.emit("install-progress", serde_json::json!({
        "scummvm_id": scummvm_id,
        "progress": 100,
        "step": "Installation complete!"
    }));

    println!("[patch_service] Installation complete for {}", scummvm_id);
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
    let _operation = PATCH_OPERATION.lock().await;
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

    // Use the recorded installation root even if the patch removed a detection
    // signature. The journal validates that this is the correct installation.
    patch_lifecycle::uninstall(Path::new(&backup_path), Path::new(&game.install_path))?;

    // Delete installed patch record
    installed_patch::Entity::delete_by_id(installed_patch_id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete installed patch record: {}", e))?;

    Ok(())
}

pub async fn download_and_extract_gog_installer(
    db: &DatabaseConnection,
    scummvm_id: String,
    installer_path: String,
    dest_dir: PathBuf,
    app_handle: tauri::AppHandle,
    app_data_dir: PathBuf,
) -> Result<(), String> {
    use crate::services::{gog_service, game_service};
    use std::process::Command;
    
    // 1. Verify innoextract is installed
    let innoextract_check = Command::new("innoextract").arg("--version").output();
    if innoextract_check.is_err() {
        // Attempt to brew install on macos
        #[cfg(target_os = "macos")]
        {
            let _ = app_handle.emit("gog-installer-progress", serde_json::json!({
                "step": "Installing innoextract...",
                "progress": 0
            }));
            let brew_install = Command::new("brew").arg("install").arg("innoextract").output();
            if brew_install.is_err() || !brew_install.unwrap().status.success() {
                return Err("innoextract is not installed. Please try 'brew install innoextract' manually.".to_string());
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Err("innoextract is not installed. Please install it and try again.".to_string());
        }
    }

    // 2. Get GOG access token
    let token = gog_service::get_valid_token(db).await?;
    
    // 3. Download the installer
    // Append the access token as a query parameter
    let url = if installer_path.contains("?") {
        format!("{}&access_token={}", installer_path, token)
    } else {
        format!("{}?access_token={}", installer_path, token)
    };
    
    let downloads_dir = app_data_dir.join("downloads");
    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to create downloads directory: {}", e))?;
    
    let file_name = std::path::Path::new(url.split('?').next().unwrap_or(&url))
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("gog_installer.exe");
        
    let download_path = downloads_dir.join(file_name);
    
    let _ = app_handle.emit("gog-installer-progress", serde_json::json!({
        "step": "Downloading installer...",
        "progress": 10
    }));
    
    // Use download_service to download the file
    crate::services::download_service::download_file(&url, &download_path, &app_handle).await
        .map_err(|e| format!("Failed to download installer: {}", e))?;
        
    let _ = app_handle.emit("gog-installer-progress", serde_json::json!({
        "step": "Extracting game files...",
        "progress": 50
    }));

    // 4. Extract using innoextract to dest_dir
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;
        
    let extract_output = Command::new("innoextract")
        .arg("-d")
        .arg(&dest_dir)
        .arg("--lowercase")
        .arg("-e")
        .arg(&download_path)
        .output()
        .map_err(|e| format!("Failed to execute innoextract: {}", e))?;
        
    if !extract_output.status.success() {
        let _ = std::fs::remove_file(&download_path);
        return Err(format!("innoextract failed: {}", String::from_utf8_lossy(&extract_output.stderr)));
    }
    
    // 5. Clean up the installer
    let _ = std::fs::remove_file(&download_path);
    
    let _ = app_handle.emit("gog-installer-progress", serde_json::json!({
        "step": "Adding to library...",
        "progress": 90
    }));

    // Innoextract typically extracts to a folder like "app". We should point to that if it exists.
    let mut game_path = dest_dir.clone();
    if dest_dir.join("app").exists() {
        game_path = dest_dir.join("app");
    }

    // 6. Add game to library
    let registry = registry_service::load_registry()?;
    let title = registry.games.iter()
        .find(|g| g.scummvm_id == scummvm_id)
        .map(|g| g.title.clone())
        .unwrap_or_else(|| scummvm_id.clone());

    game_service::add_game_manually(
        db,
        scummvm_id,
        title,
        game_path.to_string_lossy().to_string(),
        app_data_dir,
    ).await.map_err(|e| format!("Failed to add game to library: {}", e))?;
    
    let _ = app_handle.emit("gog-installer-progress", serde_json::json!({
        "step": "Complete",
        "progress": 100
    }));

    Ok(())
}

#[cfg(test)]
mod restore_tests {
    use super::*;

    #[tokio::test]
    async fn selective_restore_preserves_unrelated_files_and_database_record_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let data = root.join("game");
        let package = root.join("package");
        std::fs::create_dir(&data).unwrap();
        std::fs::create_dir(&package).unwrap();
        std::fs::write(data.join("GEMCUT.PAK"), b"original").unwrap();
        std::fs::write(data.join("save.dat"), b"saved game").unwrap();
        std::fs::write(package.join("translated"), b"translated").unwrap();
        let checksum = patch_lifecycle::hash(&package.join("translated")).unwrap();
        std::fs::write(package.join("patch.json"), serde_json::to_vec(&serde_json::json!({
            "schema_version": 1, "scummvm_id": "kyra1-cd", "language": "he", "version": "test",
            "filename": "test.zip", "platforms": {"osx": {}},
            "files": [{"path":"GEMCUT.PAK","operation":"modify", "source": {
                "kind":"copy", "path":"translated", "sha256":checksum
            }}]
        })).unwrap()).unwrap();
        let backup = patch_lifecycle::install(&package, "patch.json", &data,
            &root.join("transactions"), "kyra1-cd", "he", "test", "osx").unwrap();
        let db = crate::db::init_db(&root.join("db")).await.unwrap();
        let game = crate::services::game_service::add_game_manually(&db,
            "kyra1-cd".into(), "Kyrandia".into(), data.to_string_lossy().into_owned(), root.to_path_buf())
            .await.unwrap();
        let installed = installed_patch::ActiveModel {
            game_id: Set(game.id), scummvm_id: Set("kyra1-cd".into()),
            version: Set("test".into()), installed_date: Set(chrono::Utc::now().naive_utc()),
            status: Set("active".into()), backup_path: Set(Some(backup.to_string_lossy().into_owned())),
            ..Default::default()
        }.insert(&db).await.unwrap();
        std::fs::write(data.join("later.txt"), b"user file").unwrap();
        std::fs::write(data.join("GEMCUT.PAK"), b"another mod").unwrap();
        assert!(restore_backup(&db, installed.id).await.is_err());
        assert_eq!(get_installed_patches(&db, game.id).await.unwrap().len(), 1);
        std::fs::write(data.join("GEMCUT.PAK"), b"translated").unwrap();
        restore_backup(&db, installed.id).await.unwrap();
        assert_eq!(std::fs::read(data.join("GEMCUT.PAK")).unwrap(), b"original");
        assert_eq!(std::fs::read(data.join("save.dat")).unwrap(), b"saved game");
        assert_eq!(std::fs::read(data.join("later.txt")).unwrap(), b"user file");
        assert!(get_installed_patches(&db, game.id).await.unwrap().is_empty());
    }
}
