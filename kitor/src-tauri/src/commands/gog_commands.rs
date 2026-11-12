use tauri::{AppHandle, Emitter, Manager, State};
use crate::services::gog_service;
use crate::AppState;
use std::sync::{Arc, Mutex};

#[tauri::command]
pub async fn login_to_gog(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("gog-login") {
        return window.set_focus().map_err(|e| e.to_string());
    }

    // Capture only GOG's final authorization redirect, never a provider's code.
    // A one-shot sender also prevents duplicate exchanges and reports cancellation.
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<String, String>>();
    let sender = Arc::new(Mutex::new(Some(tx)));
    let navigation_sender = sender.clone();
    let window = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "gog-login",
        tauri::WebviewUrl::External(gog_service::build_auth_url().parse()
            .map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title("Login to GOG")
    .inner_size(800.0, 600.0)
    .initialization_script(include_str!("gog_external_login.js"))
    .on_navigation(move |url| {
        if let Ok(code) = gog_service::code_from_callback_url(url.as_str()) {
            if let Some(tx) = navigation_sender.lock().unwrap().take() {
                let _ = tx.send(Ok(code));
            }
            return false;
        }
        // Cancel this navigation after handing the process URL to the browser.
        // The original GOG document stays alive to poll for completion.
        if gog_service::is_external_login_url(url) {
            if let Err(error) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
                if let Some(tx) = navigation_sender.lock().unwrap().take() {
                    let _ = tx.send(Err(format!("Failed to open sign-in browser: {}", error)));
                }
            }
            return false;
        }
        true
    })
    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
    .center()
    .build()
    .map_err(|e| format!("Failed to create login window: {}", e))?;

    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            if let Some(tx) = sender.lock().unwrap().take() {
                let _ = tx.send(Err("GOG sign-in was cancelled.".into()));
            }
        }
    });

    let db = state.db.clone();
    tauri::async_runtime::spawn(async move {
        let result = async {
            let code = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
                .await.map_err(|_| "GOG sign-in timed out. Please try again.".to_string())?
                .map_err(|_| "GOG sign-in was cancelled.".to_string())??;
            let token = gog_service::exchange_code_for_token(&code).await?;
            gog_service::store_tokens(&db, &token.access_token, &token.refresh_token, token.expires_in).await
        }.await;
        let _ = window.close();
        match result {
            Ok(()) => { let _ = app_handle.emit("gog-login-success", ()); }
            Err(error) => { let _ = app_handle.emit("gog-login-error", error); }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn complete_gog_login(
    code: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let token_response = gog_service::exchange_code_for_token(&code).await?;
    
    gog_service::store_tokens(
        &state.db,
        &token_response.access_token,
        &token_response.refresh_token,
        token_response.expires_in,
    )
    .await?;
    
    // Close login window
    if let Some(window) = app_handle.get_webview_window("gog-login") {
        let _ = window.close();
    }
    
    // Emit success event
    let _ = app_handle.emit("gog-login-success", ());
    
    Ok(())
}

#[tauri::command]
pub async fn check_gog_auth_status(state: State<'_, AppState>) -> Result<bool, String> {
    gog_service::is_authenticated(&state.db).await
}

#[tauri::command]
pub async fn logout_from_gog(state: State<'_, AppState>) -> Result<(), String> {
    gog_service::logout(&state.db).await
}

#[tauri::command]
pub async fn get_gog_user_games(
    state: State<'_, AppState>,
) -> Result<Vec<gog_service::GogGame>, String> {
    gog_service::get_user_games(&state.db).await
}

#[tauri::command]
pub async fn get_gog_game_installers(
    state: State<'_, AppState>,
    game_id: i64,
) -> Result<Vec<gog_service::OfflineInstaller>, String> {
    gog_service::get_game_installers(&state.db, game_id).await
}

#[tauri::command]
pub fn get_gog_install_capabilities(game_id: i64, scummvm_id: String) -> Result<bool, String> {
    crate::services::gog_download_service::capabilities(game_id, &scummvm_id)
}

#[tauri::command]
pub async fn install_gog_game(
    game_id: i64,
    install_root: Option<String>,
    scummvm_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
    updates: State<'_, crate::services::app_updates::AppUpdates>,
) -> Result<crate::models::game::Model, String> {
    let _activity = updates.activity()?;
    crate::services::gog_download_service::install(game_id, scummvm_id, &state, &app, install_root.map(std::path::PathBuf::from)).await
}

#[tauri::command]
pub fn get_game_install_root(state: State<'_, AppState>) -> String {
    state.app_data_dir.join("games").to_string_lossy().into_owned()
}
