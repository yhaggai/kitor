use sea_orm::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::{game, app_config};
use crate::services::{game_service, file_service};
use serde::{Serialize, Deserialize};
use tauri::Emitter;

const SCUMMVM_VERSION: &str = "2.8.1";

#[derive(Clone, Serialize, Deserialize)]
pub struct ScummVMStatus {
    pub downloaded: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

pub async fn launch_game(
    db: &DatabaseConnection,
    game_id: i32,
    app_data_dir: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    let game = game_service::get_game_by_id(db, game_id)
        .await?
        .ok_or("Game not found")?;

    println!("=== LAUNCHING GAME ===");
    println!("Game ID: {}", game.id);
    println!("ScummVM ID: {}", game.scummvm_id);
    println!("Title: {}", game.title);
    println!("Install Path: {}", game.install_path);

    // Get or download ScummVM
    let scummvm_path = get_or_download_scummvm(db, app_data_dir, app_handle).await?;
    println!("ScummVM Path: {}", scummvm_path);

    // Read launch config from DB
    println!("Launch config from DB:");
    println!("  Subdirectory: {:?}", game.launch_subdirectory);
    println!("  Args JSON: {:?}", game.launch_args);

    // Build command with launch_config if available
    let mut cmd = Command::new(&scummvm_path);
    
    // Determine game path (with optional subdirectory from DB)
    let game_path = if let Some(subdir) = &game.launch_subdirectory {
        PathBuf::from(&game.install_path).join(subdir)
    } else {
        PathBuf::from(&game.install_path)
    };

    println!("Final game path: {}", game_path.display());

    // Add default ScummVM args
    cmd.arg("-p").arg(&game_path);

    // Add custom args from DB
    if let Some(args_json) = &game.launch_args {
        let args: Vec<String> = serde_json::from_str(args_json)
            .map_err(|e| format!("Failed to parse launch args: {}", e))?;
        
        for arg in args {
            // Template replacement
            let processed_arg = arg
                .replace("{install_path}", &game.install_path)
                .replace("{scummvm_id}", &game.scummvm_id);
            cmd.arg(processed_arg);
        }
    }

    println!("Executing command: {:?}", cmd);
    println!("=====================");

    cmd.spawn()
        .map_err(|e| format!("Failed to launch game: {}", e))?;

    // Update last_played
    let mut active_game: game::ActiveModel = game.into();
    active_game.last_played = Set(Some(chrono::Utc::now().naive_utc()));
    active_game.update(db)
        .await
        .map_err(|e| format!("Failed to update last_played: {}", e))?;

    Ok(())
}

pub async fn get_or_download_scummvm(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
    _app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    let embedded_path = get_embedded_scummvm_path(app_data_dir)?;
    
    // Check if already downloaded
    if Path::new(&embedded_path).exists() {
        return Ok(embedded_path);
    }

    // Check database state
    let config = get_config_value(db, "scummvm_downloaded").await?;
    if config == Some("true".to_string()) && Path::new(&embedded_path).exists() {
        return Ok(embedded_path);
    }

    // Not found, trigger download
    Err("ScummVM not downloaded. Please download it first.".to_string())
}

pub async fn download_scummvm(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    println!("[scummvm_service] Starting ScummVM download");

    // Emit progress: Starting
    let _ = app_handle.emit("scummvm-download-progress", serde_json::json!({
        "progress": 0,
        "step": "Preparing download..."
    }));

    // Create scummvm directory
    let scummvm_dir = app_data_dir.join("scummvm");
    std::fs::create_dir_all(&scummvm_dir)
        .map_err(|e| format!("Failed to create ScummVM directory: {}", e))?;

    // Get download URL based on platform
    let (download_url, archive_name) = get_download_url_and_name()?;
    
    println!("[scummvm_service] Download URL: {}", download_url);

    // Download to temp location
    let downloads_dir = app_data_dir.join("downloads");
    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to create downloads directory: {}", e))?;
    
    let download_path = downloads_dir.join(&archive_name);

    // Emit progress: Downloading
    let _ = app_handle.emit("scummvm-download-progress", serde_json::json!({
        "progress": 10,
        "step": "Downloading ScummVM..."
    }));

    println!("[scummvm_service] Downloading to: {}", download_path.display());
    
    // Clone app_handle for the download progress tracking
    let app_handle_clone = app_handle.clone();
    let download_result = download_scummvm_with_progress(&download_url, &download_path, &app_handle_clone).await;
    
    if let Err(e) = download_result {
        return Err(format!("Download failed: {}", e));
    }
    
    println!("[scummvm_service] Download complete");

    // Emit progress: Extracting
    let _ = app_handle.emit("scummvm-download-progress", serde_json::json!({
        "progress": 60,
        "step": "Extracting files..."
    }));

    // Extract archive
    extract_scummvm_archive(&download_path, &scummvm_dir)?;
    
    println!("[scummvm_service] Extraction complete");

    // Verify executable exists
    let scummvm_path = get_embedded_scummvm_path(app_data_dir)?;
    if !Path::new(&scummvm_path).exists() {
        return Err("ScummVM executable not found after extraction".to_string());
    }

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&scummvm_path)
            .map_err(|e| format!("Failed to get file permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&scummvm_path, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
    }

    // Clean up download
    let _ = std::fs::remove_file(&download_path);

    // Update database state
    set_config_value(db, "scummvm_downloaded", "true").await?;
    set_config_value(db, "scummvm_path", &scummvm_path).await?;
    set_config_value(db, "scummvm_version", SCUMMVM_VERSION).await?;

    // Emit progress: Complete
    let _ = app_handle.emit("scummvm-download-progress", serde_json::json!({
        "progress": 100,
        "step": "Download complete!"
    }));

    println!("[scummvm_service] ScummVM installation complete");
    Ok(())
}

pub async fn check_scummvm_status(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
) -> Result<ScummVMStatus, String> {
    let embedded_path = get_embedded_scummvm_path(app_data_dir)?;
    let exists = Path::new(&embedded_path).exists();
    
    let version = if exists {
        get_config_value(db, "scummvm_version").await?
    } else {
        None
    };

    Ok(ScummVMStatus {
        downloaded: exists,
        path: if exists { Some(embedded_path) } else { None },
        version,
    })
}

pub async fn uninstall_scummvm(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
) -> Result<(), String> {
    println!("[scummvm_service] Uninstalling ScummVM");
    
    let scummvm_dir = app_data_dir.join("scummvm");
    
    // Remove ScummVM directory
    if scummvm_dir.exists() {
        std::fs::remove_dir_all(&scummvm_dir)
            .map_err(|e| format!("Failed to remove ScummVM directory: {}", e))?;
        println!("[scummvm_service] Removed ScummVM directory");
    }
    
    // Update database state
    set_config_value(db, "scummvm_downloaded", "false").await?;
    
    // Remove path and version from config
    let _ = app_config::Entity::delete_by_id("scummvm_path".to_string())
        .exec(db)
        .await;
    let _ = app_config::Entity::delete_by_id("scummvm_version".to_string())
        .exec(db)
        .await;
    
    println!("[scummvm_service] ScummVM uninstalled successfully");
    Ok(())
}

fn get_embedded_scummvm_path(app_data_dir: &PathBuf) -> Result<String, String> {
    let scummvm_dir = app_data_dir.join("scummvm");
    
    #[cfg(target_os = "windows")]
    {
        let exe_path = scummvm_dir.join("scummvm.exe");
        return Ok(exe_path.to_string_lossy().to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let exe_path = scummvm_dir.join("scummvm");
        return Ok(exe_path.to_string_lossy().to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let exe_path = scummvm_dir.join("scummvm");
        return Ok(exe_path.to_string_lossy().to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

fn get_download_url_and_name() -> Result<(String, String), String> {
    #[cfg(target_os = "windows")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-win32.zip",
            SCUMMVM_VERSION, SCUMMVM_VERSION
        );
        let name = format!("scummvm-{}-win32.zip", SCUMMVM_VERSION);
        return Ok((url, name));
    }

    #[cfg(target_os = "linux")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-linux-x86_64.tar.xz",
            SCUMMVM_VERSION, SCUMMVM_VERSION
        );
        let name = format!("scummvm-{}-linux.tar.xz", SCUMMVM_VERSION);
        return Ok((url, name));
    }

    #[cfg(target_os = "macos")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-macosx.dmg",
            SCUMMVM_VERSION, SCUMMVM_VERSION
        );
        let name = format!("scummvm-{}-macosx.dmg", SCUMMVM_VERSION);
        return Ok((url, name));
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

fn extract_scummvm_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // Extract ZIP and move executable to root of scummvm dir
        let temp_extract = dest_dir.join("temp_extract");
        std::fs::create_dir_all(&temp_extract)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
        file_service::extract_zip(archive_path, &temp_extract)?;
        
        // Find scummvm.exe in extracted files and move to dest_dir
        for entry in std::fs::read_dir(&temp_extract)
            .map_err(|e| format!("Failed to read temp directory: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Look for scummvm.exe in this directory
                let exe_path = path.join("scummvm.exe");
                if exe_path.exists() {
                    std::fs::copy(&exe_path, dest_dir.join("scummvm.exe"))
                        .map_err(|e| format!("Failed to copy executable: {}", e))?;
                    break;
                }
            }
        }
        
        // Clean up temp
        let _ = std::fs::remove_dir_all(&temp_extract);
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Extract tar.xz
        use std::process::Command;
        
        let output = Command::new("tar")
            .args(["xf", &archive_path.to_string_lossy(), "-C", &dest_dir.to_string_lossy(), "--strip-components=1"])
            .output()
            .map_err(|e| format!("Failed to extract archive: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("tar extraction failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        // For macOS, we'll need to mount DMG and copy files
        // This is more complex and might need special handling
        Err("macOS DMG extraction not yet implemented. Please install ScummVM manually.".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

async fn get_config_value(db: &DatabaseConnection, key: &str) -> Result<Option<String>, String> {
    let config = app_config::Entity::find_by_id(key.to_string())
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?;
    
    Ok(config.map(|c| c.value))
}

async fn download_scummvm_with_progress(
    url: &str,
    dest_path: &Path,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use futures::StreamExt;

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);
    println!("[scummvm_service] Download size: {} bytes", total_size);

    let mut file = File::create(dest_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_emitted_progress: u32 = 0; // Throttle: track last emitted progress

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Failed to read chunk: {}", e))?;
        
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded += chunk.len() as u64;

        // Map download percentage (0-100) to ScummVM progress range (10-60)
        let download_percentage = if total_size > 0 {
            ((downloaded as f64 / total_size as f64) * 100.0) as u32
        } else {
            0
        };
        
        let scummvm_progress = 10 + ((download_percentage * 50) / 100); // 10% + (0-50%)

        // THROTTLE: Only emit if progress changed by at least 2%
        if scummvm_progress >= last_emitted_progress + 2 || download_percentage == 100 {
            last_emitted_progress = scummvm_progress;
            
            let _ = app_handle.emit("scummvm-download-progress", serde_json::json!({
                "progress": scummvm_progress,
                "step": format!("Downloading ScummVM... {}%", download_percentage)
            }));
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    println!("[scummvm_service] Download stream completed, {} bytes written", downloaded);
    Ok(())
}

async fn set_config_value(db: &DatabaseConnection, key: &str, value: &str) -> Result<(), String> {
    let config = app_config::ActiveModel {
        key: Set(key.to_string()),
        value: Set(value.to_string()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };
    
    app_config::Entity::insert(config)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(app_config::Column::Key)
                .update_column(app_config::Column::Value)
                .update_column(app_config::Column::UpdatedAt)
                .to_owned()
        )
        .exec(db)
        .await
        .map_err(|e| format!("Failed to save config: {}", e))?;
    
    Ok(())
}
