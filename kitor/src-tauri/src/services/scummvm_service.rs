use sea_orm::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::{game, app_config};
use crate::services::game_service;
use serde::{Serialize, Deserialize};
use tauri::Emitter;

async fn fetch_latest_scummvm_version() -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("kitor/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://api.github.com/repos/scummvm/scummvm/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch GitHub releases: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned error: {}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read GitHub API response: {}", e))?;

    let release: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let tag_name = release["tag_name"]
        .as_str()
        .ok_or("tag_name not found in GitHub API response")?;

    // Remove 'v' prefix if present (e.g., "v2.9.1" -> "2.9.1")
    let version = tag_name.strip_prefix('v').unwrap_or(tag_name).to_string();

    Ok(version)
}

pub async fn get_latest_scummvm_version() -> Result<String, String> {
    fetch_latest_scummvm_version().await
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ScummVMStatus {
    pub downloaded: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

// Some GOG Book One bundles also contain FATE.PAK. Together with the Hebrew
// font it wins ScummVM's detection as Book Two. Detect using only Book One's
// signature, then load all remaining assets through --extrapath.
fn prepare_kyrandia_launch_path(data_path: &Path, launch_path: &Path) -> Result<PathBuf, String> {
    let signature = std::fs::read_dir(data_path)
        .map_err(|e| format!("Failed to read Kyrandia data: {}", e))?
        .flatten()
        .find(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case("GEMCUT.PAK")
            && entry.path().is_file())
        .ok_or("Kyrandia GEMCUT.PAK not found")?;
    std::fs::create_dir_all(launch_path)
        .map_err(|e| format!("Failed to create Kyrandia launch folder: {}", e))?;
    // Refresh after installing or restoring a patch; the signature changes.
    std::fs::copy(signature.path(), launch_path.join("GEMCUT.PAK"))
        .map_err(|e| format!("Failed to prepare Kyrandia launch: {}", e))?;
    Ok(launch_path.to_path_buf())
}

fn add_game_paths(cmd: &mut Command, game_path: &Path, extra_path: Option<&Path>) {
    cmd.arg("-p").arg(game_path);
    if let Some(path) = extra_path {
        // ScummVM long options require --name=value, unlike the short -p option.
        let mut argument = std::ffi::OsString::from("--extrapath=");
        argument.push(path);
        cmd.arg(argument);
    }
}

fn add_required_game_options(cmd: &mut Command, scummvm_id: &str) {
    // Pajama Sam's Hebrew patch requires subtitles explicitly enabled. Apply at
    // launch so libraries with older, persisted launch_args also get the fix.
    if scummvm_id == "pajama" {
        cmd.arg("--subtitles");
    }
}

// A locally built Toon runtime fixes Hebrew string lifetimes without replacing
// the shared ScummVM installation used by other engines.
fn game_runtime_path(app_data_dir: &Path, game_id: &str, standard: PathBuf) -> PathBuf {
    let executable = if cfg!(target_os = "windows") { "scummvm.exe" } else { "scummvm" };
    let patched = app_data_dir.join("scummvm-toon-hebrew").join(executable);
    if game_id == "toonstruck" && patched.is_file()
        && patched.parent().is_some_and(|dir| dir.join("toon.dat").is_file()) {
        patched
    } else { standard }
}

fn add_toon_runtime_data(cmd: &mut Command, app_data_dir: &Path, game_id: &str) {
    let standard = PathBuf::new();
    let runtime = game_runtime_path(app_data_dir, game_id, standard.clone());
    if runtime != standard {
        let mut argument = std::ffi::OsString::from("--extrapath=");
        argument.push(runtime.parent().unwrap());
        cmd.arg(argument);
    }
}

// Collection editions need an explicit identity/platform. Let ScummVM detect
// language from the installed data, so the original English files also launch.
fn normalize_collection_args(id: &str, args: Vec<String>, patched: bool) -> Vec<String> {
    let engine = match id {
        "sq1" | "sq2" => "agi",
        "sq1sci" | "sq3" | "sq5" => "sci",
        _ => return args,
    };
    let target = format!("{engine}:{id}");
    let mut args: Vec<_> = args.into_iter().filter(|arg|
        arg != "--auto-detect" && (patched || !arg.starts_with("--language=")) && !arg.starts_with("--platform=") && arg != &target
    ).collect();
    // Persisted settings may omit Hebrew or still specify English. SQ1 VGA's
    // translated resources require SCI's RTL rendering whenever patched.
    if id == "sq1sci" && patched {
        args.retain(|arg| !arg.starts_with("--language="));
        args.push("--language=he".into());
    }
    args.push("--platform=dos".into());
    args.push(target);
    args
}

async fn start_game_process(cmd: &mut Command, log_path: &Path) -> Result<(), String> {
    let log = std::fs::File::create(log_path)
        .map_err(|e| format!("Failed to create launch log: {}", e))?;
    cmd.stderr(std::process::Stdio::from(log));
    let mut child = cmd.spawn().map_err(|e| format!("Failed to launch game: {}", e))?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if let Some(status) = child.try_wait().map_err(|e| format!("Failed to check game launch: {}", e))? {
        if !status.success() {
            let detail = std::fs::read_to_string(log_path).unwrap_or_default();
            return Err(format!("Game exited with {}: {}", status, detail.trim()));
        }
    } else {
        // Reap the process when play ends without blocking the UI.
        std::thread::spawn(move || { let _ = child.wait(); });
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn drifter_launch_command(install_path: &Path) -> Result<Command, String> {
    // Detection stores the app bundle; accept a manually selected Steam folder too.
    let bundle = [install_path.to_path_buf(), install_path.join("TheDrifter.app")]
        .into_iter()
        .find(|path| path.join("Contents/MacOS/The Drifter").is_file()
            && path.join("Contents/Resources/Data/globalgamemanagers").is_file())
        .ok_or("The Drifter app was not found in its installation folder")?;
    let bundle = bundle.canonicalize().map_err(|e| e.to_string())?;
    let mut cmd = Command::new("/usr/bin/open");
    cmd.arg(&bundle);
    cmd.current_dir(bundle.parent().ok_or("Invalid Drifter app path")?);
    Ok(cmd)
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

    #[cfg(target_os = "macos")]
    if game.scummvm_id == "drifter" {
        let mut cmd = drifter_launch_command(Path::new(&game.install_path))?;
        return start_game_process(&mut cmd, &app_data_dir.join("drifter-launch.log")).await;
    }

    if game.scummvm_id == "day-of-the-tentacle-remastered" {
        let mut cmd = super::tentacle_remastered::launch_command(Path::new(&game.install_path))?;
        return start_game_process(&mut cmd, &app_data_dir.join("dott-remastered-launch.log")).await;
    }

    if game.scummvm_id == "owlboy" {
        if !cfg!(target_os = "macos") { return Err("Owlboy Hebrew currently supports the native macOS edition only".into()); }
        let mut cmd = super::owlboy::launch_command(Path::new(&game.install_path))?;
        return start_game_process(&mut cmd, &app_data_dir.join("owlboy-launch.log")).await;
    }

    if game.scummvm_id == "undertale" {
        let data = Path::new(&game.install_path);
        #[cfg(target_os = "macos")]
        let mut cmd = {
            let bundle = data.parent().and_then(Path::parent)
                .ok_or("Invalid UNDERTALE app path")?;
            if !bundle.join("Contents/MacOS/Mac_Runner").is_file() {
                return Err("UNDERTALE Mac runner not found".into());
            }
            let mut command = Command::new("open");
            command.arg(bundle);
            command
        };
        #[cfg(target_os = "windows")]
        let mut cmd = Command::new(data.join("UNDERTALE.exe"));
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return Err("The UNDERTALE Hebrew patch supports macOS and Windows only".into());
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            cmd.current_dir(data);
            return start_game_process(&mut cmd, &app_data_dir.join("undertale-launch.log")).await;
        }
    }

    // Get or download ScummVM
    let scummvm_path = get_or_download_scummvm(db, app_data_dir, app_handle).await?;
    println!("ScummVM Path: {}", scummvm_path);

    // Remove quarantine attribute on macOS before launching
    #[cfg(target_os = "macos")]
    {
        let scummvm_dir = app_data_dir.join("scummvm");
        let app_bundle = scummvm_dir.join("ScummVM.app");
        if app_bundle.exists() {
            remove_quarantine_attribute(&app_bundle);
        }
    }

    let collection_patched = if matches!(game.scummvm_id.as_str(), "sq1" | "sq1sci" | "sq2" | "sq3" | "sq5") {
        use crate::models::installed_patch;
        installed_patch::Entity::find()
            .filter(installed_patch::Column::GameId.eq(game.id))
            .filter(installed_patch::Column::Status.eq("active"))
            .one(db).await.map_err(|e| e.to_string())?.is_some()
    } else { false };

    // Read launch config from DB
    println!("Launch config from DB:");
    println!("  Subdirectory: {:?}", game.launch_subdirectory);
    println!("  Args JSON: {:?}", game.launch_args);

    // Determine game path (with optional subdirectory from DB)
    let game_path = if game.scummvm_id == "tentacle"
        && super::tentacle::detect_classic(Path::new(&game.install_path)).is_some() {
        PathBuf::from(&game.install_path)
    } else if let Some(subdir) = &game.launch_subdirectory {
        PathBuf::from(&game.install_path).join(subdir)
    } else {
        PathBuf::from(&game.install_path)
    };

    let extra_path = if game.scummvm_id == "kyra1-cd" {
        Some(game_path.clone())
    } else {
        None
    };
    let game_path = if extra_path.is_some() {
        prepare_kyrandia_launch_path(
            &game_path,
            &app_data_dir.join("launch-data").join(game.id.to_string()),
        )?
    } else {
        game_path
    };

    println!("Final game path: {}", game_path.display());

    // Build command with launch_config if available
    #[cfg(target_os = "macos")]
    {
        // On macOS, use 'open' command with --args to launch app bundle with arguments
        let scummvm_dir = app_data_dir.join("scummvm");
        let app_bundle = scummvm_dir.join("ScummVM.app");
        
        if !app_bundle.exists() {
            return Err("ScummVM.app not found".to_string());
        }
        
        // Remove quarantine attribute before launching
        remove_quarantine_attribute(&app_bundle);
        
        // Launch executable directly (more reliable than app bundle on macOS)
        let executable_path = game_runtime_path(app_data_dir, &game.scummvm_id,
            app_bundle.join("Contents").join("MacOS").join("scummvm"));
        if !executable_path.exists() {
            return Err("ScummVM executable not found in app bundle".to_string());
        }
        
        // Build command with args
        let mut cmd = Command::new(&executable_path);
        
        // Add default ScummVM args
        add_game_paths(&mut cmd, &game_path, extra_path.as_deref());
        add_required_game_options(&mut cmd, &game.scummvm_id);
        add_toon_runtime_data(&mut cmd, app_data_dir, &game.scummvm_id);
        
        // Add custom args from DB
        if let Some(args_json) = &game.launch_args {
            let custom_args: Vec<String> = serde_json::from_str(args_json)
                .map_err(|e| format!("Failed to parse launch args: {}", e))?;
            
            for arg in normalize_collection_args(&game.scummvm_id, custom_args, collection_patched) {
                // Template replacement
                let processed_arg = arg
                    .replace("{install_path}", &game.install_path)
                    .replace("{scummvm_id}", &game.scummvm_id);
                cmd.arg(processed_arg);
            }
        }
        
        println!("Executing command: {:?}", cmd);
        println!("=====================");
        
        start_game_process(&mut cmd, &app_data_dir.join(format!("launch-{}.log", game.id))).await?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On Windows and Linux, launch the executable directly
        let mut cmd = Command::new(game_runtime_path(app_data_dir, &game.scummvm_id,
            PathBuf::from(&scummvm_path)));
        
        // Add default ScummVM args
        add_game_paths(&mut cmd, &game_path, extra_path.as_deref());
        add_required_game_options(&mut cmd, &game.scummvm_id);
        add_toon_runtime_data(&mut cmd, app_data_dir, &game.scummvm_id);

        // Add custom args from DB
        if let Some(args_json) = &game.launch_args {
            let args: Vec<String> = serde_json::from_str(args_json)
                .map_err(|e| format!("Failed to parse launch args: {}", e))?;
            
            for arg in normalize_collection_args(&game.scummvm_id, args, collection_patched) {
                // Template replacement
                let processed_arg = arg
                    .replace("{install_path}", &game.install_path)
                    .replace("{scummvm_id}", &game.scummvm_id);
                cmd.arg(processed_arg);
            }
        }

        println!("Executing command: {:?}", cmd);
        println!("=====================");

        start_game_process(&mut cmd, &app_data_dir.join(format!("launch-{}.log", game.id))).await?;
    }

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
    let (download_url, archive_name) = get_download_url_and_name().await?;
    
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
    let version = fetch_latest_scummvm_version().await?;
    set_config_value(db, "scummvm_downloaded", "true").await?;
    set_config_value(db, "scummvm_path", &scummvm_path).await?;
    set_config_value(db, "scummvm_version", &version).await?;

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
    let file_exists = Path::new(&embedded_path).exists();
    
    // Check database state
    let db_downloaded = get_config_value(db, "scummvm_downloaded").await? == Some("true".to_string());
    
    // ScummVM is only ready if the executable file actually exists
    let downloaded = file_exists;
    
    let version = if file_exists {
        get_config_value(db, "scummvm_version").await?
    } else if db_downloaded {
        Some("Missing/Corrupted".to_string())
    } else {
        None
    };

    Ok(ScummVMStatus {
        downloaded,
        path: if file_exists { Some(embedded_path) } else { None },
        version,
    })
}

pub async fn uninstall_scummvm(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
) -> Result<(), String> {
    println!("[scummvm_service] Uninstalling ScummVM");
    
    let scummvm_dir = app_data_dir.join("scummvm");
    
    // On macOS, aggressively clean up any stuck mounts
    #[cfg(target_os = "macos")]
    {
        let mount_point = scummvm_dir.join("temp_mount");
        if mount_point.exists() {
            println!("[scummvm_service] Found potential mount point at {}", mount_point.display());
            
            // Try hdiutil first
            let _ = Command::new("hdiutil")
                .args(["detach", &mount_point.to_string_lossy(), "-force"])
                .output();
                
            // Also try umount -f just in case
            let _ = Command::new("umount")
                .args(["-f", &mount_point.to_string_lossy()])
                .output();
        }
    }
    
    // Remove ScummVM directory
    if scummvm_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&scummvm_dir) {
            println!("[scummvm_service] Failed to remove directory: {}. Checking for stuck mounts...", e);
            
            // If we hit read-only FS, it's definitely a mount issue.
            // Let's look for ANY mounts inside our directory
            #[cfg(target_os = "macos")]
            {
                let mount_output = Command::new("mount").output().map_err(|e| e.to_string())?;
                let mount_str = String::from_utf8_lossy(&mount_output.stdout);
                
                let scummvm_path_str = scummvm_dir.to_string_lossy();
                
                for line in mount_str.lines() {
                    if let Some(path_part) = line.split(" on ").nth(1) {
                        let mount_path = path_part.split(" (").next().unwrap_or("").trim();
                        if mount_path.starts_with(scummvm_path_str.as_ref()) {
                            println!("[scummvm_service] Found stuck mount at: {}", mount_path);
                            
                            // Force detach this specific mount
                            let _ = Command::new("hdiutil")
                                .args(["detach", mount_path, "-force"])
                                .output();
                                
                            let _ = Command::new("umount")
                                .args(["-f", mount_path])
                                .output();
                        }
                    }
                }
                
                // Try removal one more time
                std::fs::remove_dir_all(&scummvm_dir)
                    .map_err(|e| format!("Failed to remove ScummVM directory after cleanup: {}", e))?;
            }
            
            #[cfg(not(target_os = "macos"))]
            {
                return Err(format!("Failed to remove ScummVM directory: {}", e));
            }
        }
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

pub async fn open_scummvm_app(
    db: &DatabaseConnection,
    app_data_dir: &PathBuf,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    #[allow(unused_variables)]
    let scummvm_path = get_or_download_scummvm(db, app_data_dir, app_handle).await?;
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, open the .app bundle, not the executable inside
        let scummvm_dir = app_data_dir.join("scummvm");
        let app_bundle = scummvm_dir.join("ScummVM.app");
        
        if !app_bundle.exists() {
            return Err("ScummVM.app not found".to_string());
        }
        
        // Verify app bundle structure
        let executable_path = app_bundle.join("Contents").join("MacOS").join("scummvm");
        if !executable_path.exists() {
            return Err("ScummVM executable not found in app bundle".to_string());
        }
        
        // Remove quarantine attribute before opening (handles existing installations)
        remove_quarantine_attribute(&app_bundle);
        
        // Try launching executable directly first (more reliable than app bundle)
        // If that fails, fall back to open command
        let executable_path = app_bundle.join("Contents").join("MacOS").join("scummvm");
        if executable_path.exists() {
            // Launch executable directly - this bypasses some Gatekeeper issues
            Command::new(&executable_path)
                .spawn()
                .map_err(|e| format!("Failed to launch ScummVM executable: {}", e))?;
        } else {
            // Fallback to app bundle launch
            Command::new("open")
                .arg(&app_bundle)
                .spawn()
                .map_err(|e| format!("Failed to open ScummVM app: {}", e))?;
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // On Windows and Linux, launch the executable directly
        if !Path::new(&scummvm_path).exists() {
            return Err("ScummVM executable not found".to_string());
        }
        
        Command::new(&scummvm_path)
            .spawn()
            .map_err(|e| format!("Failed to launch ScummVM: {}", e))?;
    }
    
    Ok(())
}

pub async fn open_scummvm_folder(
    app_data_dir: &PathBuf,
) -> Result<(), String> {
    let scummvm_dir = app_data_dir.join("scummvm");
    
    if !scummvm_dir.exists() {
        return Err("ScummVM directory does not exist".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&scummvm_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&scummvm_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&scummvm_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        return Err("Unsupported platform".to_string());
    }

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
        let exe_path = scummvm_dir.join("ScummVM.app").join("Contents").join("MacOS").join("scummvm");
        return Ok(exe_path.to_string_lossy().to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported platform".to_string())
    }
}

async fn get_download_url_and_name() -> Result<(String, String), String> {
    let version = fetch_latest_scummvm_version().await?;
    
    #[cfg(target_os = "windows")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-win32.zip",
            version, version
        );
        let name = format!("scummvm-{}-win32.zip", version);
        return Ok((url, name));
    }

    #[cfg(target_os = "linux")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-linux-x86_64.tar.xz",
            version, version
        );
        let name = format!("scummvm-{}-linux.tar.xz", version);
        return Ok((url, name));
    }

    #[cfg(target_os = "macos")]
    {
        let url = format!(
            "https://www.scummvm.org/frs/scummvm/{}/scummvm-{}-macosx.dmg",
            version, version
        );
        let name = format!("scummvm-{}-macosx.dmg", version);
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
        use crate::services::file_service;
        // Extract ZIP to temp, then move entire contents to dest_dir
        let temp_extract = dest_dir.join("temp_extract");
        std::fs::create_dir_all(&temp_extract)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        
        file_service::extract_zip(archive_path, &temp_extract)?;
        
        // Find the ScummVM directory (contains scummvm.exe) and move ALL contents
        for entry in std::fs::read_dir(&temp_extract)
            .map_err(|e| format!("Failed to read temp directory: {}", e))? 
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Check if this is the ScummVM directory
                let exe_path = path.join("scummvm.exe");
                if exe_path.exists() {
                    // Copy ALL files from this directory to dest_dir
                    for file_entry in std::fs::read_dir(&path)
                        .map_err(|e| format!("Failed to read ScummVM directory: {}", e))?
                    {
                        let file_entry = file_entry.map_err(|e| format!("Failed to read file: {}", e))?;
                        let src = file_entry.path();
                        let dest = dest_dir.join(file_entry.file_name());
                        
                        if src.is_dir() {
                            // Copy directory recursively
                            copy_dir_recursive(&src, &dest)?;
                        } else {
                            std::fs::copy(&src, &dest)
                                .map_err(|e| format!("Failed to copy {}: {}", src.display(), e))?;
                        }
                    }
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
        use std::process::Command;
        
        // Create temporary mount point
        let mount_point = dest_dir.join("temp_mount");
        std::fs::create_dir_all(&mount_point)
            .map_err(|e| format!("Failed to create mount point: {}", e))?;
        
        // Mount DMG
        let mount_output = Command::new("hdiutil")
            .args([
                "attach",
                "-nobrowse",
                "-mountpoint",
                &mount_point.to_string_lossy(),
                &archive_path.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute hdiutil attach: {}", e))?;
        
        if !mount_output.status.success() {
            let _ = std::fs::remove_dir_all(&mount_point);
            return Err(format!(
                "Failed to mount DMG: {}",
                String::from_utf8_lossy(&mount_output.stderr)
            ));
        }
        
        // Find ScummVM.app in mounted volume
        let mut scummvm_app_path: Option<PathBuf> = None;
        for entry in std::fs::read_dir(&mount_point)
            .map_err(|e| format!("Failed to read mount point: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                let file_name = path.file_name().and_then(|n| n.to_str());
                if file_name == Some("ScummVM.app") {
                    scummvm_app_path = Some(path.clone());
                    break;
                }
            }
        }
        
        let scummvm_app = scummvm_app_path.ok_or("ScummVM.app not found in DMG")?;
        
        // Copy ScummVM.app to destination using ditto (preserves all attributes, symlinks, etc.)
        let dest_app = dest_dir.join("ScummVM.app");
        
        // Remove existing app if it exists
        if dest_app.exists() {
            std::fs::remove_dir_all(&dest_app)
                .map_err(|e| format!("Failed to remove existing app: {}", e))?;
        }
        
        // Use ditto to copy app bundle (preserves permissions, symlinks, extended attributes)
        let ditto_output = Command::new("ditto")
            .arg(&scummvm_app)
            .arg(&dest_app)
            .output()
            .map_err(|e| format!("Failed to execute ditto: {}", e))?;
        
        if !ditto_output.status.success() {
            return Err(format!(
                "Failed to copy ScummVM.app: {}",
                String::from_utf8_lossy(&ditto_output.stderr)
            ));
        }
        
        // Remove quarantine attribute (macOS Gatekeeper)
        remove_quarantine_attribute(&dest_app);
        
        // Unmount DMG
        let detach_output = Command::new("hdiutil")
            .args(["detach", &mount_point.to_string_lossy()])
            .output()
            .map_err(|e| format!("Failed to execute hdiutil detach: {}", e))?;
        
        // Clean up mount point
        let _ = std::fs::remove_dir_all(&mount_point);
        
        if !detach_output.status.success() {
            // Log warning but don't fail - DMG might already be unmounted
            println!(
                "Warning: Failed to unmount DMG: {}",
                String::from_utf8_lossy(&detach_output.stderr)
            );
        }
        
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = (archive_path, dest_dir); // Acknowledge parameters for unused warning
        Err("Unsupported platform".to_string())
    }
}

#[cfg(target_os = "windows")]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;

    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn remove_quarantine_attribute(app_path: &Path) {
    use std::process::Command;
    
    // Only remove quarantine attribute, preserve everything else (including signatures)
    let xattr_output = Command::new("xattr")
        .args(["-d", "com.apple.quarantine", &app_path.to_string_lossy()])
        .output();
    
    match xattr_output {
        Ok(output) if output.status.success() => {
            println!("[scummvm_service] Removed quarantine attribute from ScummVM.app");
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Only warn if it's not "No such xattr" error (which is fine)
            if !stderr.contains("No such xattr") {
                println!(
                    "[scummvm_service] Warning: Could not remove quarantine attribute: {}",
                    stderr
                );
            }
        }
        Err(e) => {
            println!(
                "[scummvm_service] Warning: Failed to execute xattr: {}",
                e
            );
        }
    }
    
    // Verify executable exists and check its architecture
    let executable_path = app_path.join("Contents").join("MacOS").join("scummvm");
    if executable_path.exists() {
        // Set executable permissions
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&executable_path).map(|m| m.permissions()) {
            perms.set_mode(0o755);
            if let Err(e) = std::fs::set_permissions(&executable_path, perms) {
                println!("[scummvm_service] Warning: Could not set executable permissions: {}", e);
            }
        }
        
        // Check executable architecture
        let file_output = Command::new("file")
            .arg(&executable_path)
            .output();
        
        if let Ok(output) = file_output {
            let info = String::from_utf8_lossy(&output.stdout);
            println!("[scummvm_service] Executable info: {}", info);
        }
    } else {
        println!("[scummvm_service] Warning: Executable not found in app bundle");
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

#[cfg(test)]
mod kyrandia_tests {
    use super::*;

    #[test]
    fn patched_toon_runtime_is_optional_and_isolated() {
        let root = tempfile::tempdir().unwrap();
        let standard = root.path().join("standard-scummvm");
        assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), standard);
        let directory = root.path().join("scummvm-toon-hebrew");
        std::fs::create_dir(&directory).unwrap();
        let patched = directory.join(if cfg!(target_os = "windows") { "scummvm.exe" } else { "scummvm" });
        std::fs::write(&patched, b"fixture").unwrap();
        assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), standard);
        std::fs::write(directory.join("toon.dat"), b"fixture").unwrap();
        assert_eq!(game_runtime_path(root.path(), "toonstruck", standard.clone()), patched);
        assert_eq!(game_runtime_path(root.path(), "pajama", standard.clone()), standard);
        let mut cmd = Command::new("scummvm");
        add_toon_runtime_data(&mut cmd, root.path(), "toonstruck");
        assert_eq!(cmd.get_args().next().unwrap().to_string_lossy(),
            format!("--extrapath={}", directory.display()));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn drifter_launches_native_bundle_from_both_detected_and_manual_paths() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("TheDrifter.app");
        assert!(drifter_launch_command(root.path()).is_err());
        std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        std::fs::create_dir_all(bundle.join("Contents/Resources/Data")).unwrap();
        std::fs::write(bundle.join("Contents/MacOS/The Drifter"), b"runner").unwrap();
        std::fs::write(bundle.join("Contents/Resources/Data/globalgamemanagers"), b"data").unwrap();
        for path in [root.path(), bundle.as_path()] {
            let cmd = drifter_launch_command(path).unwrap();
            assert_eq!(cmd.get_program(), "/usr/bin/open");
            assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec![bundle.canonicalize().unwrap().as_os_str()]);
            assert_eq!(cmd.get_current_dir(), Some(root.path().canonicalize().unwrap().as_path()));
        }
    }


    #[test]
    fn pajama_subtitles_do_not_depend_on_saved_launch_args() {
        let mut cmd = Command::new("scummvm");
        add_required_game_options(&mut cmd, "pajama");
        cmd.args(["--language=he", "--fullscreen", "--auto-detect"]);
        assert!(cmd.get_args().any(|arg| arg == "--subtitles"));

        let mut other_game = Command::new("scummvm");
        add_required_game_options(&mut other_game, "kyra1-cd");
        assert_eq!(other_game.get_args().count(), 0);
    }

    #[test]
    fn extra_path_is_a_single_long_option_including_spaces() {
        let mut cmd = Command::new("scummvm");
        add_game_paths(&mut cmd, Path::new("/launch folder"), Some(Path::new("/Game.app/game data")));
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), vec![
            std::ffi::OsStr::new("-p"), std::ffi::OsStr::new("/launch folder"),
            std::ffi::OsStr::new("--extrapath=/Game.app/game data")]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn immediate_launch_failure_reaches_the_caller() {
        let log = std::env::temp_dir().join(format!("kitor-failed-launch-{}.log", std::process::id()));
        let mut cmd = Command::new("/bin/sh");
        cmd.args(["-c", "echo 'invalid launch option' >&2; exit 1"]);
        let error = start_game_process(&mut cmd, &log).await.unwrap_err();
        assert!(error.contains("invalid launch option"));
        std::fs::remove_file(log).unwrap();
    }

    #[test]
    fn launch_signature_is_isolated_and_refreshed_after_patching() {
        let root = std::env::temp_dir().join(format!("kitor-kyra-launch-{}-{}",
            std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
        let data = root.join("data");
        let launch = root.join("launch");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::write(data.join("gemcut.pak"), b"original").unwrap();
        std::fs::write(data.join("FATE.PAK"), b"conflicting signature").unwrap();
        prepare_kyrandia_launch_path(&data, &launch).unwrap();
        assert_eq!(std::fs::read(launch.join("GEMCUT.PAK")).unwrap(), b"original");
        assert!(!launch.join("FATE.PAK").exists());
        std::fs::write(data.join("gemcut.pak"), b"patched").unwrap();
        prepare_kyrandia_launch_path(&data, &launch).unwrap();
        assert_eq!(std::fs::read(launch.join("GEMCUT.PAK")).unwrap(), b"patched");
        assert!(data.join("FATE.PAK").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod collection_launch_tests {
    use super::normalize_collection_args;
    #[test]
    fn patched_sq1_vga_enables_hebrew_with_stale_settings() {
        for args in [vec![], vec!["--language=en".into()], vec!["--language=he".into()]] {
            assert_eq!(normalize_collection_args("sq1sci", args, true),
                vec!["--language=he", "--platform=dos", "sci:sq1sci"]);
        }
        assert_eq!(normalize_collection_args("sq1sci", vec!["--language=he".into()], false),
            vec!["--platform=dos", "sci:sq1sci"]);
    }

    #[test]
    fn original_and_patched_collection_launches_select_the_correct_language() {
        let args = vec!["--language=he".into(), "--fullscreen".into(), "--auto-detect".into()];
        assert_eq!(normalize_collection_args("sq1", args.clone(), false), vec!["--fullscreen", "--platform=dos", "agi:sq1"]);
        assert_eq!(normalize_collection_args("sq1", args.clone(), true), vec!["--language=he", "--fullscreen", "--platform=dos", "agi:sq1"]);
        assert_eq!(normalize_collection_args("monkey2-se", args.clone(), false), args);
    }
}
