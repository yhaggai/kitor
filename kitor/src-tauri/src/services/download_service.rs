use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Emitter;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: u32,
}

pub async fn download_patch_file(
    url: &str,
    dest_path: &Path,
    app_handle: &tauri::AppHandle,
    scummvm_id: &str,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    download_with_progress(url, dest_path, |progress| {
        let bytes_per_second = progress.downloaded as f64 / started.elapsed().as_secs_f64().max(0.001);
        let seconds_remaining = if progress.total > 0 && bytes_per_second > 0.0 {
            Some(progress.total.saturating_sub(progress.downloaded) as f64 / bytes_per_second)
        } else { None };
        let _ = app_handle.emit("patch-download-progress", serde_json::json!({
            "scummvm_id": scummvm_id,
            "downloaded": progress.downloaded,
            "total": progress.total,
            "percentage": progress.percentage.min(100),
            "bytes_per_second": bytes_per_second,
            "seconds_remaining": seconds_remaining,
        }));
        let _ = app_handle.emit("download-progress", progress);
    }).await
}

async fn download_with_progress(
    url: &str,
    dest_path: &Path,
    progress: impl Fn(DownloadProgress),
) -> Result<(), String> {
    // Never expose partial downloads or HTTP error pages as completed assets.
    let temp_path = dest_path.with_extension(format!("download-{}-{}",
        std::process::id(), time::OffsetDateTime::now_utc().unix_timestamp_nanos()));
    let result = async {
        let response = super::install_cancellation::interruptible(async {
            reqwest::Client::new().get(url).send().await
                .map_err(|_| "Could not download file. Check your connection.".to_string())
        }).await?;
        let status = response.status();
        {
            response.error_for_status_ref()
                .map_err(|_| format!("Download failed: HTTP {}. Check the download URL and access permissions.", status))?;
            let total = response.content_length().unwrap_or(0);
            let mut file = File::create(&temp_path).await
                .map_err(|e| format!("Failed to create download: {}", e))?;
            let mut downloaded = 0;
            let mut stream = response.bytes_stream();
            let mut last_report = std::time::Instant::now();
            progress(DownloadProgress { downloaded: 0, total, percentage: 0 });
            while let Some(item) =
                super::install_cancellation::interruptible(async { Ok(stream.next().await) }).await?
            {
                let chunk = item.map_err(|e| format!("Incomplete download: {}", e))?;
                file.write_all(&chunk).await.map_err(|e| format!("Failed to write download: {}", e))?;
                downloaded += chunk.len() as u64;
                if last_report.elapsed() >= std::time::Duration::from_millis(250) {
                progress(DownloadProgress {
                    downloaded, total,
                    percentage: if total > 0 { ((downloaded as f64 / total as f64) * 100.0) as u32 } else { 0 },
                });
                last_report = std::time::Instant::now();
                }
            }
            file.flush().await.map_err(|e| format!("Failed to flush download: {}", e))?;
        }
        let size = tokio::fs::metadata(&temp_path).await.map_err(|e| e.to_string())?.len();
        if size == 0 { return Err("Downloaded file is empty".to_string()); }
        // Check the entire archive (including CRCs) before publishing the download.
        if dest_path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("zip")) {
            crate::services::file_service::validate_zip(&temp_path)?;
        }
        super::install_cancellation::checkpoint()?;
        tokio::fs::rename(&temp_path, dest_path).await
            .map_err(|e| format!("Failed to finish download: {}", e))?;
        progress(DownloadProgress { downloaded: size, total: size, percentage: 100 });
        Ok(())
    }.await;
    if result.is_err() { let _ = tokio::fs::remove_file(&temp_path).await; }
    result
}

#[cfg(test)]
mod tests;
