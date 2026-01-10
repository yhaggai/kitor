use reqwest::header::CONTENT_TYPE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn covers_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("covers")
}

pub fn ensure_covers_dir(app_data_dir: &Path) -> Result<PathBuf, String> {
    let dir = covers_dir(app_data_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create covers dir: {}", e))?;
    Ok(dir)
}

pub fn get_cached_cover_path(scummvm_id: &str, app_data_dir: &Path) -> Result<Option<PathBuf>, String> {
    let dir = covers_dir(app_data_dir);
    if !dir.exists() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(&dir).map_err(|e| format!("Failed to read covers dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read covers dir entry: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let file_stem = path.file_stem().and_then(OsStr::to_str);
        if file_stem == Some(scummvm_id) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn extension_from_url(url: &str) -> Option<String> {
    let url = url.split('?').next().unwrap_or(url);
    let last = url.rsplit('/').next()?;
    let ext = last.rsplit('.').next()?;
    let ext = ext.trim().to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(ext),
        _ => None,
    }
}

fn extension_from_content_type(content_type: Option<&str>) -> Option<String> {
    let content_type = content_type?.to_ascii_lowercase();
    match content_type.split(';').next().unwrap_or("").trim() {
        "image/jpeg" => Some("jpg".to_string()),
        "image/png" => Some("png".to_string()),
        "image/webp" => Some("webp".to_string()),
        "image/gif" => Some("gif".to_string()),
        _ => None,
    }
}

pub async fn download_and_cache_cover(
    scummvm_id: &str,
    cover_url: &str,
    app_data_dir: PathBuf,
) -> Result<PathBuf, String> {
    let dir = ensure_covers_dir(&app_data_dir)?;

    if let Some(existing) = get_cached_cover_path(scummvm_id, &app_data_dir)? {
        return Ok(existing);
    }

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(cover_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download cover image: {}", e))?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let ext = extension_from_url(cover_url)
        .or_else(|| extension_from_content_type(content_type))
        .unwrap_or_else(|| "jpg".to_string());

    let dest_path = dir.join(format!("{}.{}", scummvm_id, ext));

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read cover bytes: {}", e))?;

    tokio::fs::write(&dest_path, bytes)
        .await
        .map_err(|e| format!("Failed to write cover image: {}", e))?;

    Ok(dest_path)
}

pub fn clear_cover_cache(app_data_dir: &Path) -> Result<(), String> {
    let dir = covers_dir(app_data_dir);
    if !dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(&dir).map_err(|e| format!("Failed to read covers dir: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read covers dir entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete cover file {}: {}", path.display(), e))?;
        }
    }

    Ok(())
}



