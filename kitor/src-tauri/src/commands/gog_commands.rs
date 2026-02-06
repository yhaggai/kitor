use tauri::{AppHandle, Emitter, Manager, State};
use crate::services::{gog_service, http_server};
use crate::AppState;

#[tauri::command]
pub async fn login_to_gog(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Start local callback server
    let (rx, _port) = http_server::start_callback_server()?;
    
    let auth_url = gog_service::build_auth_url();
    
    // Create login window
    tauri::WebviewWindowBuilder::new(
        &app_handle,
        "gog-login",
        tauri::WebviewUrl::External(auth_url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title("Login to GOG")
    .inner_size(800.0, 600.0)
    .initialization_script(r#"
        // Check for the code parameter in the URL
        const checkCode = () => {
            const url = window.location.href;
            if (url.includes('code=')) {
                // Extract code regardless of the domain we are on
                const urlParams = new URLSearchParams(window.location.search);
                const code = urlParams.get('code');
                
                if (code) {
                    // Redirect to our local server which is listening for this
                    window.location.replace(`http://localhost:7463/gog-callback?code=${code}`);
                }
            }
        };

        // Check immediately and then every 500ms
        checkCode();
        setInterval(checkCode, 500);
    "#)
    .center()
    .build()
    .map_err(|e| format!("Failed to create login window: {}", e))?;
    
    // Wait for callback in background
    let app_handle_clone = app_handle.clone();
    let db = state.db.clone();
    
    tauri::async_runtime::spawn(async move {
        // Wait for code from callback server (with timeout)
        if let Ok(code) = rx.recv_timeout(std::time::Duration::from_secs(300)) {
            // Exchange code for token
            match gog_service::exchange_code_for_token(&code).await {
                Ok(token_response) => {
                    match gog_service::store_tokens(
                        &db,
                        &token_response.access_token,
                        &token_response.refresh_token,
                        token_response.expires_in,
                    ).await {
                        Ok(_) => {
                            let _ = app_handle_clone.emit("gog-login-success", ());
                            // Close login window
                            if let Some(window) = app_handle_clone.get_webview_window("gog-login") {
                                let _ = window.close();
                            }
                        }
                        Err(e) => {
                            let _ = app_handle_clone.emit("gog-login-error", e);
                        }
                    }
                }
                Err(e) => {
                    let _ = app_handle_clone.emit("gog-login-error", e);
                }
            }
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
