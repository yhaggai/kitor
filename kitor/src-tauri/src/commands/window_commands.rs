use tauri::{Manager, Window};

#[tauri::command]
pub async fn close_splashscreen(window: Window) -> Result<(), String> {
    if let Some(splash) = window.get_webview_window("splashscreen") {
        splash.close().map_err(|e| e.to_string())?;
    }
    
    if let Some(main) = window.get_webview_window("main") {
        main.show().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
