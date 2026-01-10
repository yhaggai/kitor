use tauri::State;
use crate::services::scummvm_service;
use crate::AppState;

#[tauri::command]
pub async fn launch_game(
    game_id: i32,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    scummvm_service::launch_game(&state.db, game_id, &state.app_data_dir, &app_handle).await
}

#[tauri::command]
pub async fn download_scummvm_binary(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    scummvm_service::download_scummvm(&state.db, &state.app_data_dir, &app_handle).await
}

#[tauri::command]
pub async fn check_scummvm_status(
    state: State<'_, AppState>,
) -> Result<scummvm_service::ScummVMStatus, String> {
    scummvm_service::check_scummvm_status(&state.db, &state.app_data_dir).await
}

#[tauri::command]
pub async fn uninstall_scummvm(
    state: State<'_, AppState>,
) -> Result<(), String> {
    scummvm_service::uninstall_scummvm(&state.db, &state.app_data_dir).await
}
