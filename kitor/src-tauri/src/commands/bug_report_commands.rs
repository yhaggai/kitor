use tauri_plugin_dialog::DialogExt;

/// Save through the native picker so email attachments also work in desktop webviews.
#[tauri::command]
pub async fn save_bug_report(app: tauri::AppHandle, draft: String) -> Result<bool, String> {
    if draft.len() > 15 * 1024 * 1024 {
        return Err("The report is too large. Remove a screenshot and try again.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = app.dialog().file()
            .set_title("Save bug report email")
            .set_file_name("kitor-bug-report.eml")
            .add_filter("Email draft", &["eml"])
            .blocking_save_file() else {
                return Ok(false);
            };
        let path = file.into_path().map_err(|e| e.to_string())?;
        std::fs::write(path, draft).map_err(|e| format!("Could not save report: {e}"))?;
        Ok(true)
    }).await.map_err(|e| e.to_string())?
}

/// Explicit allowlist: do not serialize database models, which contain personal paths.
#[tauri::command]
pub async fn collect_bug_diagnostics(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    use crate::services::{game_service, patch_service, scummvm_service, diagnostics};
    let mut games = Vec::new();
    for game in game_service::get_all_games(&state.db).await? {
        let patches = patch_service::get_installed_patches(&state.db, game.id).await?;
        games.push(serde_json::json!({
            "id": game.id, "game": game.scummvm_id,
            "install_directory_exists": std::path::Path::new(&game.install_path).is_dir(),
            "patches": patches.iter().map(|patch| serde_json::json!({
                "version": patch.version, "status": patch.status,
                "backup_exists": patch.backup_path.as_ref().map(|p| std::path::Path::new(p).exists())
            })).collect::<Vec<_>>()
        }));
    }
    let runtime = scummvm_service::check_scummvm_status(&state.db, &state.app_data_dir).await.ok();
    serde_json::to_string_pretty(&serde_json::json!({
        "schema_version": 1,
        "collected_at": chrono::Utc::now().to_rfc3339(),
        "kitor_version": app.package_info().version.to_string(),
        "os": std::env::consts::OS, "architecture": std::env::consts::ARCH,
        "scummvm": runtime.map(|r| serde_json::json!({"downloaded": r.downloaded, "version": r.version})),
        "games": games, "recent_operations": diagnostics::history(&state.app_data_dir),
        "limitations": "Launch completion means the launch request succeeded, not that gameplay started. Raw process output and personal paths are excluded."
    })).map_err(|_| "Could not encode diagnostics".into())
}
