use sea_orm::DatabaseConnection;
use std::path::PathBuf;
use tauri::Manager;

mod db;
mod models;
mod services;
mod commands;

pub struct AppState {
    pub db: DatabaseConnection,
    pub app_data_dir: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize database
            let db = tauri::async_runtime::block_on(async {
                db::init_db(&app_data_dir)
                    .await
                    .expect("Failed to initialize database")
            });

            // Store state
            app.manage(AppState {
                db,
                app_data_dir,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Game commands
            commands::game_commands::get_all_games,
            commands::game_commands::scan_games,
            commands::game_commands::add_game_manually,
            commands::game_commands::remove_game,
            commands::game_commands::get_game_by_id,
            // ScummVM commands
            commands::scummvm_commands::launch_game,
            commands::scummvm_commands::get_scummvm_path,
            commands::scummvm_commands::verify_scummvm,
            // Patch commands
            commands::patch_commands::get_patches_for_game,
            commands::patch_commands::get_all_patches,
            commands::patch_commands::download_and_install_patch,
            commands::patch_commands::get_installed_patches,
            commands::patch_commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

