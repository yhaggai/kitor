use crate::services::app_updates::{self, AppUpdates, CheckResult, Progress};
use tauri::{ipc::Channel, AppHandle, State};

#[tauri::command]
pub async fn check_app_update(
    app: AppHandle,
    state: State<'_, AppUpdates>,
) -> Result<CheckResult, String> {
    app_updates::check(&app, &state).await
}
#[tauri::command]
pub async fn install_app_update(
    version: String,
    progress: Channel<Progress>,
    state: State<'_, AppUpdates>,
) -> Result<(), String> {
    app_updates::install(&state, &version, progress).await
}
#[tauri::command]
pub fn restart_after_app_update(
    app: AppHandle,
    state: State<'_, AppUpdates>,
) -> Result<(), String> {
    app_updates::restart(&app, &state)
}
