use tauri::State;
use crate::models::game;
use crate::services::game_service::{self, DetectedGameInfo, GameWithPatches};
use crate::services::image_service;
use crate::services::registry_service;
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
pub async fn refresh_cover_image(game_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let game = game_service::get_game_by_id(&state.db, game_id)
        .await?
        .ok_or_else(|| "Game not found".to_string())?;

    let registry = registry_service::load_registry()?;
    let cover_url = registry
        .games
        .iter()
        .find(|g| g.scummvm_id == game.scummvm_id)
        .and_then(|g| g.cover_url.clone())
        .ok_or_else(|| "No cover_url for this game in registry".to_string())?;

    if let Some(existing) = image_service::get_cached_cover_path(&game.scummvm_id, &state.app_data_dir)? {
        let _ = std::fs::remove_file(existing);
    }

    let _ = image_service::download_and_cache_cover(&game.scummvm_id, &cover_url, state.app_data_dir.clone()).await?;
    Ok(())
}

#[tauri::command]
pub async fn clear_cover_cache(state: State<'_, AppState>) -> Result<(), String> {
    image_service::clear_cover_cache(&state.app_data_dir)?;
    Ok(())
}





