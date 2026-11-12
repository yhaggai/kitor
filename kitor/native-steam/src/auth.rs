use crate::{
    wire::{Fields, Message},
    Event,
};
use anyhow::{ensure, Context, Result};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    Engine,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub account: String,
    pub steam_id: u64,
    pub refresh_token: String,
}
impl Credentials {
    pub fn load() -> Result<Option<Self>> {
        match entry()?.get_password() {
            Ok(s) => Ok(Some(serde_json::from_str(&s)?)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => anyhow::bail!("Unlock the system credential store to connect Steam"),
        }
    }
    pub fn save(&self) -> Result<()> {
        entry()?
            .set_password(&serde_json::to_string(self)?)
            .map_err(|_| {
                anyhow::anyhow!("Could not save Steam session in the system credential store")
            })
    }
    pub fn delete() -> Result<()> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => anyhow::bail!("Could not remove saved Steam session"),
        }
    }
}
fn entry() -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(
        "click.kitor.steam",
        "native-content-session",
    )?)
}
async fn call(client: &reqwest::Client, method: &str, msg: Message) -> Result<Fields> {
    let url = format!("https://api.steampowered.com/IAuthenticationService/{method}/v1/");
    let arguments = [
        ("input_protobuf_encoded", STANDARD.encode(msg.0)),
        ("format", "protobuf_raw".to_owned()),
    ];
    let request = if method == "GetPasswordRSAPublicKey" {
        client.get(url).query(&arguments)
    } else {
        client.post(url).form(&arguments)
    };
    let r = request
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("Could not contact Steam authentication"))?
        .error_for_status()
        .map_err(|_| {
            anyhow::anyhow!("Steam authentication service is unavailable. Please try again.")
        })?;
    if let Some(code) = r.headers().get("x-eresult") {
        match code.to_str().unwrap_or("") {
            "1" => {}
            "5" => anyhow::bail!("Incorrect Steam username or password."),
            "65" | "88" => anyhow::bail!("Incorrect Steam Guard code. Please try again."),
            "84" => anyhow::bail!("Too many Steam sign-in attempts. Please wait and try again."),
            _ => anyhow::bail!("Steam authentication request was rejected"),
        }
    }
    Fields::parse(&r.bytes().await?)
}
pub async fn qr(
    client: &reqwest::Client,
    emit: &(impl Fn(Event) + Send + Sync),
) -> Result<Credentials> {
    let device = Message::default()
        .bytes(1, "Kitor")
        .number(2, 1)
        .number(3, 20);
    let mut session = call(
        client,
        "BeginAuthSessionViaQR",
        Message::default()
            .bytes(1, "Kitor")
            .number(2, 1)
            .bytes(3, device.0)
            .bytes(4, "Client"),
    )
    .await?;
    let request_id = session.b(3).to_vec();
    let mut id = session.n(1);
    let interval = f32::from_bits(session.n(4) as u32);
    let interval = if interval.is_finite() {
        interval.clamp(1.0, 10.0)
    } else {
        3.0
    };
    ensure!(
        id != 0 && !request_id.is_empty(),
        "Steam did not create a login session"
    );
    let deadline = tokio::time::Instant::now() + Duration::from_secs(180);
    loop {
        let url = session.s(2)?;
        if !url.is_empty() {
            ensure!(
                url.starts_with("https://s.team/q/"),
                "Unexpected Steam QR address"
            );
            let code = qrcode::QrCode::new(url.as_bytes())?;
            emit(Event::Qr {
                url,
                svg: code
                    .render::<qrcode::render::svg::Color>()
                    .min_dimensions(240, 240)
                    .build(),
            });
        }
        ensure!(
            tokio::time::Instant::now() < deadline,
            "Steam QR code expired. Please connect again."
        );
        tokio::time::sleep(Duration::from_secs_f32(interval)).await;
        session = call(
            client,
            "PollAuthSessionStatus",
            Message::default().number(1, id).bytes(2, &request_id),
        )
        .await?;
        let refresh_token = session.s(3)?;
        if !refresh_token.is_empty() {
            let claims = refresh_token
                .split('.')
                .nth(1)
                .context("Invalid Steam session")?;
            let claims: serde_json::Value =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(claims)?)?;
            let steam_id = claims["sub"]
                .as_str()
                .context("Steam session has no identity")?
                .parse()?;
            // The CM server validates the token. Claims here only supply the login identity.
            return Ok(Credentials {
                account: session.s(6)?,
                steam_id,
                refresh_token,
            });
        }
        if session.n(1) != 0 {
            id = session.n(1);
        }
    }
}

pub async fn password(
    client: &reqwest::Client,
    username: String,
    password: String,
    mut codes: tokio::sync::mpsc::Receiver<String>,
    emit: &(impl Fn(Event) + Send + Sync),
) -> Result<Credentials> {
    use rsa::{rand_core::OsRng, BigUint, Pkcs1v15Encrypt, RsaPublicKey};
    let password = zeroize::Zeroizing::new(password);
    let username = username.trim();
    ensure!(
        !username.is_empty()
            && username.len() <= 128
            && !password.is_empty()
            && password.len() <= 1024,
        "Enter your Steam username and password"
    );
    let rsa = call(
        client,
        "GetPasswordRSAPublicKey",
        Message::default().bytes(1, username),
    )
    .await?;
    let modulus =
        BigUint::parse_bytes(rsa.s(1)?.as_bytes(), 16).context("Invalid Steam encryption key")?;
    let exponent =
        BigUint::parse_bytes(rsa.s(2)?.as_bytes(), 16).context("Invalid Steam encryption key")?;
    let public_key = RsaPublicKey::new(modulus, exponent)?;
    let encrypted = public_key
        .encrypt(&mut OsRng, Pkcs1v15Encrypt, password.as_bytes())
        .map_err(|_| anyhow::anyhow!("Could not encrypt Steam password"))?;
    drop(password);
    let device = Message::default()
        .bytes(1, "Kitor")
        .number(2, 1)
        .number(3, 20);
    let session = call(
        client,
        "BeginAuthSessionViaCredentials",
        Message::default()
            .bytes(1, "Kitor")
            .bytes(2, username)
            .bytes(3, STANDARD.encode(encrypted))
            .number(4, rsa.n(3))
            .number(5, 1)
            .number(6, 1)
            .number(7, 1)
            .bytes(8, "Client")
            .bytes(9, device.0),
    )
    .await?;
    let mut id = session.n(1);
    let steam_id = session.n(5);
    let request_id = session.b(2).to_vec();
    ensure!(
        id != 0 && steam_id != 0 && !request_id.is_empty(),
        "Steam did not create a login session"
    );
    let methods = session
        .repeated(4)
        .map(Fields::parse)
        .collect::<Result<Vec<_>>>()?;
    let code_type = [3, 2]
        .into_iter()
        .find(|kind| methods.iter().any(|m| m.n(1) == *kind));
    ensure!(
        code_type.is_some() || methods.iter().any(|m| matches!(m.n(1), 1 | 4 | 5)),
        "This Steam verification method is not supported"
    );
    emit(Event::AuthChallenge {
        code_type,
        message: if code_type == Some(2) {
            "Enter the Steam Guard code sent to your email."
        } else if code_type == Some(3) {
            "Enter the code from Steam Guard."
        } else if methods.iter().any(|m| m.n(1) == 4) {
            "Approve sign-in in the Steam app."
        } else if methods.iter().any(|m| m.n(1) == 5) {
            "Approve sign-in using the link sent to your email."
        } else {
            "Signing in…"
        }
        .into(),
    });
    let seconds = f32::from_bits(session.n(3) as u32);
    let seconds = if seconds.is_finite() {
        seconds.clamp(1.0, 10.0)
    } else {
        3.0
    };
    let mut ticker = tokio::time::interval(Duration::from_secs_f32(seconds));
    let deadline = tokio::time::sleep(Duration::from_secs(180));
    tokio::pin!(deadline);
    loop {
        tokio::select! {
            _ = &mut deadline => anyhow::bail!("Steam sign-in expired. Please sign in again."),
            code = codes.recv() => {
                let code = zeroize::Zeroizing::new(code.context("Steam sign-in cancelled")?);
                let kind = code_type.context("Steam is waiting for mobile approval")?;
                if let Err(error) = call(client, "UpdateAuthSessionWithSteamGuardCode", Message::default().number(1, id).fixed(2, steam_id).bytes(3, code.trim()).number(4, kind)).await {
                    emit(Event::AuthChallenge { code_type, message: error.to_string() });
                }
            },
            _ = ticker.tick() => {
                let response = call(client, "PollAuthSessionStatus", Message::default().number(1, id).bytes(2, &request_id)).await?;
                let refresh_token = response.s(3)?;
                if !refresh_token.is_empty() {
                    return Ok(Credentials { account: response.s(6)?, steam_id, refresh_token });
                }
                if response.n(1) != 0 { id = response.n(1); }
            }
        }
    }
}
