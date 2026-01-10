use tauri::{AppHandle, State};
use crate::models::installed_patch;
use crate::services::{patch_service, registry_service};
use crate::AppState;
use std::path::Path;

#[tauri::command]
pub async fn get_patches_for_game(
    game_id: i32,
    state: State<'_, AppState>,
) -> Result<Vec<patch_service::PatchInfo>, String> {
    patch_service::get_patches_for_game(&state.db, game_id).await
}

#[tauri::command]
pub async fn get_all_patches(state: State<'_, AppState>) -> Result<Vec<patch_service::PatchInfo>, String> {
    patch_service::get_all_patches(&state.db, state.app_data_dir.clone()).await
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

#[tauri::command]
pub async fn get_purchase_guide(
    scummvm_id: String,
) -> Result<String, String> {
    let registry = registry_service::load_registry()?;
    
    let game = registry.games
        .iter()
        .find(|g| g.scummvm_id == scummvm_id)
        .ok_or("Game not found in registry")?;
    
    let guide_filename = game.purchase_guide
        .as_ref()
        .ok_or("No purchase guide available")?;
    
    let guide_path = if cfg!(debug_assertions) {
        Path::new("../patches/guides").join(guide_filename)
    } else {
        Path::new("patches/guides").join(guide_filename)
    };
    
    std::fs::read_to_string(&guide_path)
        .map_err(|e| format!("Failed to read guide: {}", e))
}
