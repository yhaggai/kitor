use crate::{
    auth::Credentials,
    wire::{Fields, Message},
};
use anyhow::{bail, ensure, Context, Result};
use futures::{SinkExt, StreamExt};
use std::{collections::VecDeque, io::Read, time::Duration};
use tokio_tungstenite::{
    connect_async, tungstenite::Message as Ws, MaybeTlsStream, WebSocketStream,
};
struct Packet {
    kind: u32,
    header: Fields,
    body: Fields,
}
pub struct Connection {
    socket: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    queue: VecDeque<Packet>,
    steam_id: u64,
    session: u64,
    job: u64,
    pub licenses: Vec<u32>,
    pub cell_id: u64,
    heartbeat: tokio::time::Interval,
}
impl Connection {
    pub async fn login(client: &reqwest::Client, credentials: &Credentials) -> Result<Self> {
        let directory: serde_json::Value = client
            .get("https://api.steampowered.com/ISteamDirectory/GetCMListForConnect/v1/")
            .query(&[("cellid", "0"), ("cmtype", "websockets")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let servers = directory["response"]["serverlist"]
            .as_array()
            .context("Steam returned no connection servers")?;
        let mut socket = None;
        for s in servers.iter().take(5) {
            let Some(host) = s["endpoint"].as_str() else {
                continue;
            };
            ensure!(
                host.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".-:".contains(&b)),
                "Invalid Steam server"
            );
            if let Ok(Ok((ws, _))) = tokio::time::timeout(
                Duration::from_secs(15),
                connect_async(format!("wss://{host}/cmsocket/")),
            )
            .await
            {
                socket = Some(ws);
                break;
            }
        }
        let mut c = Self {
            socket: socket.context("Could not connect to Steam")?,
            queue: VecDeque::new(),
            steam_id: credentials.steam_id,
            session: 0,
            job: 1,
            licenses: vec![],
            cell_id: 0,
            heartbeat: tokio::time::interval(Duration::from_secs(20)),
        };
        c.send(
            9805,
            Message::default(),
            Message::default().number(1, 65580),
        )
        .await?;
        use rsa::rand_core::RngCore;
        let login_id = rsa::rand_core::OsRng.next_u32();
        let logon = Message::default()
            .number(1, 65580)
            .number(7, 20)
            .number(8, 1)
            .number(2, login_id as u64)
            // Set both current and legacy login IDs so parallel clients remain distinct.
            .bytes(11, Message::default().fixed32(1, login_id).0)
            .bytes(6, "english")
            .bytes(50, &credentials.account)
            .bytes(96, "Kitor")
            .bytes(108, &credentials.refresh_token);
        c.send(5514, Message::default(), logon).await?;
        tokio::time::timeout(Duration::from_secs(45), async {
            loop {
                let p = c.next().await?;
                if p.kind == 751 {
                    ensure!(
                        p.body.n(1) == 1,
                        "Steam login rejected (code {}). Reconnect Steam.",
                        p.body.n(1)
                    );
                    c.session = p.header.n(2);
                    c.cell_id = p.body.n(7);
                    let interval = p.body.n(3).max(5);
                    c.heartbeat = tokio::time::interval(Duration::from_secs(interval));
                    return Ok::<_, anyhow::Error>(());
                }
            }
        })
        .await
        .context("Steam login timed out")??;
        Ok(c)
    }
    async fn send(&mut self, kind: u32, header: Message, body: Message) -> Result<()> {
        let header = header.fixed(1, self.steam_id).number(2, self.session).0;
        let mut b = Vec::with_capacity(8 + header.len() + body.0.len());
        b.extend((kind | 0x80000000).to_le_bytes());
        b.extend((header.len() as u32).to_le_bytes());
        b.extend(header);
        b.extend(body.0);
        self.socket.send(Ws::Binary(b.into())).await?;
        Ok(())
    }
    fn unpack(&mut self, b: &[u8], depth: usize) -> Result<()> {
        ensure!(
            depth < 8 && b.len() >= 8 && b.len() <= 32 * 1024 * 1024,
            "Invalid Steam packet"
        );
        let raw = u32::from_le_bytes(b[..4].try_into()?);
        if raw & 0x80000000 == 0 {
            return Ok(());
        }
        let kind = raw & 0x7fffffff;
        let size = u32::from_le_bytes(b[4..8].try_into()?) as usize;
        ensure!(size <= b.len() - 8, "Truncated Steam header");
        let header = Fields::parse(&b[8..8 + size])?;
        let body = Fields::parse(&b[8 + size..])?;
        if kind == 1 {
            let mut data = body.b(2).to_vec();
            if body.n(1) != 0 {
                let mut out = vec![];
                flate2::read::GzDecoder::new(data.as_slice())
                    .take(32 * 1024 * 1024 + 1)
                    .read_to_end(&mut out)?;
                ensure!(
                    out.len() <= 32 * 1024 * 1024,
                    "Steam multi packet too large"
                );
                data = out;
            }
            let mut pos = 0;
            while pos < data.len() {
                ensure!(data.len() - pos >= 4, "Truncated Steam multi packet");
                let n = u32::from_le_bytes(data[pos..pos + 4].try_into()?) as usize;
                pos += 4;
                ensure!(n <= data.len() - pos, "Truncated Steam multi packet");
                self.unpack(&data[pos..pos + n], depth + 1)?;
                pos += n;
            }
        } else {
            if kind == 780 {
                ensure!(body.n(1) == 1, "Steam license query failed");
                for license in body.repeated(2) {
                    self.licenses.push(Fields::parse(license)?.n(1) as u32);
                }
            }
            ensure!(self.queue.len() < 10000, "Too many pending Steam messages");
            self.queue.push_back(Packet { kind, header, body });
        }
        Ok(())
    }
    async fn next(&mut self) -> Result<Packet> {
        loop {
            if let Some(p) = self.queue.pop_front() {
                return Ok(p);
            }
            tokio::select! {_ = self.heartbeat.tick()=>{self.send(703,Message::default(),Message::default()).await?;},msg=self.socket.next()=>{match msg.context("Steam disconnected")??{Ws::Binary(b)=>self.unpack(&b,0)?,Ws::Ping(b)=>self.socket.send(Ws::Pong(b)).await?,Ws::Close(_)=>bail!("Steam disconnected"),_=>{}}}}
        }
    }
    pub async fn request(
        &mut self,
        kind: u32,
        response: u32,
        method: Option<&str>,
        body: Message,
    ) -> Result<Fields> {
        self.job += 1;
        let job = self.job;
        let mut h = Message::default().fixed(10, job);
        if let Some(m) = method {
            h = h.bytes(12, m);
        }
        self.send(kind, h, body).await?;
        tokio::time::timeout(Duration::from_secs(45), async {
            loop {
                let p = self.next().await?;
                if p.kind == response && p.header.n(11) == job {
                    ensure!(
                        p.header.n(13) == 0 || p.header.n(13) == 1,
                        "Steam request rejected (code {})",
                        p.header.n(13)
                    );
                    return Ok(p.body);
                }
            }
        })
        .await
        .context("Steam request timed out")?
    }
    pub async fn unified(&mut self, method: &str, body: Message) -> Result<Fields> {
        self.request(151, 147, Some(method), body).await
    }
    pub async fn keep_alive(&mut self) -> Result<()> {
        loop {
            self.next().await?;
        }
    }
}
