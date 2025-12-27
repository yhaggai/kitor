use reqwest;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use serde::{Serialize, Deserialize};
use tauri::Emitter;

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: u32,
}

pub async fn download_file(
    url: &str,
    dest_path: &Path,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download file: {}", e))?;

    let total_size = response.content_length().unwrap_or(0);

    let mut file = File::create(dest_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Failed to read chunk: {}", e))?;
        
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write to file: {}", e))?;

        downloaded += chunk.len() as u64;

        let percentage = if total_size > 0 {
            ((downloaded as f64 / total_size as f64) * 100.0) as u32
        } else {
            0
        };

        // Emit progress event
        let _ = app_handle.emit("download-progress", DownloadProgress {
            downloaded,
            total: total_size,
            percentage,
        });
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    Ok(())
}



