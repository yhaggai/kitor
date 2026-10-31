use tauri::State;
use crate::models::game;
use crate::services::game_service::{self, DetectedGameInfo, GameWithPatches};
use crate::AppState;

#[tauri::command]
pub async fn open_game_folder(game_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let game = game_service::get_game_by_id(&state.db, game_id)
        .await?
        .ok_or_else(|| "Game not found".to_string())?;
    let path = std::path::Path::new(&game.install_path);
    if !path.is_dir() {
        return Err(format!("Game folder does not exist: {}", game.install_path));
    }
    // A game's install directory can itself be an .app bundle. The default
    // macOS opener launches bundles; explicitly ask Finder to browse them.
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("/usr/bin/open")
            .args(["-a", "Finder"])
            .arg(path)
            .output()
            .map_err(|error| format!("Failed to open game folder: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "Failed to open game folder: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        tauri_plugin_opener::open_path(path, None::<&str>)
            .map_err(|error| format!("Failed to open game folder: {error}"))
    }
}

#[tauri::command]
pub async fn get_all_games(state: State<'_, AppState>) -> Result<Vec<game::Model>, String> {
    game_service::get_all_games(&state.db).await
}

#[tauri::command]
pub async fn scan_games(
    scan_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<game::Model>, String> {
    game_service::scan_directory(&state.db, &scan_path, state.app_data_dir.clone()).await
}

#[tauri::command]
pub async fn add_game_manually(
    scummvm_id: String,
    title: String,
    install_path: String,
    state: State<'_, AppState>,
) -> Result<game::Model, String> {
    game_service::add_game_manually(&state.db, scummvm_id, title, install_path, state.app_data_dir.clone()).await
}

#[tauri::command]
pub async fn detect_game_in_folder(path: String) -> Result<Option<DetectedGameInfo>, String> {
    game_service::detect_game_in_folder(std::path::Path::new(&path))
}

#[tauri::command]
pub async fn remove_game(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    game_service::remove_game(&state.db, id).await
}

#[tauri::command]
pub async fn get_game_by_id(
    id: i32,
    state: State<'_, AppState>,
) -> Result<Option<game::Model>, String> {
    game_service::get_game_by_id(&state.db, id).await
}

#[tauri::command]
pub async fn get_games_with_patches(state: State<'_, AppState>) -> Result<Vec<GameWithPatches>, String> {
    game_service::get_games_with_patches(&state.db, state.app_data_dir.clone()).await
}






#[tauri::command]
pub async fn scan_installed_games(state: State<'_, AppState>) -> Result<crate::services::game_discovery_service::ScanResult, String> {
    crate::services::game_discovery_service::scan_installed_games(&state.db, state.app_data_dir.clone()).await
}

#[tauri::command]
pub async fn refresh_game(game_id: i32, state: State<'_, AppState>) -> Result<crate::services::game_refresh_service::GameHealth, String> {
    crate::services::game_refresh_service::refresh(game_id, &state).await
}
