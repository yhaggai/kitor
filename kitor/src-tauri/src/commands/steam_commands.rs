use crate::{
    services::steam_service::{self, Status, SteamState},
    AppState,
};
use tauri::State;

#[tauri::command]
pub async fn get_steam_status(steam: State<'_, SteamState>) -> Result<Status, String> {
    if steam_service::restore_key(&steam).await {
        return Ok(steam_service::refresh(&steam).await);
    }
    Ok(steam.session.lock().await.status())
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
pub async fn refresh_steam_library(steam: State<'_, SteamState>) -> Result<Status, String> {
    steam_service::restore_key(&steam).await;
    Ok(steam_service::refresh(&steam).await)
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
    app: State<'_, AppState>,
    updates: State<'_, crate::services::app_updates::AppUpdates>,
) -> Result<Vec<crate::models::game::Model>, String> {
    let _activity = updates.activity()?;
    crate::services::steam_download_service::download_to_confirmed(app_id, game_ids, &app, install_root.map(std::path::PathBuf::from), overwrite.unwrap_or(false)).await
}

#[tauri::command]
pub fn get_steam_download_capabilities(app_id: u32, game_id: String) -> crate::services::steam_download_service::DownloadCapabilities {
    crate::services::steam_download_service::capabilities(app_id, &game_id)
}
