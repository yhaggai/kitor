use super::REPO;
use reqwest::{Client, Method};
use serde_json::Value;
use std::time::Duration;

pub fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Kitor-translation-contributions")
        .timeout(Duration::from_secs(40))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize GitHub connection".into())
}
pub fn encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, percent_encoding::NON_ALPHANUMERIC).to_string()
}
pub fn string(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("GitHub response omitted {field}"))
}
pub async fn bounded(mut response: reqwest::Response) -> Result<String, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "GitHub returned HTTP {}. Check sign-in, permissions, or rate limits and retry.",
            status.as_u16()
        ));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "GitHub response was interrupted")?
    {
        if bytes.len() + chunk.len() > 8_000_000 {
            return Err("GitHub response exceeds size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| "GitHub response was not UTF-8".into())
}
pub async fn api(path: &str, body: Option<Value>, token: Option<&str>) -> Result<Value, String> {
    let mut request = client()?
        .request(
            if body.is_some() {
                Method::POST
            } else {
                Method::GET
            },
            format!("https://api.github.com/{path}"),
        )
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    if let Some(body) = body {
        request = request.json(&body);
    }
    let text = bounded(request.send().await.map_err(|_| "Could not reach GitHub")?).await?;
    serde_json::from_str(&text).map_err(|_| "Invalid GitHub response".into())
}
pub async fn raw(path: &str, sha: &str, token: Option<&str>) -> Result<String, String> {
    let mut request = client()?
        .get(format!(
            "https://api.github.com/repos/{REPO}/contents/{path}?ref={}",
            encode(sha)
        ))
        .header("Accept", "application/vnd.github.raw+json");
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    bounded(
        request
            .send()
            .await
            .map_err(|_| "Could not load translation source from GitHub")?,
    )
    .await
}
