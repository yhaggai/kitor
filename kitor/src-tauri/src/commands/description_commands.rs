use std::time::Duration;

const DESCRIPTION_FEED_URL: &str = "https://assets.kitor.click/catalog/game-descriptions-v1.json";
const MAX_FEED_BYTES: usize = 512 * 1024;

#[tauri::command]
pub async fn fetch_game_descriptions() -> Result<String, String> {
    let mut response = reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| error.to_string())?
        .get(DESCRIPTION_FEED_URL)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| error.to_string())?;

    if response.status() != reqwest::StatusCode::OK {
        return Err("Unexpected description feed response".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
        if bytes.len() + chunk.len() > MAX_FEED_BYTES {
            return Err("Description feed is too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }

    String::from_utf8(bytes).map_err(|error| error.to_string())
}
