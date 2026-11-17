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
    catalog_id: Option<String>,
) -> Result<EditorData, String> {
    let config = contributions::csv_text::catalog(catalog_id.as_deref().unwrap_or("drifter"))?;
    let _lock = state.gate.lock().await;
    contributions::open(&account_root(&app, &contributor_id)?, config).await
}
#[tauri::command]
pub async fn save_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
    catalog_id: Option<String>,
    revision: u64,
    edits: BTreeMap<String, String>,
) -> Result<u64, String> {
    let config = contributions::csv_text::catalog(catalog_id.as_deref().unwrap_or("drifter"))?;
    let _lock = state.gate.lock().await;
    contributions::save(
        &account_root(&app, &contributor_id)?,
        config,
        revision,
        edits,
    )
}
#[tauri::command]
pub async fn preview_translation_submission(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
    catalog_id: Option<String>,
) -> Result<Preview, String> {
    let config = contributions::csv_text::catalog(catalog_id.as_deref().unwrap_or("drifter"))?;
    let _lock = state.gate.lock().await;
    contributions::preview(&account_root(&app, &contributor_id)?, config)
}
#[tauri::command]
pub async fn submit_translation_draft(
    app: State<'_, AppState>,
    state: State<'_, Contributions>,
    contributor_id: String,
    catalog_id: Option<String>,
    revision: u64,
    digest: String,
    email_opt_in: Option<bool>,
) -> Result<Submission, String> {
    let config = contributions::csv_text::catalog(catalog_id.as_deref().unwrap_or("drifter"))?;
    let _lock = state.gate.lock().await;
    let token = contributions::identity::token_for(&contributor_id)?;
    contributions::submit(
        &account_root(&app, &contributor_id)?,
        config,
        revision,
        &digest,
        &token,
        email_opt_in.unwrap_or(false),
    )
    .await
}
#[tauri::command]
pub async fn contribution_bot_status() -> Result<contributions::BotStatus, String> {
    contributions::bot_status().await
}
