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
    updates: State<'_, crate::services::app_updates::AppUpdates>,
) -> Result<(), String> {
    let _activity = updates.activity()?;
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
    updates: State<'_, crate::services::app_updates::AppUpdates>,
) -> Result<(), String> {
    let _activity = updates.activity()?;
    scummvm_service::uninstall_scummvm(&state.db, &state.app_data_dir).await
}

#[tauri::command]
pub async fn get_latest_scummvm_version() -> Result<String, String> {
    scummvm_service::get_latest_scummvm_version().await
}

#[tauri::command]
pub async fn open_scummvm(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    scummvm_service::open_scummvm_app(&state.db, &state.app_data_dir, &app_handle).await
}

#[tauri::command]
pub async fn open_scummvm_folder(
    state: State<'_, AppState>,
) -> Result<(), String> {
    scummvm_service::open_scummvm_folder(&state.app_data_dir).await
}
