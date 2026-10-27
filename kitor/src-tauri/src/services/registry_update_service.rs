use super::registry_service::updates;
use std::io::Write;
static REFRESH: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn refresh() -> Result<(), String> {
    let _lock = REFRESH.lock().await;
    let mut response = super::github_download_service::registry().await?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > 2_000_000 {
            return Err("Patch registry is too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let catalog = updates::parse(&bytes)?;
    let path = updates::cache_path()?;
    if let Ok(previous) = std::fs::read(path) {
        if let Ok(previous) = updates::parse(&previous) {
            updates::validate_advance(&previous, &catalog)?;
        }
    }
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing cache directory")?)
        .map_err(|e| e.to_string())?;
    temp.write_all(&bytes).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}
