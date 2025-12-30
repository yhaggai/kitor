use tauri::State;
use crate::services::scummvm_service;
use crate::AppState;

#[tauri::command]
pub async fn launch_game(game_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    scummvm_service::launch_game(&state.db, game_id).await
}

#[tauri::command]
pub fn get_scummvm_path() -> Result<String, String> {
    scummvm_service::get_scummvm_path()
}

#[tauri::command]
pub fn verify_scummvm() -> bool {
    scummvm_service::verify_scummvm_installation()
}




