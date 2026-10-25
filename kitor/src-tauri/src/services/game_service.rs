use sea_orm::*;
use std::path::Path;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::models::game::{self, Entity as Game};
use crate::models::installed_patch;
use crate::services::image_service;
use crate::services::registry_service;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPatchInfo {
    pub id: i32,
    pub scummvm_id: String,
    pub version: String,
    pub installed_date: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePatchInfo {
    pub language: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWithPatches {
    pub id: i32,
    pub scummvm_id: String,
    pub title: String,
    pub install_path: String,
    pub cover_url: Option<String>,
    pub detected_version: Option<String>,
    pub added_date: String,
    pub last_played: Option<String>,
    pub has_available_patch: bool,
    pub installed_patches: Vec<InstalledPatchInfo>,
    pub available_patches: Vec<AvailablePatchInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGameInfo {
    pub install_path: String,
    pub scummvm_id: String,
    pub title: String,
    pub available_patches_count: usize,
}


pub async fn get_all_games(db: &DatabaseConnection) -> Result<Vec<game::Model>, String> {
    Game::find()
        .all(db)
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))
}

pub async fn get_game_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<game::Model>, String> {
    Game::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| format!("Failed to fetch game: {}", e))
}

pub async fn scan_directory(
    db: &DatabaseConnection,
    scan_path: &str,
    app_data_dir: PathBuf,
) -> Result<Vec<game::Model>, String> {
    let path = Path::new(scan_path);
    if !path.exists() || !path.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    let mut games = Vec::new();

    // A selected game/app is itself a candidate; otherwise scan its children.
    let candidates = if detect_scummvm_game(path).is_some() {
        vec![path.to_path_buf()]
    } else {
        std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to read entry: {}", e))?
    };

    for entry_path in candidates {

        if entry_path.is_dir() {
            if let Some(game_info) = detect_scummvm_game(&entry_path) {
                // Check if game already exists
                let existing = Game::find()
                    .filter(game::Column::ScummvmId.eq(&game_info.scummvm_id))
                    .one(db)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?;

                if existing.is_none() {
                    let (launch_subdirectory, launch_args, cover_url) =
                        get_game_metadata_from_registry(&game_info.scummvm_id)?;
                    
                    let game = game::ActiveModel {
                        scummvm_id: Set(game_info.scummvm_id),
                        title: Set(game_info.title),
                        install_path: Set(game_info.install_path),
                        cover_url: Set(cover_url.clone()),
                        detected_version: Set(None),
                        added_date: Set(chrono::Utc::now().naive_utc()),
                        last_played: Set(None),
                        launch_subdirectory: Set(launch_subdirectory),
                        launch_args: Set(launch_args),
                        ..Default::default()
                    };

                    let result = game.insert(db).await
                        .map_err(|e| format!("Failed to insert game: {}", e))?;

                    if let Some(url) = cover_url {
                        let scummvm_id = result.scummvm_id.clone();
                        let app_data_dir = app_data_dir.clone();
                        tokio::spawn(async move {
                            let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
                        });
                    }
                    
                    games.push(result);
                }
            }
        }
    }

    Ok(games)
}

pub async fn add_game_manually(
    db: &DatabaseConnection,
    scummvm_id: String,
    title: String,
    install_path: String,
    app_data_dir: PathBuf,
) -> Result<game::Model, String> {
    let install_path = detect_game_in_folder(Path::new(&install_path))?
        .filter(|detected| detected.scummvm_id == scummvm_id)
        .map(|detected| detected.install_path)
        .unwrap_or(install_path);

    // Idempotent add: if game exists (unique scummvm_id), update metadata + return it.
    if let Some(existing) = Game::find()
        .filter(game::Column::ScummvmId.eq(&scummvm_id))
        .one(db)
        .await
        .map_err(|e| format!("Database error: {}", e))?
    {
        let mut active: game::ActiveModel = existing.into();
        active.title = Set(title);
        active.install_path = Set(install_path);

        let (subdirectory, args, cover_url) = get_game_metadata_from_registry(&scummvm_id)?;
        active.launch_subdirectory = Set(subdirectory);
        active.launch_args = Set(args);
        if cover_url.is_some() {
            active.cover_url = Set(cover_url.clone());
        }

        if let Some(url) = cover_url {
            let scummvm_id = scummvm_id.clone();
            let app_data_dir = app_data_dir.clone();
            tokio::spawn(async move {
                let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
            });
        }

        return active
            .update(db)
            .await
            .map_err(|e| format!("Failed to update existing game: {}", e));
    }

    let (launch_subdirectory, launch_args, cover_url) =
        get_game_metadata_from_registry(&scummvm_id)?;

    let game = game::ActiveModel {
        scummvm_id: Set(scummvm_id),
        title: Set(title),
        install_path: Set(install_path),
        cover_url: Set(cover_url.clone()),
        detected_version: Set(None),
        added_date: Set(chrono::Utc::now().naive_utc()),
        last_played: Set(None),
        launch_subdirectory: Set(launch_subdirectory),
        launch_args: Set(launch_args),
        ..Default::default()
    };

    let result = game.insert(db)
        .await
        .map_err(|e| format!("Failed to add game: {}", e))?;

    if let Some(url) = cover_url {
        let scummvm_id = result.scummvm_id.clone();
        tokio::spawn(async move {
            let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
        });
    }

    Ok(result)
}

pub async fn remove_game(db: &DatabaseConnection, id: i32) -> Result<(), String> {
    Game::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to remove game: {}", e))?;
    
    Ok(())
}

struct GameInfo {
    install_path: String,
    scummvm_id: String,
    title: String,
}

fn get_game_metadata_from_registry(
    scummvm_id: &str,
) -> Result<(Option<String>, Option<String>, Option<String>), String> {
    let registry = registry_service::load_registry()?;
    
    if let Some(game_def) = registry.games.iter().find(|g| g.scummvm_id == scummvm_id) {
        let cover_url = game_def.cover_url.clone();

        if let Some(launch_config) = &game_def.launch_config {
            let subdirectory = launch_config.subdirectory.clone();
            let args_json = if !launch_config.args.is_empty() {
                Some(serde_json::to_string(&launch_config.args)
                    .map_err(|e| format!("Failed to serialize launch args: {}", e))?)
            } else {
                None
            };
            return Ok((subdirectory, args_json, cover_url));
        }

        return Ok((None, None, cover_url));
    }
    
    Ok((None, None, None))
}

pub fn detect_game_in_folder(path: &Path) -> Result<Option<DetectedGameInfo>, String> {
    if !path.exists() || !path.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    // Load registry and try to match against game definitions
    let registry = registry_service::load_registry()?;

    // GOG macOS bundles wrap the actual data folder. Keep the resolved folder
    // as install_path so patch installation and ScummVM use the same files.
    let mut candidates = vec![path.to_path_buf()];
    if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("app")) {
        candidates.push(path.join("Contents/Resources/game/game"));
        candidates.push(path.join("Contents/Resources/game"));
    }

    for game_def in registry.games {
        if game_def.patches.is_empty() {
            continue;
        }
        // UNDERTALE's macOS archive lives inside its native app bundle.
        // Require the runner as well: game.ios alone is not a unique signature.
        if game_def.scummvm_id == "undertale" {
            for bundle in [path.to_path_buf(), path.join("UNDERTALE.app"),
                path.join("../..")] {
                let data = bundle.join("Contents/Resources");
                if data.join("game.ios").is_file()
                    && bundle.join("Contents/MacOS/Mac_Runner").is_file() {
                    return Ok(Some(DetectedGameInfo {
                        install_path: data.canonicalize().map_err(|e| e.to_string())?
                            .to_string_lossy().into_owned(),
                        scummvm_id: game_def.scummvm_id,
                        title: game_def.title,
                        available_patches_count: game_def.patches.len(),
                    }));
                }
            }
        }
        for candidate in &candidates {
            let mut data_paths = vec![candidate.clone()];
            if let Some(subdir) = &game_def.detection_patterns.detection_subdirectory {
                data_paths.push(candidate.join(subdir));
            }
            for data_path in data_paths {
                let Ok(entries) = std::fs::read_dir(&data_path) else {
                    continue;
                };
                let matches = entries.flatten().any(|entry| {
                    entry.file_type().is_ok_and(|kind| kind.is_file())
                        && game_def.detection_patterns.unique_files.iter().any(|name| {
                            entry.file_name().to_string_lossy().eq_ignore_ascii_case(name)
                        })
                });
                if matches {
                    return Ok(Some(DetectedGameInfo {
                        install_path: data_path.to_string_lossy().into_owned(),
                        scummvm_id: game_def.scummvm_id,
                        title: game_def.title,
                        available_patches_count: game_def.patches.len(),
                    }));
                }
            }
        }
    }

    Ok(None)
}

fn detect_scummvm_game(path: &Path) -> Option<GameInfo> {
    let detected = detect_game_in_folder(path).ok()??;
    Some(GameInfo {
        install_path: detected.install_path,
        scummvm_id: detected.scummvm_id,
        title: detected.title,
    })
}

pub async fn get_games_with_patches(
    db: &DatabaseConnection,
    app_data_dir: PathBuf,
) -> Result<Vec<GameWithPatches>, String> {
    // Get all games from DB
    let games = Game::find()
        .all(db)
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))?;
    
    // Load registry
    let registry = registry_service::load_registry()?;

    let _ = image_service::ensure_covers_dir(&app_data_dir);
    
    let mut games_with_patches = Vec::new();
    
    for game in games {
        // Find game definition in registry
        let game_def = registry.games.iter()
            .find(|g| g.scummvm_id == game.scummvm_id);
        
        // Get installed patches for this game
        let installed = installed_patch::Entity::find()
            .filter(installed_patch::Column::GameId.eq(game.id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch installed patches: {}", e))?;
        
        let installed_patches: Vec<InstalledPatchInfo> = installed.iter().map(|p| {
            InstalledPatchInfo {
                id: p.id,
                scummvm_id: p.scummvm_id.clone(),
                version: p.version.clone(),
                installed_date: p.installed_date.format("%Y-%m-%d").to_string(),
                status: p.status.clone(),
            }
        }).collect();
        
        // Get available patches from registry
        let available_patches: Vec<AvailablePatchInfo> = if let Some(def) = game_def {
            def.patches.iter().map(|p| {
                AvailablePatchInfo {
                    language: p.language.clone(),
                    version: p.version.clone(),
                    author: p.author.clone(),
                    description: p.description.clone(),
                }
            }).collect()
        } else {
            Vec::new()
        };
        
        // Only include games that have patches (available or installed)
        let has_available_patch = !available_patches.is_empty();
        let has_installed_patch = !installed_patches.is_empty();
        
        if has_available_patch || has_installed_patch {
            let registry_cover_url = game_def.and_then(|d| d.cover_url.clone());
            let cached_cover_path = image_service::get_cached_cover_path(&game.scummvm_id, &app_data_dir)?;

            if cached_cover_path.is_none() {
                if let Some(url) = registry_cover_url.clone().or_else(|| game.cover_url.clone()) {
                    let scummvm_id = game.scummvm_id.clone();
                    let app_data_dir = app_data_dir.clone();
                    tokio::spawn(async move {
                        let _ = image_service::download_and_cache_cover(&scummvm_id, &url, app_data_dir).await;
                    });
                }
            }

            let cover_url = if let Some(path) = cached_cover_path {
                Some(path.to_string_lossy().to_string())
            } else {
                registry_cover_url.or(game.cover_url.clone())
            };

            games_with_patches.push(GameWithPatches {
                id: game.id,
                scummvm_id: game.scummvm_id,
                title: game.title,
                install_path: game.install_path,
                cover_url,
                detected_version: game.detected_version,
                added_date: game.added_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_played: game.last_played.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                has_available_patch,
                installed_patches,
                available_patches,
            });
        }
    }
    
    Ok(games_with_patches)
}




#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("kitor-kyra-{}-{}",
                std::process::id(), chrono::Utc::now().timestamp_nanos_opt().unwrap()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn data(&self, relative: &str) -> PathBuf {
            let path = self.0.join(relative);
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("kyragem.pak"), b"fixture").unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn resolves_kyrandia_bundle_flat_and_dat_layouts() {
        let fixture = Fixture::new();
        for relative in ["flat", "dos/DAT", "Kyrandia.app/Contents/Resources/game/game"] {
            let data = fixture.data(relative);
            let selected = match relative {
                "dos/DAT" => fixture.0.join("dos"),
                "flat" => data.clone(),
                _ => fixture.0.join("Kyrandia.app"),
            };
            let detected = detect_game_in_folder(&selected).unwrap().unwrap();
            assert_eq!(detected.scummvm_id, "kyra1-cd");
            assert_eq!(PathBuf::from(detected.install_path), data);
            assert_eq!(detected.available_patches_count, 1);
        }
    }

    #[test]
    fn detects_undertale_bundle_parent_resources_and_windows() {
        let fixture = Fixture::new();
        let bundle = fixture.0.join("UNDERTALE.app");
        let data = bundle.join("Contents/Resources");
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        std::fs::write(data.join("game.ios"), b"fixture").unwrap();
        assert!(detect_game_in_folder(&data).unwrap().is_none());
        std::fs::write(bundle.join("Contents/MacOS/Mac_Runner"), b"runner").unwrap();
        for selected in [&fixture.0, &bundle, &data] {
            let detected = detect_game_in_folder(selected).unwrap().unwrap();
            assert_eq!(detected.scummvm_id, "undertale");
            assert_eq!(PathBuf::from(detected.install_path), data.canonicalize().unwrap());
        }
        let windows = fixture.0.join("windows");
        std::fs::create_dir(&windows).unwrap();
        std::fs::write(windows.join("UNDERTALE.exe"), b"runner").unwrap();
        assert_eq!(detect_game_in_folder(&windows).unwrap().unwrap().scummvm_id, "undertale");
    }

    #[test]
    fn does_not_detect_a_directory_named_like_a_game_file() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.0.join("KYRAGEM.PAK")).unwrap();
        assert!(detect_game_in_folder(&fixture.0).unwrap().is_none());
    }

    #[tokio::test]
    async fn scanning_and_manual_add_store_the_resolved_folder() {
        let fixture = Fixture::new();
        let bundle = fixture.0.join("Kyrandia.app");
        let data = fixture.data("Kyrandia.app/Contents/Resources/game/game");
        let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
        let games = scan_directory(&db, fixture.0.to_str().unwrap(), fixture.0.clone()).await.unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(Path::new(&games[0].install_path), data);
        remove_game(&db, games[0].id).await.unwrap();
        let games = scan_directory(&db, bundle.to_str().unwrap(), fixture.0.clone()).await.unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(Path::new(&games[0].install_path), data);
        let added = add_game_manually(&db, "kyra1-cd".into(), "Kyrandia".into(),
            bundle.to_string_lossy().into_owned(), fixture.0.clone()).await.unwrap();
        assert_eq!(added.id, games[0].id);
        assert_eq!(Path::new(&added.install_path), data);
        assert!(added.launch_args.unwrap().contains("kyra:kyra1"));
    }

    #[test]
    #[ignore = "requires a local GOG Kyrandia installation; set KITOR_KYRANDIA_APP"]
    fn detects_local_kyrandia_installation() {
        let path = PathBuf::from(std::env::var("KITOR_KYRANDIA_APP").unwrap());
        let detected = detect_game_in_folder(&path).unwrap().unwrap();
        assert_eq!(detected.scummvm_id, "kyra1-cd");
        assert_eq!(PathBuf::from(detected.install_path), path.join("Contents/Resources/game/game"));
    }
}
