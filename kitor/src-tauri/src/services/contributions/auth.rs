use super::{github, Contributions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Device {
    code: String,
    client_id: String,
    interval: u64,
    next: Instant,
    expires: Instant,
}
#[derive(Serialize)]
pub struct Status {
    pub connected: bool,
    pub configured: bool,
}
#[derive(Serialize)]
pub struct Login {
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u64,
}
#[derive(Serialize)]
pub struct Poll {
    pub connected: bool,
    pub interval: u64,
}
#[derive(Serialize, Deserialize)]
struct Credential {
    token: String,
    expires_at: Option<u64>,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new("click.kitor.github", "translation-contribution-oauth")
        .map_err(|_| "Could not open the system credential store".into())
}
fn client_id() -> Option<String> {
    std::env::var("KITOR_GITHUB_CLIENT_ID")
        .ok()
        .or_else(|| option_env!("KITOR_GITHUB_CLIENT_ID").map(str::to_owned))
        .filter(|v| {
            !v.is_empty()
                && v.len() < 150
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
        })
}
pub async fn token() -> Result<Option<String>, String> {
    tokio::task::spawn_blocking(|| match entry()?.get_password() {
        Ok(value) => {
            let credential: Credential = serde_json::from_str(&value)
                .map_err(|_| "Reconnect GitHub to replace an invalid credential")?;
            if credential.expires_at.is_some_and(|expiry| now() >= expiry) {
                Ok(None)
            } else {
                Ok(Some(credential.token))
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Unlock the system credential store and retry".into()),
    })
    .await
    .map_err(|_| "Credential task failed")?
}
pub async fn status() -> Result<Status, String> {
    Ok(Status {
        connected: token().await?.is_some(),
        configured: client_id().is_some(),
    })
}
pub async fn disconnect(state: &Contributions) -> Result<(), String> {
    *state.device.lock().await = None;
    tokio::task::spawn_blocking(|| match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not remove GitHub credential".into()),
    })
    .await
    .map_err(|_| "Credential task failed")?
}
pub async fn begin(state: &Contributions) -> Result<Login, String> {
    let client_id = client_id().ok_or("GitHub sign-in needs the Kitor OAuth app client ID. Set KITOR_GITHUB_CLIENT_ID when building or running the app.")?;
    let response = github::client()?
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", client_id.as_str()), ("scope", "public_repo")])
        .send()
        .await
        .map_err(|_| "Could not start GitHub sign-in")?;
    let value: Value = serde_json::from_str(&github::bounded(response).await?)
        .map_err(|_| "Invalid sign-in response")?;
    if value.get("error").is_some() {
        return Err("GitHub device sign-in is unavailable. Check that Device Flow is enabled on the OAuth app.".into());
    }
    let interval = value["interval"].as_u64().unwrap_or(5).max(5);
    let code = github::string(&value, "device_code")?;
    *state.device.lock().await = Some(Device {
        code,
        client_id,
        interval,
        next: Instant::now() + Duration::from_secs(interval),
        expires: Instant::now() + Duration::from_secs(value["expires_in"].as_u64().unwrap_or(900)),
    });
    Ok(Login {
        user_code: github::string(&value, "user_code")?,
        verification_uri: "https://github.com/login/device".into(),
        interval,
    })
}
pub async fn poll(state: &Contributions) -> Result<Poll, String> {
    let mut guard = state.device.lock().await;
    let device = guard.as_mut().ok_or("Start GitHub sign-in again")?;
    if Instant::now() >= device.expires {
        *guard = None;
        return Err("Sign-in expired. Please start again.".into());
    }
    if Instant::now() < device.next {
        return Ok(Poll {
            connected: false,
            interval: device.interval,
        });
    }
    let response = github::client()?
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", device.client_id.as_str()),
            ("device_code", device.code.as_str()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|_| "Could not finish GitHub sign-in")?;
    let value: Value = serde_json::from_str(&github::bounded(response).await?)
        .map_err(|_| "Invalid sign-in response")?;
    if let Some(token) = value["access_token"].as_str() {
        let credential = Credential {
            token: token.into(),
            expires_at: value["expires_in"]
                .as_u64()
                .map(|seconds| now() + seconds.saturating_sub(60)),
        };
        tokio::task::spawn_blocking(move || {
            entry()?
                .set_password(&serde_json::to_string(&credential).unwrap())
                .map_err(|_| {
                    "Could not save GitHub sign-in in the system credential store".to_string()
                })
        })
        .await
        .map_err(|_| "Credential task failed")??;
        *guard = None;
        return Ok(Poll {
            connected: true,
            interval: 0,
        });
    }
    match value["error"].as_str() {
        Some("authorization_pending") => {}
        Some("slow_down") => device.interval += 5,
        _ => {
            *guard = None;
            return Err("GitHub sign-in was declined or expired. Please retry.".into());
        }
    }
    device.next = Instant::now() + Duration::from_secs(device.interval);
    Ok(Poll {
        connected: false,
        interval: device.interval,
    })
}
