use crate::{
    services::contributions::{self, Contributions, EditorData, Preview, Submission},
    AppState,
};
use std::collections::BTreeMap;
use tauri::State;
fn account_root(app: &AppState, expected: &str) -> Result<std::path::PathBuf, String> {
    let user = contributions::identity::contributor()?;
    if user.id != expected {
        return Err("Account changed. Reopen your draft.".into());
    }
    Ok(app.app_data_dir.join("contributors").join(user.id))
}

#[tauri::command]
pub async fn open_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
) -> Result<EditorData, String> {
    let _lock = state.gate.lock().await;
    contributions::open(&account_root(&app, &contributor_id)?).await
}
#[tauri::command]
pub async fn save_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
    revision: u64,
    edits: BTreeMap<String, String>,
) -> Result<u64, String> {
    let _lock = state.gate.lock().await;
    contributions::save(&account_root(&app, &contributor_id)?, revision, edits)
}
#[tauri::command]
pub async fn preview_translation_submission(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
) -> Result<Preview, String> {
    let _lock = state.gate.lock().await;
    contributions::preview(&account_root(&app, &contributor_id)?)
}
#[tauri::command]
pub async fn submit_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
    revision: u64,
    digest: String,
) -> Result<Submission, String> {
    let _lock = state.gate.lock().await;
    let token = contributions::identity::token_for(&contributor_id)?;
    contributions::submit(
        &account_root(&app, &contributor_id)?,
        revision,
        &digest,
        &token,
    )
    .await
}
#[tauri::command]
pub async fn contribution_bot_status() -> Result<contributions::BotStatus, String> {
    contributions::bot_status().await
}
