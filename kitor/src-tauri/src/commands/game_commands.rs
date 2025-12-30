use tauri::State;
use crate::models::game;
use crate::services::game_service::{self, DetectedGameInfo, GameWithPatches};
use crate::AppState;

#[tauri::command]
pub async fn get_all_games(state: State<'_, AppState>) -> Result<Vec<game::Model>, String> {
    game_service::get_all_games(&state.db).await
}

#[tauri::command]
pub async fn scan_games(
    scan_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<game::Model>, String> {
    game_service::scan_directory(&state.db, &scan_path).await
}

#[tauri::command]
pub async fn add_game_manually(
    scummvm_id: String,
    title: String,
    install_path: String,
    state: State<'_, AppState>,
) -> Result<game::Model, String> {
    game_service::add_game_manually(&state.db, scummvm_id, title, install_path).await
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
    game_service::get_games_with_patches(&state.db).await
}





