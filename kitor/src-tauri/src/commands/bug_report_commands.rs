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
