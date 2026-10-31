//! Install owned Windows GOG releases as unpatched, managed ScummVM games.
use super::{game_service, gog_service, registry_service};
use crate::{models::game, AppState};
use futures::StreamExt;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tokio::io::AsyncWriteExt;

static INSTALL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn supported(def: &registry_service::GameDefinition) -> bool {
    def.gog.is_some() && !def.behavior.launch_profile(false).is_native()
}

pub fn capabilities(id: i64, game_id: &str) -> Result<bool, String> {
    Ok(cfg!(target_os = "macos")
        && registry_service::load_registry()?.games.iter().any(|g| {
            g.scummvm_id == game_id && g.gog.as_ref().is_some_and(|g| g.id == id) && supported(g)
        }))
}

#[derive(Deserialize)]
struct InstallerFile {
    downlink: String,
}
#[derive(Deserialize)]
struct Installer {
    os: String,
    language: String,
    files: Vec<InstallerFile>,
}

fn select_installer(value: serde_json::Value) -> Result<Installer, String> {
    let installers: Vec<Installer> = serde_json::from_value(
        value
            .pointer("/downloads/installers")
            .cloned()
            .ok_or("GOG did not return any installers")?,
    )
    .map_err(|_| "GOG returned invalid installer metadata")?;
    // Pick one complete release, including every .bin part, never combine languages.
    installers
        .into_iter()
        .find(|i| i.os == "windows" && i.language == "en" && !i.files.is_empty())
        .ok_or_else(|| "No English Windows installer is available for this game on GOG.".into())
}

fn https_url(value: &str, api: bool) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| "Invalid GOG download URL")?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || (api && url.host_str() != Some("api.gog.com"))
    {
        return Err("Unexpected GOG download URL".into());
    }
    Ok(url)
}

fn checked_filename(name: &str) -> Result<String, String> {
    if name.is_empty()
        || name.chars().any(|c| c.is_control())
        || name.contains(['/', '\\', ':', '%'])
        || !(name.to_ascii_lowercase().ends_with(".exe")
            || name.to_ascii_lowercase().ends_with(".bin"))
    {
        return Err("GOG returned an unsupported installer filename".into());
    }
    Ok(name.into())
}

fn decode_filename(name: &str) -> Result<String, String> {
    let decoded = percent_encoding::percent_decode_str(name)
        .decode_utf8()
        .map_err(|_| "GOG returned an invalid installer filename encoding")?;
    checked_filename(&decoded)
}

fn filename(url: &url::Url) -> Result<String, String> {
    // query_pairs already decodes query values. Decode only the basename of a
    // URL path, so an encoded separator cannot become a local path component.
    if let Some((_, path)) = url.query_pairs().find(|(key, _)| key == "path") {
        return checked_filename(path.rsplit('/').next().unwrap_or_default());
    }
    decode_filename(url.path().rsplit('/').next().unwrap_or_default())
}

fn response_filename(
    headers: &reqwest::header::HeaderMap,
    final_url: &url::Url,
    original_url: &url::Url,
) -> Result<String, String> {
    // A signed CDN URL may end in /secure or an opaque ID. The response can
    // provide the real name, which must be preserved for multipart installers.
    if let Some(header) = headers.get(reqwest::header::CONTENT_DISPOSITION) {
        let header = header.to_str().map_err(|_| "Invalid GOG filename header")?;
        let params: Vec<_> = header
            .split(';')
            .skip(1)
            .filter_map(|part| part.trim().split_once('='))
            .map(|(key, value)| (key.trim(), value.trim().trim_matches('"')))
            .collect();
        if let Some((_, value)) = params
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("filename*"))
        {
            let mut parts = value.splitn(3, '\'');
            let encoding = parts.next().unwrap_or_default();
            let _language = parts.next();
            if !encoding.eq_ignore_ascii_case("utf-8") {
                return Err("Unsupported GOG filename encoding".into());
            }
            return decode_filename(parts.next().ok_or("Invalid GOG filename header")?);
        }
        if let Some((_, value)) = params
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("filename"))
        {
            return checked_filename(value);
        }
    }
    filename(final_url).or_else(|_| filename(original_url))
}

// Catalog sizes can describe an older installer revision. Validate the bytes
// promised by this HTTP response; extraction independently verifies Inno CRCs.
fn validate_transfer_size(received: u64, expected: Option<u64>) -> Result<(), String> {
    if received == 0 {
        return Err("GOG returned an empty installer. Please retry.".into());
    }
    if let Some(expected) = expected {
        if received != expected {
            return Err(format!("GOG download was incomplete: received {received} of {expected} bytes. Please retry."));
        }
    }
    Ok(())
}

fn progress(app: &tauri::AppHandle, id: &str, step: &str, percentage: Option<u64>) {
    let _ = app.emit(
        "gog-game-install-progress",
        serde_json::json!({
            "game_id": id, "step": step, "percentage": percentage
        }),
    );
}

async fn extractor(app: &tauri::AppHandle, id: &str) -> Result<PathBuf, String> {
    async fn find() -> Option<PathBuf> {
        for path in [
            "innoextract",
            "/opt/homebrew/bin/innoextract",
            "/usr/local/bin/innoextract",
        ] {
            if tokio::process::Command::new(path)
                .arg("--version")
                .output()
                .await
                .is_ok_and(|o| o.status.success())
            {
                return Some(path.into());
            }
        }
        None
    }
    if let Some(path) = find().await {
        return Ok(path);
    }
    progress(app, id, "Preparing installation tools…", None);
    for brew in ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"] {
        if Path::new(brew).is_file() {
            let status = tokio::process::Command::new(brew)
                .args(["install", "innoextract"])
                .env("HOMEBREW_NO_AUTO_UPDATE", "1")
                .output()
                .await
                .map_err(|_| "Could not prepare the GOG extraction tool")?;
            if status.status.success() {
                if let Some(path) = find().await {
                    return Ok(path);
                }
            }
            return Err(
                "Could not install the GOG extraction tool. Check Homebrew and retry.".into(),
            );
        }
    }
    Err(
        "GOG installation requires innoextract. Install Homebrew or innoextract, then retry."
            .into(),
    )
}

async fn json(
    client: &reqwest::Client,
    url: url::Url,
    token: &str,
) -> Result<serde_json::Value, String> {
    client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|_| "Could not contact GOG. Check your connection.")?
        .error_for_status()
        .map_err(|_| "GOG denied the download. Reconnect the account that owns this game.")?
        .json()
        .await
        .map_err(|_| "GOG returned an invalid download response".into())
}

// Detection uses the requested catalog identity, including collection subfolders.
// Never follow extracted links or register data outside the staging directory.
fn find_game(root: &Path, id: &str) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut directories = vec![(root.clone(), 0)];
    let mut found = None;
    while let Some((path, depth)) = directories.pop() {
        if let Some(game) = game_service::detect_game_for_id(&path, id)? {
            let detected = PathBuf::from(game.install_path)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !detected.starts_with(&root) {
                return Err("Extracted game path escapes its installation".into());
            }
            found.get_or_insert(detected);
        }
        for entry in std::fs::read_dir(&path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_symlink() || (!kind.is_file() && !kind.is_dir()) {
                return Err("Unexpected link or special file in GOG game data".into());
            }
            if kind.is_dir() {
                if depth >= 12 {
                    return Err("GOG installation has too many nested folders".into());
                }
                directories.push((entry.path(), depth + 1));
            }
        }
    }
    found.ok_or_else(|| "The downloaded release does not match this game's supported edition. No game was added.".into())
}

pub async fn install(
    id: i64,
    game_id: String,
    state: &AppState,
    app: &tauri::AppHandle,
    install_root: Option<PathBuf>,
) -> Result<game::Model, String> {
    let _guard = INSTALL
        .try_lock()
        .map_err(|_| "Another GOG installation is running")?;
    if !capabilities(id, &game_id)? {
        return Err("This game does not support automatic GOG installation on Mac".into());
    }
    if let Some(game) = game_service::get_all_games(&state.db)
        .await?
        .into_iter()
        .find(|g| g.scummvm_id == game_id)
    {
        if Path::new(&game.install_path).is_dir() {
            return Ok(game);
        }
        return Err(
            "This game's library folder is missing. Remove its library entry before reinstalling."
                .into(),
        );
    }
    let def = registry_service::load_registry()?
        .games
        .into_iter()
        .find(|g| g.scummvm_id == game_id)
        .ok_or("Game not found")?;
    let games = install_root.unwrap_or_else(|| state.app_data_dir.join("games"));
    if !games.is_absolute() {
        return Err("Choose an absolute installation folder".into());
    }
    std::fs::create_dir_all(&games).map_err(|e| e.to_string())?;
    let destination = games.join(format!("{game_id}-gog"));
    // Registration can be retried without redownloading or overwriting the original files.
    let game_path = if destination.exists() {
        find_game(&destination, &game_id)?
    } else {
        progress(app, &game_id, "Checking your GOG library…", None);
        if !gog_service::get_user_games(&state.db)
            .await?
            .iter()
            .any(|g| g.id == id)
        {
            return Err("Connect the GOG account that owns this game, then retry.".into());
        }
        let tool = extractor(app, &game_id).await?;
        let token = gog_service::get_valid_token(&state.db).await?;
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?;
        let metadata = json(
            &client,
            https_url(
                &format!("https://api.gog.com/products/{id}?expand=downloads"),
                true,
            )?,
            &token,
        )
        .await?;
        let installer = select_installer(metadata)?;
        let downloads = tempfile::tempdir_in(&games).map_err(|e| e.to_string())?;
        let staging = tempfile::tempdir_in(&games).map_err(|e| e.to_string())?;
        let part_count = installer.files.len();
        let mut executables = Vec::new();
        let mut names = std::collections::HashSet::new();
        // Long transfers use per-read timeouts rather than the metadata request timeout.
        let transfer = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(30))
            .read_timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| e.to_string())?;
        for (index, part) in installer.files.into_iter().enumerate() {
            let resolved = json(&client, https_url(&part.downlink, true)?, &token).await?;
            let url = https_url(
                resolved["downlink"]
                    .as_str()
                    .ok_or("Missing GOG download link")?,
                false,
            )?;
            let response = transfer
                .get(url.clone())
                .send()
                .await
                .map_err(|_| "Could not download game files")?
                .error_for_status()
                .map_err(|_| "GOG game download failed. Retry or reconnect GOG.")?;
            if response.status() != reqwest::StatusCode::OK {
                return Err("GOG returned a partial installer response. Please retry.".into());
            }
            let expected_bytes = response.content_length();
            let name = response_filename(response.headers(), response.url(), &url)?;
            if !names.insert(name.to_ascii_lowercase()) {
                return Err("Duplicate GOG installer part".into());
            }
            let path = downloads.path().join(&name);
            if name.to_ascii_lowercase().ends_with(".exe") {
                executables.push(path.clone());
            }
            let mut file = tokio::fs::File::create(&path)
                .await
                .map_err(|e| e.to_string())?;
            let mut stream = response.bytes_stream();
            let mut size = 0u64;
            let mut last = std::time::Instant::now();
            let step = format!("Downloading game… (file {} of {part_count})", index + 1);
            progress(
                app,
                &game_id,
                &step,
                expected_bytes.filter(|n| *n > 0).map(|_| 0),
            );
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|_| "Game download was interrupted. Please retry.")?;
                file.write_all(&chunk).await.map_err(|e| e.to_string())?;
                size += chunk.len() as u64;
                if last.elapsed().as_millis() >= 250 {
                    progress(
                        app,
                        &game_id,
                        &step,
                        expected_bytes
                            .filter(|n| *n > 0)
                            .map(|total| (size.saturating_mul(100) / total).min(100)),
                    );
                    last = std::time::Instant::now();
                }
            }
            file.flush().await.map_err(|e| e.to_string())?;
            validate_transfer_size(size, expected_bytes)?;
            progress(app, &game_id, &step, Some(100));
        }
        if executables.len() != 1 {
            return Err("This GOG release needs an unsupported installer layout".into());
        }
        progress(app, &game_id, "Preparing game files…", None);
        let output = tokio::process::Command::new(tool)
            .args([
                "--extract",
                "--gog",
                "--lowercase",
                "--silent",
                "--output-dir",
            ])
            .arg(staging.path())
            .arg(&executables[0])
            .output()
            .await
            .map_err(|_| "Could not extract the GOG installer")?;
        if !output.status.success() {
            return Err("Could not extract this GOG release. Your library was not changed.".into());
        }
        let source = staging.path().to_path_buf();
        let identity = game_id.clone();
        let relative = tokio::task::spawn_blocking(move || {
            let path = find_game(&source, &identity)?;
            path.strip_prefix(source.canonicalize().map_err(|e| e.to_string())?)
                .map(Path::to_path_buf)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;
        std::fs::rename(staging.path(), &destination).map_err(|e| e.to_string())?;
        destination.join(relative)
    };
    progress(app, &game_id, "Adding game to library…", None);
    let game = game_service::add_game_manually(
        &state.db,
        game_id.clone(),
        def.title,
        game_path.to_string_lossy().into_owned(),
        state.app_data_dir.clone(),
    )
    .await?;
    progress(app, &game_id, "Ready to play", Some(100));
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chooses_one_english_windows_release_with_all_parts() {
        let data = serde_json::json!({"downloads":{"installers":[
            {"os":"osx","language":"en","files":[{"downlink":"mac","size":1}]},
            {"os":"windows","language":"de","files":[{"downlink":"de","size":1}]},
            {"os":"windows","language":"en","files":[{"downlink":"exe","size":10},{"downlink":"bin","size":20}]}
        ]}});
        let installer = select_installer(data).unwrap();
        assert_eq!(installer.files.len(), 2);
        assert_eq!(installer.files[1].downlink, "bin");
        assert!(select_installer(serde_json::json!({"downloads":{"installers":[]}})).is_err());
    }
    #[test]
    fn validates_download_locations_and_part_names() {
        assert!(https_url("https://evil.example/file", true).is_err());
        assert!(https_url("http://api.gog.com/file", true).is_err());
        assert_eq!(
            filename(&https_url("https://cdn.gog.com/setup.exe?token=secret", false).unwrap())
                .unwrap(),
            "setup.exe"
        );
        assert_eq!(
            filename(
                &https_url(
                    "https://cdn.gog.com/secure?path=/game/setup-1.bin&token=x",
                    false
                )
                .unwrap()
            )
            .unwrap(),
            "setup-1.bin"
        );
        assert!(filename(&https_url("https://cdn.gog.com/file.zip", false).unwrap()).is_err());
    }
    #[test]
    fn handles_encoded_names_redirects_and_download_headers() {
        let encoded = https_url(
            "https://cdn.gog.com/setup_torins_passage_2.0_%2812345%29.exe?token=private",
            false,
        )
        .unwrap();
        assert_eq!(
            filename(&encoded).unwrap(),
            "setup_torins_passage_2.0_(12345).exe"
        );
        let opaque = https_url("https://cdn.gog.com/secure?token=private", false).unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            "attachment; filename=\"setup_torin-1.bin\""
                .parse()
                .unwrap(),
        );
        assert_eq!(
            response_filename(&headers, &opaque, &opaque).unwrap(),
            "setup_torin-1.bin"
        );
        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            "attachment; filename=ignored.exe; filename*=UTF-8''setup_torin_%281%29.exe"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            response_filename(&headers, &opaque, &opaque).unwrap(),
            "setup_torin_(1).exe"
        );
        headers.clear();
        assert_eq!(
            response_filename(&headers, &encoded, &opaque).unwrap(),
            "setup_torins_passage_2.0_(12345).exe"
        );
        assert_eq!(
            response_filename(&headers, &opaque, &encoded).unwrap(),
            "setup_torins_passage_2.0_(12345).exe"
        );
        assert!(
            filename(&https_url("https://cdn.gog.com/setup%2Fescape.exe", false).unwrap()).is_err()
        );
        assert!(filename(&https_url("https://cdn.gog.com/setup%00.exe", false).unwrap()).is_err());
        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            "attachment; filename=\"../escape.exe\"".parse().unwrap(),
        );
        assert!(response_filename(&headers, &encoded, &encoded).is_err());
    }

    #[test]
    fn checks_actual_response_length_instead_of_stale_catalog_size() {
        // The catalog may say 10 bytes while the current CDN file has 12.
        let installer = select_installer(serde_json::json!({"downloads":{"installers":[
            {"os":"windows","language":"en","files":[{"downlink":"exe","size":10}]}
        ]}}))
        .unwrap();
        assert_eq!(installer.files.len(), 1);
        assert!(validate_transfer_size(12, Some(12)).is_ok());
        assert!(validate_transfer_size(12, None).is_ok());
        assert!(validate_transfer_size(0, None).is_err());
        assert!(validate_transfer_size(0, Some(0)).is_err());
        let error = validate_transfer_size(9, Some(12)).unwrap_err();
        assert!(error.contains("received 9 of 12 bytes"));
        assert!(validate_transfer_size(13, Some(12)).is_err());
    }

    #[test]
    fn detects_nested_original_game_and_rejects_links() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("app/data");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("sky.dsk"), b"original game").unwrap();
        assert_eq!(
            find_game(dir.path(), "sky").unwrap(),
            game.canonicalize().unwrap()
        );
        assert!(find_game(dir.path(), "torin").is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&game, dir.path().join("link")).unwrap();
            assert!(find_game(dir.path(), "sky").is_err());
        }
    }
    #[test]
    fn only_offers_games_playable_without_a_patch() {
        let registry = registry_service::load_registry().unwrap();
        assert!(supported(
            registry
                .games
                .iter()
                .find(|g| g.scummvm_id == "sky")
                .unwrap()
        ));
        for id in ["monkey-se", "monkey2-se"] {
            assert_eq!(supported(
                registry.games.iter().find(|g| g.scummvm_id == id).unwrap()
            ), cfg!(target_os = "macos"));
        }
        assert!(!supported(
            registry
                .games
                .iter()
                .find(|g| g.scummvm_id == "monkey-se-native")
                .unwrap()
        ));
    }
}

#[cfg(test)]
mod torin_tests {
    use super::*;

    #[test]
    fn torin_rejects_other_sci_games_with_the_same_filenames() {
        let dir = tempfile::tempdir().unwrap();
        for (name, size) in [("RESMAP.000", 9799), ("RESSCI.000", 55973887)] {
            let file = std::fs::File::create(dir.path().join(name)).unwrap();
            file.set_len(size).unwrap();
        }
        assert!(game_service::detect_game_for_id(dir.path(), "torin").unwrap().is_none());
    }

    #[tokio::test]
    #[ignore = "Read-only check of the explicitly supplied KITOR_TORIN_LIBRARY database"]
    async fn owned_torin_is_registered_unpatched_and_uses_scummvm() {
        let root = PathBuf::from(std::env::var_os("KITOR_TORIN_LIBRARY").expect("explicit library required"));
        let database = root.join("kitor.db");
        let db = sea_orm::Database::connect(format!("sqlite:{}?mode=ro", database.display())).await.unwrap();
        let games = game_service::get_games_with_patches(&db, root).await.unwrap();
        let game = games.iter().find(|g| g.scummvm_id == "torin").expect("Torin must be in the library");
        assert!(!game.is_patched);
        assert!(game.installed_patches.is_empty());
        assert!(game.requires_scummvm);
        let detected = game_service::detect_game_for_id(Path::new(&game.install_path), "torin").unwrap().unwrap();
        assert_eq!(detected.scummvm_id, "torin");
        let behavior = super::super::game_behavior::for_game("torin").unwrap();
        assert!(!behavior.launch_profile(false).is_native());
        assert!(!behavior.launch_profile(false).args.iter().any(|a| a == "--language=he"));
        println!("Verified registered, unpatched Torin with original ScummVM launch profile: {}", game.install_path);
    }
}
