use sea_orm::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::models::game;
use crate::services::game_service;

pub async fn launch_game(
    db: &DatabaseConnection,
    game_id: i32,
) -> Result<(), String> {
    let game = game_service::get_game_by_id(db, game_id)
        .await?
        .ok_or("Game not found")?;

    println!("=== LAUNCHING GAME ===");
    println!("Game ID: {}", game.id);
    println!("ScummVM ID: {}", game.scummvm_id);
    println!("Title: {}", game.title);
    println!("Install Path: {}", game.install_path);

    let scummvm_path = get_scummvm_path()?;
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

pub fn get_scummvm_path() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let common_paths = vec![
            "C:\\Program Files\\ScummVM\\scummvm.exe",
            "C:\\Program Files (x86)\\ScummVM\\scummvm.exe",
        ];

        for path in common_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        // Check PATH environment variable
        if let Ok(output) = Command::new("where").arg("scummvm.exe").output() {
            if output.status.success() {
                if let Ok(path_str) = String::from_utf8(output.stdout) {
                    let path = path_str.trim();
                    if !path.is_empty() {
                        return Ok(path.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let common_paths = vec![
            "/usr/bin/scummvm",
            "/usr/local/bin/scummvm",
            "/usr/games/scummvm",
        ];

        for path in common_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let common_paths = vec![
            "/Applications/ScummVM.app/Contents/MacOS/scummvm",
            "/usr/local/bin/scummvm",
        ];

        for path in common_paths {
            if Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }
    }

    Err("ScummVM not found. Please install ScummVM or configure the path manually.".to_string())
}

pub fn verify_scummvm_installation() -> bool {
    get_scummvm_path().is_ok()
}








