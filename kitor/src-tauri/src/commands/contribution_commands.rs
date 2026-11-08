use crate::{
    services::contributions::{self, Contributions, EditorData, Preview, Submission},
    AppState,
};
use std::collections::BTreeMap;
use tauri::State;

#[tauri::command]
pub async fn open_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
) -> Result<EditorData, String> {
    let _lock = state.gate.lock().await;
    contributions::open(&app.app_data_dir).await
}
#[tauri::command]
pub async fn save_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    revision: u64,
    edits: BTreeMap<String, String>,
) -> Result<u64, String> {
    let _lock = state.gate.lock().await;
    contributions::save(&app.app_data_dir, revision, edits)
}
#[tauri::command]
pub async fn preview_translation_submission(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
) -> Result<Preview, String> {
    let _lock = state.gate.lock().await;
    contributions::preview(&app.app_data_dir)
}
#[tauri::command]
pub async fn submit_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    revision: u64,
    digest: String,
) -> Result<Submission, String> {
    let _lock = state.gate.lock().await;
    contributions::submit(&app.app_data_dir, revision, &digest).await
}
#[tauri::command]
pub async fn contribution_bot_status() -> Result<contributions::BotStatus, String> {
    contributions::bot_status().await
}
