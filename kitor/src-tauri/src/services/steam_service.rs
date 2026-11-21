//! Steam OpenID identity and Web API access. Keys live in the OS credential store, never app files or logs.
use rand::TryRng;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{Mutex, Notify},
};

const OPENID: &str = "https://steamcommunity.com/openid/login";
const NS: &str = "http://specs.openid.net/auth/2.0";

#[derive(Default)]
pub struct SteamState {
    pub session: Mutex<Session>,
    pub cancel: Notify,
}
#[derive(Default)]
pub struct Session {
    pub steam_id: Option<String>,
    key: Option<String>,
    credentials_loaded: bool,
    games: Option<HashSet<u32>>,
    checked: Option<Instant>,
    message: Option<String>,
    pub generation: u64,
    pub connecting: bool,
}
#[derive(Serialize)]
pub struct Status {
    steam_id: Option<String>,
    has_api_key: bool,
    connecting: bool,
    owned_app_ids: Option<Vec<u32>>,
    message: Option<String>,
}
impl Status {
    pub fn native(library: Option<kitor_steam::OwnedLibrary>) -> Self {
        Self {
            steam_id: library.as_ref().map(|library| library.steam_id.clone()),
            owned_app_ids: library.map(|library| library.owned_app_ids),
            has_api_key: false,
            connecting: false,
            message: None,
        }
    }
}
impl Session {
    pub fn begin_login(&mut self) {
        self.generation += 1;
        self.connecting = true;
        self.games = None;
        self.checked = None;
        self.message = None;
    }

    pub fn status(&self) -> Status {
        Status {
            steam_id: self.steam_id.clone(),
            has_api_key: self.key.is_some(),
            connecting: self.connecting,
            owned_app_ids: self.games.as_ref().map(|g| g.iter().copied().collect()),
            message: self.message.clone(),
        }
    }
}
fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "Could not create Steam client".into())
}
fn valid_id(id: &str) -> bool {
    id.len() == 17 && id.bytes().all(|c| c.is_ascii_digit())
}

pub fn load_identity(path: &Path) -> Option<String> {
    let id = std::fs::read_to_string(path.join("steam-account.txt")).ok()?;
    valid_id(&id).then_some(id)
}
pub fn save_identity(path: &Path, id: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file =
        tempfile::NamedTempFile::new_in(path).map_err(|_| "Could not save Steam account")?;
    file.write_all(id.as_bytes())
        .map_err(|_| "Could not save Steam account")?;
    file.persist(path.join("steam-account.txt"))
        .map_err(|_| "Could not save Steam account")?;
    Ok(())
}
pub fn clear_identity(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path.join("steam-account.txt")) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("Could not remove Steam account".into()),
    }
}
pub fn validate_key(key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.len() != 32 || !key.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Enter a valid 32-character Steam Web API key".into());
    }
    Ok(())
}
pub fn set_key(session: &mut Session, key: String) -> Result<(), String> {
    validate_key(&key)?;
    session.credentials_loaded = true;
    session.key = Some(key.trim().into());
    session.games = None;
    session.checked = None;
    session.message = None;
    session.generation += 1;
    Ok(())
}

// All credential operations are serialized by the session mutex and run off the async executor.
fn credential_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new("click.kitor.steam", "web-api-key")
        .map_err(|_| "Could not access the system credential store".into())
}
pub async fn persist_key(key: String) -> Result<(), String> {
    validate_key(&key)?;
    tokio::task::spawn_blocking(move || {
        credential_entry()?.set_password(key.trim())
            .map_err(|_| "Could not save the Steam key in the system credential store. Unlock it and try again.".to_string())
    }).await.map_err(|_| "Credential store task failed")?
}
pub async fn delete_key() -> Result<(), String> {
    tokio::task::spawn_blocking(|| {
        match credential_entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Could not remove the saved Steam key. Unlock the system credential store and try again.".into()),
        }
    }).await.map_err(|_| "Credential store task failed")?
}
pub async fn restore_key(state: &SteamState) -> bool {
    let mut session = state.session.lock().await;
    if session.credentials_loaded {
        return false;
    }
    let result = tokio::task::spawn_blocking(|| match credential_entry()?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err(
            "Could not read the saved Steam key. Unlock the system credential store and try again."
                .to_string(),
        ),
    })
    .await
    .unwrap_or_else(|_| Err("Credential store task failed".into()));
    match result {
        Ok(Some(key)) => match set_key(&mut session, key) {
            Ok(()) => true,
            Err(error) => {
                session.message = Some(error);
                false
            }
        },
        Ok(None) => {
            session.credentials_loaded = true;
            false
        }
        Err(error) => {
            session.message = Some(error);
            false
        }
    }
}
pub fn clear_session(session: &mut Session) {
    let generation = session.generation + 1;
    *session = Session::default();
    session.generation = generation;
    session.credentials_loaded = true;
}

// Missing game_count means unavailable/private, never an empty owned library.
#[derive(Deserialize)]
struct OwnedResponse {
    response: OwnedLibrary,
}
#[derive(Deserialize)]
struct OwnedLibrary {
    game_count: Option<usize>,
    games: Option<Vec<OwnedGame>>,
}
#[derive(Deserialize)]
struct OwnedGame {
    appid: u32,
}
fn parse_owned(value: &str) -> Result<Option<HashSet<u32>>, String> {
    let parsed: OwnedResponse =
        serde_json::from_str(value).map_err(|_| "Steam returned an invalid library response")?;
    let Some(count) = parsed.response.game_count else {
        return Ok(None);
    };
    let games = parsed.response.games.unwrap_or_default();
    if games.len() != count {
        return Err("Steam returned an incomplete library response".into());
    }
    Ok(Some(games.into_iter().map(|g| g.appid).collect()))
}
pub async fn refresh(state: &SteamState) -> Status {
    let (id, key, generation) = {
        let session = state.session.lock().await;
        let (Some(id), Some(key)) = (&session.steam_id, &session.key) else {
            return session.status();
        };
        (id.clone(), key.clone(), session.generation)
    };
    let result = async {
        let response = client()?
            .get("https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/")
            .query(&[
                ("key", key.as_str()),
                ("steamid", id.as_str()),
                ("include_played_free_games", "1"),
            ])
            .send()
            .await
            .map_err(|_| "Could not reach Steam. Try refreshing again.")?;
        if response.status() == reqwest::StatusCode::FORBIDDEN
            || response.status() == reqwest::StatusCode::UNAUTHORIZED
        {
            return Err("Steam rejected the API key. Check your key and try again.".into());
        }
        if !response.status().is_success() {
            return Err("Steam is unavailable or rate limited. Try again later.".into());
        }
        parse_owned(
            &response
                .text()
                .await
                .map_err(|_| "Could not read Steam's response")?,
        )
    }
    .await;
    let mut session = state.session.lock().await;
    if session.generation == generation {
        session.checked = Some(Instant::now());
        match result {
            Ok(Some(games)) => {
                session.games = Some(games);
                session.message = None;
            }
            Ok(None) => {
                session.games = None;
                session.message =
                    Some("Ownership unknown. Your Steam game details may be private.".into());
            }
            Err(error) => {
                session.games = None;
                session.message = Some(error);
            }
        }
    }
    session.status()
}
pub async fn game_url(state: &SteamState, app_id: u32) -> String {
    let stale = state
        .session
        .lock()
        .await
        .checked
        .is_none_or(|t| t.elapsed() > Duration::from_secs(300));
    if stale {
        refresh(state).await;
    }
    let session = state.session.lock().await;
    let owned = session
        .games
        .as_ref()
        .is_some_and(|games| games.contains(&app_id));
    destination(app_id, owned)
}
fn destination(app_id: u32, owned: bool) -> String {
    if owned {
        format!("steam://nav/games/details/{app_id}")
    } else {
        format!("steam://store/{app_id}")
    }
}

fn assertion(params: &HashMap<String, String>, return_to: &str) -> Result<String, String> {
    let get = |key: &str| params.get(key).map(String::as_str).unwrap_or("");
    if get("openid.mode") == "cancel" {
        return Err("Steam sign-in was cancelled".into());
    }
    if get("openid.ns") != NS
        || get("openid.mode") != "id_res"
        || get("openid.op_endpoint") != OPENID
        || get("openid.return_to") != return_to
    {
        return Err("Invalid Steam sign-in response".into());
    }
    let claimed = get("openid.claimed_id");
    let id = claimed
        .strip_prefix("https://steamcommunity.com/openid/id/")
        .unwrap_or("");
    if !valid_id(id) || get("openid.identity") != claimed {
        return Err("Invalid Steam identity".into());
    }
    let signed: HashSet<_> = get("openid.signed").split(',').collect();
    for required in [
        "op_endpoint",
        "claimed_id",
        "identity",
        "return_to",
        "response_nonce",
        "assoc_handle",
    ] {
        if !signed.contains(required) {
            return Err("Incomplete Steam signature".into());
        }
    }
    let nonce = get("openid.response_nonce");
    let date = nonce
        .get(..20)
        .and_then(|v| time::OffsetDateTime::parse(v, &time::format_description::well_known::Rfc3339).ok())
        .ok_or("Invalid Steam response nonce")?;
    let age = (time::OffsetDateTime::now_utc() - date).whole_seconds();
    if !(-60..=300).contains(&age) {
        return Err("Steam sign-in expired. Please try again.".into());
    }
    Ok(id.into())
}
async fn verify(params: HashMap<String, String>, return_to: &str) -> Result<String, String> {
    let id = assertion(&params, return_to)?;
    let mut form: HashMap<_, _> = params
        .into_iter()
        .filter(|(k, _)| k.starts_with("openid."))
        .collect();
    form.insert("openid.mode".into(), "check_authentication".into());
    let response = client()?
        .post(OPENID)
        .form(&form)
        .send()
        .await
        .map_err(|_| "Could not verify Steam sign-in")?;
    if !response.status().is_success() {
        return Err("Steam could not verify sign-in".into());
    }
    let body = response
        .text()
        .await
        .map_err(|_| "Could not read Steam verification")?;
    if !body.lines().any(|line| line.trim_end() == "is_valid:true") {
        return Err("Steam rejected the sign-in signature".into());
    }
    Ok(id)
}

pub async fn login() -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|_| "Could not start Steam sign-in callback")?;
    let port = listener
        .local_addr()
        .map_err(|_| "Could not read callback port")?
        .port();
    let mut random = [0u8; 32];
    rand::rngs::SysRng
        .try_fill_bytes(&mut random)
        .map_err(|_| "Could not generate Steam sign-in token")?;
    let token: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let return_to = format!("http://127.0.0.1:{port}/steam-callback/{token}");
    let mut auth = url::Url::parse(OPENID).unwrap();
    auth.query_pairs_mut().extend_pairs([
        ("openid.ns", NS),
        ("openid.mode", "checkid_setup"),
        ("openid.return_to", return_to.as_str()),
        ("openid.realm", format!("http://127.0.0.1:{port}/").as_str()),
        (
            "openid.identity",
            "http://specs.openid.net/auth/2.0/identifier_select",
        ),
        (
            "openid.claimed_id",
            "http://specs.openid.net/auth/2.0/identifier_select",
        ),
    ]);
    tauri_plugin_opener::open_url(auth.as_str(), None::<&str>)
        .map_err(|_| "Could not open Steam sign-in in your browser")?;
    tokio::time::timeout(Duration::from_secs(300), async {
        loop {
            let (mut stream, _) = listener.accept().await.map_err(|_| "Steam callback failed")?;
            let read = tokio::time::timeout(Duration::from_secs(5), async {
                let mut bytes = Vec::new();
                loop {
                    let mut chunk = [0u8; 2048];
                    let size = stream.read(&mut chunk).await.ok()?;
                    if size == 0 { return None; }
                    bytes.extend_from_slice(&chunk[..size]);
                    if bytes.len() > 16384 { return None; }
                    if bytes.windows(4).any(|v| v == b"\r\n\r\n") { return String::from_utf8(bytes).ok(); }
                }
            }).await.ok().flatten();
            let Some(request) = read else { continue; };
            let parts: Vec<_> = request.lines().next().unwrap_or("").split_whitespace().collect();
            let target = parts.get(1).copied().unwrap_or("");
            let parsed = url::Url::parse(&format!("http://127.0.0.1:{port}{target}"));
            let Ok(parsed) = parsed else { continue; };
            if parts.first() != Some(&"GET") || parsed.path() != format!("/steam-callback/{token}") { continue; }
            let mut params = HashMap::new();
            let mut duplicate = false;
            for (k, v) in parsed.query_pairs() {
                if params.insert(k.into_owned(), v.into_owned()).is_some() { duplicate = true; }
            }
            let result = if duplicate { Err("Invalid duplicate sign-in fields".into()) } else { verify(params, &return_to).await };
            let body = if result.is_ok() { "Steam connected. You can close this tab and return to Kitor." }
                else { "Steam sign-in failed. Return to Kitor to try again." };
            let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = tokio::time::timeout(Duration::from_secs(2), stream.write_all(response.as_bytes())).await;
            return result;
        }
    }).await.map_err(|_| "Steam sign-in timed out. Please try again.".to_string())?
}

#[cfg(test)]
mod tests;
