use tauri::{AppHandle, State};
use crate::models::installed_patch;
use crate::services::patch_service;
use crate::AppState;

#[tauri::command]
pub async fn get_patches_for_game(
    game_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<patch_service::PatchInfo>, String> {
    patch_service::get_patches_for_game(&state.db, game_id).await
}

#[tauri::command]
pub async fn get_all_patches(state: State<'_, AppState>) -> Result<Vec<patch_service::PatchInfo>, String> {
    patch_service::get_all_patches(&state.db).await
}

#[tauri::command]
pub async fn download_and_install_patch(
    scummvm_id: String,
    language: String,
    version: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    patch_service::download_and_install_patch(
        &state.db,
        scummvm_id,
        language,
        version,
        app_handle,
        state.app_data_dir.clone(),
    )
    .await
}

#[tauri::command]
pub async fn get_installed_patches(
    game_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<installed_patch::Model>, String> {
    patch_service::get_installed_patches(&state.db, game_id).await
}

#[tauri::command]
pub async fn restore_backup(
    installed_patch_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    patch_service::restore_backup(&state.db, installed_patch_id).await
}
