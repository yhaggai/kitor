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

/// GOG authentication never enters a URL, error string, or CDN request.
pub async fn download_gog_installer(
    initial: url::Url, token: &str, destination: &Path, app: &tauri::AppHandle,
) -> Result<(), String> {
    let initial = super::gog_download_security::installer_url(initial.as_str())?;
    let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none())
        .referer(false).build().map_err(|_| "Could not create GOG download client")?;
    let mut url = initial.clone();
    for hop in 0..=5 {
        let mut request = client.get(url.clone());
        // Only the initial catalog endpoint receives the account credential.
        // Redirected URLs carry their own CDN authorization, if required.
        if hop == 0 { request = request.bearer_auth(token); }
        let response = request.send().await.map_err(|_| "GOG download request failed")?;
        if response.status().is_redirection() {
            let location = response.headers().get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok()).ok_or("Invalid GOG download redirect")?;
            let next = url.join(location).map_err(|_| "Invalid GOG download redirect")?;
            if !super::gog_download_security::allowed_redirect(&next) {
                return Err("GOG redirected to an untrusted download location".into());
            }
            url = next;
            continue;
        }
        response.error_for_status_ref().map_err(|_| "GOG installer download was rejected")?;
        let total = response.content_length().unwrap_or(0);
        let mut file = File::create(destination).await.map_err(|_| "Could not create installer download")?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| "Incomplete GOG installer download")?;
            file.write_all(&chunk).await.map_err(|_| "Could not write installer download")?;
            downloaded += chunk.len() as u64;
            let percentage = if total == 0 { 0 } else { ((downloaded as f64 / total as f64) * 100.0).min(100.0) as u32 };
            let _ = app.emit("download-progress", DownloadProgress { downloaded, total, percentage });
        }
        file.flush().await.map_err(|_| "Could not flush installer download")?;
        if downloaded == 0 { return Err("GOG returned an empty installer".into()); }
        return Ok(());
    }
    Err("Too many GOG download redirects".into())
}

pub async fn download_file(
    url: &str,
    dest_path: &Path,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    download_with_progress(url, dest_path, |progress| {
        let _ = app_handle.emit("download-progress", progress);
    }).await
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
        std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()));
    let result = async {
        let response = match super::github_download_service::download(url).await? {
            Some(response) => response,
            None => reqwest::Client::new().get(url).send().await
                .map_err(|_| "Could not download file. Check your connection.".to_string())?,
        };
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
            while let Some(item) = stream.next().await {
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
        tokio::fs::rename(&temp_path, dest_path).await
            .map_err(|e| format!("Failed to finish download: {}", e))?;
        progress(DownloadProgress { downloaded: size, total: size, percentage: 100 });
        Ok(())
    }.await;
    if result.is_err() { let _ = tokio::fs::remove_file(&temp_path).await; }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failed_http_and_invalid_zip_preserve_existing_download() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        for status in ["404 Not Found", "200 OK"] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/patch.zip", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = [0; 4096];
                socket.read(&mut request).await.unwrap();
                socket.write_all(format!("HTTP/1.1 {}\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found", status).as_bytes()).await.unwrap();
            });
            let dir = std::env::temp_dir().join(format!("kitor-http-test-{}-{}",
                std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("patch.zip");
            std::fs::write(&path, b"previous download").unwrap();
            let error = download_with_progress(&url, &path, |_| {}).await.unwrap_err();
            assert!(error.contains(if status.starts_with("404") { "HTTP 404" } else { "not a valid ZIP" }), "{}", error);
            assert_eq!(std::fs::read(&path).unwrap(), b"previous download");
            assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
            server.await.unwrap();
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn reports_transfer_progress_with_known_and_unknown_sizes() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        for known_size in [true, false] {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/patch.bin", listener.local_addr().unwrap());
            let server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = [0; 4096];
                socket.read(&mut request).await.unwrap();
                let length = if known_size { "Content-Length: 6\r\n" } else { "" };
                socket.write_all(format!("HTTP/1.1 200 OK\r\n{length}Connection: close\r\n\r\nabc").as_bytes()).await.unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                socket.write_all(b"def").await.unwrap();
            });
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("patch.bin");
            let reports = std::sync::Mutex::new(Vec::new());
            download_with_progress(&url, &path, |p| reports.lock().unwrap().push(p)).await.unwrap();
            server.await.unwrap();
            let reports = reports.into_inner().unwrap();
            assert!(reports.len() >= 3);
            assert_eq!(reports[0].downloaded, 0);
            assert_eq!(reports[0].total, if known_size { 6 } else { 0 });
            assert!(reports[1].downloaded > 0);
            let last = reports.last().unwrap();
            assert_eq!((last.downloaded, last.total, last.percentage), (6, 6, 100));
            assert_eq!(std::fs::read(path).unwrap(), b"abcdef");
        }
    }

    #[tokio::test]
    #[ignore = "requires network and a saved GitHub token for the private patch repository"]
    async fn downloads_real_kyrandia_release() {
        let dir = std::env::temp_dir().join(format!("kitor-release-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("patch.zip");
        // Reproduce and replace the old nine-byte failed download.
        std::fs::write(&path, b"Not Found").unwrap();
        download_with_progress(
            "https://github.com/yhaggai/kitor/releases/download/patch-kyra1-cd-he-v1.0-beta/KyraHebPatch-BETA.zip",
            &path, |_| {}).await.unwrap();
        crate::services::file_service::extract_zip(&path, &dir.join("extracted")).unwrap();
        assert!(dir.join("extracted/GEMCUT.PAK").is_file());
        assert!(dir.join("extracted/8FAT.FNT").is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
