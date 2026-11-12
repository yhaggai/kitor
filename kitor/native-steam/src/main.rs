use kitor_steam::{connect, download, Event};
fn emit(event: Event) {
    match event {
        Event::AuthChallenge { message, .. } => println!("{message}"),
        Event::Qr { url, .. } => {
            println!("Scan with Steam mobile app: {url}");
            if let Ok(q) = qrcode::QrCode::new(url.as_bytes()) {
                println!(
                    "{}",
                    q.render::<qrcode::render::unicode::Dense1x2>()
                        .quiet_zone(true)
                        .build()
                );
            }
        }
        Event::Status { message } => println!("{message}"),
        Event::Connected { account, .. } => println!("Connected as {account}"),
        Event::Progress {
            downloaded_bytes,
            total_bytes,
        } => println!("Downloaded {downloaded_bytes}/{total_bytes} bytes"),
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("connect") => connect(emit).await,
        Some("library") => {
            match kitor_steam::owned_games().await? {
                Some(library) => println!("{}", serde_json::to_string(&library)?),
                None => println!("No native Steam session"),
            }
            Ok(())
        }
        Some("download") if a.len() == 5 => {
            download(
                a[2].parse()?,
                a[3].parse()?,
                std::path::Path::new(&a[4]),
                emit,
            )
            .await
        }
        _ => anyhow::bail!("Usage: kitor-steam connect | download APP DEPOT DESTINATION"),
    }
}
