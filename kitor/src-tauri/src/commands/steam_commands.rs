use crate::{
    services::steam_service::{self, Status, SteamState},
    AppState,
};
use tauri::State;

#[tauri::command]
pub async fn get_steam_status() -> Result<Status, String> {
    kitor_steam::owned_games().await.map(Status::native).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_steam_api_key(
    api_key: String,
    steam: State<'_, SteamState>,
) -> Result<Status, String> {
    {
        let mut session = steam.session.lock().await;
        if session.connecting {
            return Err("Finish or cancel Steam sign-in first".into());
        }
        steam_service::persist_key(api_key.clone()).await?;
        steam_service::set_key(&mut session, api_key)?;
    }
    Ok(steam_service::refresh(&steam).await)
}
#[tauri::command]
pub async fn refresh_steam_library() -> Result<Status, String> {
    get_steam_status().await
}

#[tauri::command]
pub async fn connect_steam(
    steam: State<'_, SteamState>,
    app: State<'_, AppState>,
) -> Result<Status, String> {
    // Subscribe before setting connecting so cancellation cannot be missed.
    let cancelled = steam.cancel.notified();
    tokio::pin!(cancelled);
    cancelled.as_mut().enable();
    let generation = {
        let mut session = steam.session.lock().await;
        if session.connecting {
            return Err("Steam sign-in is already in progress".into());
        }
        session.begin_login();
        session.generation
    };
    let result = tokio::select! {
        result = steam_service::login() => result,
        _ = &mut cancelled => Err("Steam sign-in was cancelled".into()),
    };
    {
        let mut session = steam.session.lock().await;
        if session.generation != generation {
            return Err("Steam sign-in was cancelled".into());
        }
        session.connecting = false;
        let id = result?;
        steam_service::save_identity(&app.app_data_dir, &id)?;
        session.steam_id = Some(id);
    }
    Ok(steam_service::refresh(&steam).await)
}
#[tauri::command]
pub async fn disconnect_steam(
    steam: State<'_, SteamState>,
    app: State<'_, AppState>,
) -> Result<Status, String> {
    let mut session = steam.session.lock().await;
    steam_service::delete_key().await?;
    steam_service::clear_session(&mut session);
    steam.cancel.notify_waiters();
    steam_service::clear_identity(&app.app_data_dir)?;
    Ok(session.status())
}

#[tauri::command]
pub async fn download_steam_windows_game(
    app_id: u32,
    install_root: Option<String>,
    game_ids: Vec<String>,
    overwrite: Option<bool>,
    app_handle: tauri::AppHandle,
    app: State<'_, AppState>,
    updates: State<'_, crate::services::app_updates::AppUpdates>,
) -> Result<Vec<crate::models::game::Model>, String> {
    let _activity = updates.activity()?;
    crate::services::steam_download_service::download_to_confirmed(app_id, game_ids, &app, install_root.map(std::path::PathBuf::from), overwrite.unwrap_or(false), Some(&app_handle)).await
}

#[tauri::command]
pub fn get_steam_download_capabilities(app_id: u32, game_id: String) -> crate::services::steam_download_service::DownloadCapabilities {
    crate::services::steam_download_service::capabilities(app_id, &game_id)
}

static NATIVE_LOGIN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static NATIVE_CANCEL: tokio::sync::Notify = tokio::sync::Notify::const_new();

#[tauri::command]
pub async fn get_native_steam_status() -> Result<bool, String> {
    kitor_steam::connected().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn connect_native_steam(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    let _guard = NATIVE_LOGIN.try_lock().map_err(|_| "Steam download sign-in is already running")?;
    let cancel = NATIVE_CANCEL.notified();
    tokio::pin!(cancel);
    cancel.as_mut().enable();
    tokio::select! {
        result = kitor_steam::connect(|event| { let _ = app.emit("native-steam-auth", event); }) => result.map_err(|e| e.to_string()),
        _ = cancel => Err("Steam download sign-in cancelled".into()),
    }
}

#[tauri::command]
pub async fn cancel_native_steam_login() {
    NATIVE_CANCEL.notify_waiters();
}

#[tauri::command]
pub async fn disconnect_native_steam() -> Result<(), String> {
    NATIVE_CANCEL.notify_waiters();
    let _guard = NATIVE_LOGIN.lock().await;
    kitor_steam::disconnect().await.map_err(|e| e.to_string())
}

static NATIVE_CODES: tokio::sync::Mutex<Option<tokio::sync::mpsc::Sender<String>>> = tokio::sync::Mutex::const_new(None);

#[tauri::command]
pub async fn connect_native_steam_password(username: String, password: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    let _guard = NATIVE_LOGIN.try_lock().map_err(|_| "Steam download sign-in is already running")?;
    let cancel = NATIVE_CANCEL.notified();
    tokio::pin!(cancel);
    cancel.as_mut().enable();
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    *NATIVE_CODES.lock().await = Some(sender);
    let result = tokio::select! {
        result = kitor_steam::connect_password(username, password, receiver, |event| { let _ = app.emit("native-steam-auth", event); }) => result.map_err(|e| e.to_string()),
        _ = cancel => Err("Steam download sign-in cancelled".into()),
    };
    *NATIVE_CODES.lock().await = None;
    result
}

#[tauri::command]
pub async fn submit_native_steam_guard(code: String) -> Result<(), String> {
    let code = code.trim();
    if !(4..=8).contains(&code.len()) || !code.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err("Enter a valid Steam Guard code.".into());
    }
    NATIVE_CODES.lock().await.as_ref().ok_or("Steam sign-in has expired")?
        .try_send(code.to_uppercase()).map_err(|_| "Steam is processing the previous code. Please wait.".to_string())
}
