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

pub async fn download_file(
    url: &str,
    dest_path: &Path,
    app_handle: &tauri::AppHandle,
) -> Result<(), String> {
    download_with_progress(url, dest_path, |progress| {
        let _ = app_handle.emit("download-progress", progress);
    }).await
}

// Only release asset URLs on GitHub may use the user's GitHub CLI login.
fn github_release_asset(url: &str) -> Option<(String, String, String)> {
    let url = url::Url::parse(url).ok()?;
    if url.scheme() != "https" || url.host_str()? != "github.com"
        || !url.username().is_empty() || url.password().is_some()
        || url.port().is_some() || url.query().is_some() || url.fragment().is_some() {
        return None;
    }
    let parts: Vec<_> = url.path_segments()?.collect();
    if parts.len() != 6 || parts[2] != "releases" || parts[3] != "download"
        || parts.iter().any(|part| part.is_empty() || part.starts_with('-') || part.contains('%')) {
        return None;
    }
    // gh treats the asset name as a glob; require a literal name.
    if parts[5].contains(['*', '?', '[', ']']) {
        return None;
    }
    Some((format!("{}/{}", parts[0], parts[1]), parts[4].into(), parts[5].into()))
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
        let response = reqwest::Client::new().get(url).send().await
            .map_err(|e| format!("Failed to download file: {}", e))?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            if let Some((repo, tag, asset)) = github_release_asset(url) {
                let output = tokio::process::Command::new("gh")
                    .args(["release", "download", &tag, "--repo", &format!("github.com/{}", repo),
                        "--pattern", &asset, "--output"])
                    .arg(&temp_path)
                    .env("GH_PROMPT_DISABLED", "1")
                    .output().await
                    .map_err(|_| "Patch download returned HTTP 404. For a private GitHub release, install GitHub CLI and sign in with `gh auth login`; otherwise check the patch URL.".to_string())?;
                if !output.status.success() {
                    return Err("Could not download the GitHub release asset. Check that the release exists and `gh auth status` shows an account with access to the repository.".to_string());
                }
            } else {
                return Err(format!("Download failed: HTTP {}. Check the download URL and access permissions.", status));
            }
        } else {
            response.error_for_status_ref()
                .map_err(|_| format!("Download failed: HTTP {}. Check the download URL and access permissions.", status))?;
            let total = response.content_length().unwrap_or(0);
            let mut file = File::create(&temp_path).await
                .map_err(|e| format!("Failed to create download: {}", e))?;
            let mut downloaded = 0;
            let mut stream = response.bytes_stream();
            while let Some(item) = stream.next().await {
                let chunk = item.map_err(|e| format!("Incomplete download: {}", e))?;
                file.write_all(&chunk).await.map_err(|e| format!("Failed to write download: {}", e))?;
                downloaded += chunk.len() as u64;
                progress(DownloadProgress {
                    downloaded, total,
                    percentage: if total > 0 { ((downloaded as f64 / total as f64) * 100.0) as u32 } else { 0 },
                });
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

    #[test]
    fn only_literal_github_release_assets_can_use_authentication() {
        assert_eq!(github_release_asset("https://github.com/owner/repo/releases/download/v1/patch.zip"),
            Some(("owner/repo".into(), "v1".into(), "patch.zip".into())));
        for url in ["https://example.com/owner/repo/releases/download/v1/patch.zip",
            "http://github.com/owner/repo/releases/download/v1/patch.zip",
            "https://github.com/owner/repo/releases/download/v1/*.zip",
            "https://github.com/owner/repo/releases/download/--help/patch.zip",
            "https://github.com/owner/repo/blob/main/patch.zip"] {
            assert!(github_release_asset(url).is_none(), "{}", url);
        }
    }

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
    #[ignore = "requires network and gh authentication for the private patch repository"]
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
