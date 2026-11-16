//! Native Steam content transport. No Steam client or downloader subprocess.
mod auth;
mod catalog;
mod cm;
mod content;
mod wire;
pub use catalog::installed_build;
use anyhow::{ensure, Context, Result};
use serde::Serialize;
use std::{path::Path, time::Duration};
use wire::{Fields, Message};
#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    AuthChallenge {
        code_type: Option<u64>,
        message: String,
    },
    Qr {
        url: String,
        svg: String,
    },
    Status {
        message: String,
    },
    Progress {
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    Connected {
        steam_id: String,
        account: String,
    },
}
fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}
pub async fn connect(emit: impl Fn(Event) + Send + Sync) -> Result<()> {
    let client = client()?;
    let credentials = auth::qr(&client, &emit).await?;
    let _connection = cm::Connection::login(&client, &credentials).await?;
    let identity = (
        credentials.steam_id.to_string(),
        credentials.account.clone(),
    );
    credentials.save()?;
    emit(Event::Connected {
        steam_id: identity.0,
        account: identity.1,
    });
    Ok(())
}
pub async fn connect_password(
    username: String,
    password: String,
    codes: tokio::sync::mpsc::Receiver<String>,
    emit: impl Fn(Event) + Send + Sync,
) -> Result<()> {
    let client = client()?;
    let credentials = auth::password(&client, username, password, codes, &emit).await?;
    let _connection = cm::Connection::login(&client, &credentials).await?;
    credentials.save()?;
    emit(Event::Connected {
        steam_id: credentials.steam_id.to_string(),
        account: credentials.account,
    });
    Ok(())
}
pub async fn disconnect() -> Result<()> {
    tokio::task::spawn_blocking(auth::Credentials::delete).await?
}
pub async fn connected() -> Result<bool> {
    Ok(tokio::task::spawn_blocking(auth::Credentials::load)
        .await??
        .is_some())
}
pub async fn download(
    app: u32,
    depot: u32,
    destination: &Path,
    emit: impl Fn(Event) + Send + Sync,
) -> Result<()> {
    ensure!(
        !destination.exists(),
        "Download folder already exists; choose a new folder"
    );
    let credentials = tokio::task::spawn_blocking(auth::Credentials::load)
        .await??
        .context("Connect Steam downloads first")?;
    let started = std::time::Instant::now();
    let client = client()?;
    emit(Event::Status {
        message: "Connecting to Steam".into(),
    });
    let mut cm = cm::Connection::login(&client, &credentials).await?;
    let appinfo = cm
        .request(
            8903,
            8904,
            None,
            Message::default()
                .bytes(2, Message::default().number(1, app as u64).0)
                .number(7, 1),
        )
        .await?;
    let appinfo = appinfo
        .repeated(1)
        .map(Fields::parse)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .find(|f| f.n(1) == app as u64)
        .context("Steam did not return this game's metadata")?;
    let manifest = catalog::manifest(appinfo.b(5), depot)?;
    let key = cm
        .request(
            5438,
            5439,
            None,
            Message::default()
                .number(1, depot as u64)
                .number(2, app as u64),
        )
        .await?;
    ensure!(
        key.n(1) == 1 && key.n(2) == depot as u64 && key.b(3).len() == 32,
        "Steam did not authorize this game's files"
    );
    let code = cm
        .unified(
            "ContentServerDirectory.GetManifestRequestCode#1",
            Message::default()
                .number(1, app as u64)
                .number(2, depot as u64)
                .number(3, manifest)
                .bytes(4, "public"),
        )
        .await?
        .n(1);
    ensure!(code != 0, "Steam did not authorize this manifest");
    let servers = cm
        .unified(
            "ContentServerDirectory.GetServersForSteamPipe#1",
            Message::default().number(1, cm.cell_id).number(2, 8),
        )
        .await?;
    let mut hosts = vec![];
    for s in servers.repeated(1) {
        let s = Fields::parse(s)?;
        let vhost = s.s(9)?;
        let host = if vhost.is_empty() { s.s(8)? } else { vhost };
        if host.ends_with(".steamcontent.com")
            && host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
        {
            hosts.push(host);
        }
    }
    ensure!(
        !hosts.is_empty(),
        "Steam returned no supported content servers"
    );
    let parent = destination
        .parent()
        .context("Download requires an absolute destination")?;
    std::fs::create_dir_all(parent)?;
    let staging = tempfile::tempdir_in(parent)?;
    emit(Event::Status {
        message: "Downloading game".into(),
    });
    tokio::select! {r=content::download(&client,&hosts,key.b(3),depot,manifest,code,staging.path(),&emit)=>r?,r=cm.keep_alive()=>{r?;anyhow::bail!("Steam connection ended");}}
    ensure!(
        !destination.exists(),
        "Download destination was created during download"
    );
    std::fs::rename(staging.path(), destination)?;
    emit(Event::Status {
        message: format!(
            "Download verified in {:.1} seconds",
            started.elapsed().as_secs_f64()
        ),
    });
    Ok(())
}

#[derive(Serialize)]
pub struct OwnedLibrary {
    pub steam_id: String,
    pub owned_app_ids: Vec<u32>,
}

/// Query the signed-in player's library over the authenticated Steam connection.
/// This does not use a Web API key or infer ownership from public app metadata.
pub async fn owned_games() -> Result<Option<OwnedLibrary>> {
    let Some(credentials) = tokio::task::spawn_blocking(auth::Credentials::load).await?? else {
        return Ok(None);
    };
    let client = client()?;
    let mut connection = cm::Connection::login(&client, &credentials).await?;
    let response = connection
        .unified(
            "Player.GetOwnedGames#1",
            Message::default()
                .number(1, credentials.steam_id)
                .number(2, 0)
                .number(3, 1)
                .number(5, 1)
                .number(6, 0)
                .number(9, 0),
        )
        .await?;
    // Missing data is not evidence of an empty library.
    ensure!(
        response.0.iter().any(|(tag, _)| *tag == 1),
        "Steam did not return the owned-game library. Please reconnect and retry."
    );
    let mut ids = std::collections::BTreeSet::new();
    let mut count = 0u64;
    for game in response.repeated(2) {
        let game = Fields::parse(game)?;
        count += 1;
        // Shared titles must not be presented as personally owned.
        if game.n(21) == 0 {
            let id = u32::try_from(game.n(1)).context("Invalid Steam app ID")?;
            ensure!(id != 0, "Steam returned an invalid owned game");
            ids.insert(id);
        }
    }
    ensure!(
        count == response.n(1),
        "Steam returned an incomplete owned-game library. Please refresh."
    );
    Ok(Some(OwnedLibrary {
        steam_id: credentials.steam_id.to_string(),
        owned_app_ids: ids.into_iter().collect(),
    }))
}
