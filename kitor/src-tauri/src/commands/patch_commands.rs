use tauri::{AppHandle, State};
use crate::models::installed_patch;
use crate::services::{patch_service, registry_service};
use crate::AppState;
use std::path::Path;

#[tauri::command]
pub async fn refresh_patch_registry() -> Result<(), String> {
    crate::services::registry_update_service::refresh().await
}

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
#[tauri::command]
pub fn get_os() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "osx".to_string()
    } else {
        "linux".to_string()
    }
}

#[tauri::command]
pub async fn open_gog_client(game_id: i64) -> Result<(), String> {
    if game_id <= 0 {
        return Err("Invalid GOG game ID".to_string());
    }
    tauri_plugin_opener::open_url(
        format!("goggalaxy://openGameView/{}", game_id),
        None::<&str>,
    ).map_err(|e| format!("Could not open GOG Galaxy. Make sure it is installed: {}", e))
}

#[tauri::command]
pub async fn open_steam_client(app_id: u32, steam: State<'_, crate::services::steam_service::SteamState>) -> Result<(), String> {
    if app_id == 0 {
        return Err("Invalid Steam app ID".to_string());
    }
    crate::services::steam_service::restore_key(&steam).await;
    tauri_plugin_opener::open_url(
        crate::services::steam_service::game_url(&steam, app_id).await,
        None::<&str>,
    ).map_err(|e| format!("Could not open Steam. Make sure it is installed: {}", e))
}

#[tauri::command]
pub async fn download_and_extract_gog_installer(
    scummvm_id: String,
    installer_path: String,
    dest_dir: String,
    state: tauri::State<'_, crate::AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use std::path::PathBuf;
    crate::services::patch_service::download_and_extract_gog_installer(
        &state.db,
        scummvm_id,
        installer_path,
        PathBuf::from(dest_dir),
        app_handle,
        state.app_data_dir.clone(),
    ).await
}
