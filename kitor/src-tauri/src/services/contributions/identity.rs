use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub id: String,
    pub name: String,
}
#[derive(Serialize, Deserialize)]
struct Session {
    token: String,
    contributor: Contributor,
}
struct Pending {
    id: String,
    verifier: String,
}
#[derive(Default)]
pub struct Identity {
    pending: Mutex<Option<Pending>>,
}
pub fn endpoint() -> Result<String, String> {
    let value = option_env!("KITOR_CONTRIBUTION_API_URL")
        .unwrap_or("http://127.0.0.1:8787")
        .trim_end_matches('/');
    let url = url::Url::parse(value).map_err(|_| "Invalid contribution API URL")?;
    if url.scheme() != "https"
        && !(cfg!(debug_assertions)
            && url.scheme() == "http"
            && url.host_str() == Some("127.0.0.1"))
    {
        return Err("Contribution API requires HTTPS".into());
    }
    Ok(value.into())
}
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new("kitor-contribution-worker", &endpoint()?)
        .map_err(|_| "Could not access secure credential storage".into())
}
fn stored() -> Result<Option<Session>, String> {
    match entry()?.get_password() {
        Ok(s) => serde_json::from_str(&s)
            .map(Some)
            .map_err(|_| "Invalid saved session".into()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not read secure credential storage".into()),
    }
}
pub fn contributor() -> Result<Contributor, String> {
    stored()?
        .map(|s| s.contributor)
        .ok_or("Sign in with Google to contribute".into())
}
pub fn token_for(expected: &str) -> Result<String, String> {
    let session = stored()?.ok_or("Sign in with Google to contribute")?;
    if session.contributor.id != expected {
        return Err("Account changed. Reopen your draft.".into());
    }
    Ok(session.token)
}
pub async fn request(
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not initialize connection")?;
    let mut r = client.request(
        if body.is_some() {
            reqwest::Method::POST
        } else {
            reqwest::Method::GET
        },
        format!("{}{path}", endpoint()?),
    );
    if let Some(t) = token {
        r = r.bearer_auth(t);
    }
    if let Some(b) = body {
        r = r.json(&b);
    }
    let mut response = r
        .send()
        .await
        .map_err(|_| "Contribution service unavailable. Your draft is saved")?;
    let status = response.status();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "Response interrupted")? {
        if bytes.len() + chunk.len() > 65536 {
            return Err("Response too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "Invalid service response")?;
    if !status.is_success() {
        return Err(value["error"]
            .as_str()
            .unwrap_or("Contribution request failed")
            .into());
    }
    Ok(value)
}
#[tauri::command]
pub async fn contributor_status() -> Result<Option<Contributor>, String> {
    Ok(stored()?.map(|session| session.contributor))
}
#[tauri::command]
pub async fn contributor_sign_in(state: tauri::State<'_, Identity>) -> Result<String, String> {
    let mut pending = state.pending.lock().await;
    if let Some(p) = pending.take() {
        let _ = request(
            "/auth/cancel",
            Some(json!({"id":p.id,"verifier":p.verifier})),
            None,
        )
        .await;
    }
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let v = request("/auth/start", Some(json!({"challenge":challenge})), None).await?;
    let id = v["id"].as_str().ok_or("Missing sign-in ID")?.to_string();
    let link = v["authorization_url"]
        .as_str()
        .ok_or("Missing Google URL")?
        .to_string();
    let u = url::Url::parse(&link).map_err(|_| "Invalid Google URL")?;
    if u.scheme() != "https"
        || u.host_str() != Some("accounts.google.com")
        || u.path() != "/o/oauth2/v2/auth"
    {
        return Err("Unexpected sign-in URL".into());
    }
    *pending = Some(Pending { id, verifier });
    Ok(link)
}
#[tauri::command]
pub async fn contributor_poll(
    state: tauri::State<'_, Identity>,
) -> Result<Option<Contributor>, String> {
    let mut pending = state.pending.lock().await;
    let p = pending.as_ref().ok_or("Sign-in cancelled")?;
    let v = request(
        "/auth/poll",
        Some(json!({"id":p.id,"verifier":p.verifier})),
        None,
    )
    .await?;
    if v["pending"] == true {
        return Ok(None);
    }
    let session: Session = serde_json::from_value(v).map_err(|_| "Invalid sign-in response")?;
    if session.token.len() != 64
        || !session.token.bytes().all(|b| b.is_ascii_hexdigit())
        || !session
            .contributor
            .id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() || b == b'-')
        || session.contributor.id.len() != 36
    {
        return Err("Invalid session".into());
    }
    entry()?
        .set_password(&serde_json::to_string(&session).map_err(|_| "Could not save session")?)
        .map_err(|_| "Could not save session securely")?;
    *pending = None;
    Ok(Some(session.contributor))
}
#[tauri::command]
pub async fn contributor_cancel(state: tauri::State<'_, Identity>) -> Result<(), String> {
    let mut pending = state.pending.lock().await;
    if let Some(p) = pending.take() {
        let _ = request(
            "/auth/cancel",
            Some(json!({"id":p.id,"verifier":p.verifier})),
            None,
        )
        .await;
    }
    Ok(())
}
#[tauri::command]
pub async fn contributor_sign_out(state: tauri::State<'_, Identity>) -> Result<(), String> {
    contributor_cancel(state).await?;
    if let Some(s) = stored()? {
        request("/auth/logout", Some(json!({})), Some(&s.token)).await?;
        entry()?
            .delete_credential()
            .map_err(|_| "Could not clear saved session")?;
    }
    Ok(())
}
