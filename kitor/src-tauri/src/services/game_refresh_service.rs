//! On-demand, read-only disk verification; recovery records are never discarded.
use super::{game_service, patch_lifecycle, patch_service};
use crate::AppState;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize)]
pub struct GameHealth {
    pub game_status: String,
    pub patch_status: String,
    pub details: Vec<String>,
    pub checked_at: String,
}

fn check_game(root: &Path, id: &str) -> Result<(String, Vec<String>), String> {
    match std::fs::metadata(root) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(("missing".into(), vec!["Game folder is missing.".into()]))
        }
        Err(e) => return Err(format!("Cannot read game folder: {e}")),
        Ok(m) if !m.is_dir() => {
            return Ok((
                "missing".into(),
                vec!["Game path is no longer a folder.".into()],
            ))
        }
        _ => {}
    }
    if id == "monkey2-se" {
        let mut missing = vec![];
        for name in ["Monkey2.exe", "monkey2.pak", "audio/SFXOriginal.xwb"] {
            let path = patch_lifecycle::safe_path(root, name)?;
            match std::fs::metadata(path) {
                Ok(m) if m.is_file() && m.len() > 0 => {}
                Ok(_) => missing.push(format!("Game file empty or invalid: {name}")),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    missing.push(format!("Game file missing: {name}"))
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        return Ok((
            if missing.is_empty() {
                "present"
            } else {
                "missing"
            }
            .into(),
            missing,
        ));
    }
    let detected = game_service::detect_game_in_folder(root)?;
    if detected.is_some_and(|g| g.scummvm_id == id) {
        Ok(("present".into(), vec![]))
    } else {
        Ok(("unrecognized".into(), vec!["Folder exists, but the expected game signature was not found. A patch may have changed it.".into()]))
    }
}

pub async fn refresh(game_id: i32, state: &AppState) -> Result<GameHealth, String> {
    let _operation = patch_service::PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Refresh after it finishes.")?;
    let game = game_service::get_game_by_id(&state.db, game_id)
        .await?
        .ok_or("Game not found")?;
    let patches = patch_service::get_installed_patches(&state.db, game_id).await?;
    let app_data = state.app_data_dir.clone();
    tokio::task::spawn_blocking(move || {
        let root = PathBuf::from(&game.install_path);
        let (game_status, details) = check_game(&root, &game.scummvm_id)?;
        let mut health = GameHealth {
            game_status,
            details,
            patch_status: "not_installed".into(),
            checked_at: chrono::Utc::now().to_rfc3339(),
        };
        if !root.is_dir() {
            if !patches.is_empty() {
                health.patch_status = "unverified".into();
            }
            return Ok(health);
        }
        if !patches.is_empty() {
            health.patch_status = "installed".into();
            for patch in patches {
                let target = if cfg!(target_os = "macos") && game.scummvm_id == "drifter" {
                    super::drifter_patch::macos_target(
                        &std::env::home_dir().ok_or("Cannot locate home directory")?,
                    )?
                } else {
                    root.clone()
                };
                let result = patch
                    .backup_path
                    .as_ref()
                    .ok_or("No patch verification manifest is recorded".to_string())
                    .and_then(|p| {
                        patch_lifecycle::verify_installed(
                            Path::new(p),
                            &target,
                            &game.scummvm_id,
                            &patch.version,
                        )
                    });
                match result {
                    Ok(problems) if problems.is_empty() => {}
                    Ok(problems) => {
                        health.patch_status = "damaged".into();
                        health.details.extend(problems);
                    }
                    Err(error) => {
                        if health.patch_status != "damaged" {
                            health.patch_status = "unverified".into();
                        }
                        health.details.push(error);
                    }
                }
            }
        } else {
            // Also verify the manifest from the user's manual Monkey Island 2 installation.
            let record = app_data.join("manual-patches/monkey2-steam-he-1.1-kitor2/manifest.json");
            if game.scummvm_id == "monkey2-se" && record.is_file() {
                if let Err(e) = verify_manual(&record, &root, &mut health) {
                    health.patch_status = "unverified".into();
                    health.details.push(e);
                }
            }
            if health.patch_status == "not_installed"
                && patch_lifecycle::safe_path(&root, "hebrew")?.is_dir()
            {
                health.patch_status = "unverified".into();
                health.details.push(
                    "Hebrew files exist, but no matching installation manifest is recorded.".into(),
                );
            }
        }
        Ok(health)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn verify_manual(record: &Path, root: &Path, health: &mut GameHealth) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(record).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let matches = value["game_root"]
        .as_str()
        .and_then(|p| Path::new(p).canonicalize().ok())
        == Some(root.canonicalize().map_err(|e| e.to_string())?);
    if !matches || value["status"] != "installed" || value["patch"] != "monkey2-se/he" {
        return Ok(());
    }
    let files = value["files"]
        .as_array()
        .filter(|f| !f.is_empty())
        .ok_or("Manual patch manifest has no files")?;
    health.patch_status = "installed".into();
    for file in files {
        let name = file["path"].as_str().ok_or("Invalid manual patch path")?;
        let expected = file["sha256"].as_str().ok_or("Missing manual patch hash")?;
        let path = patch_lifecycle::safe_path(root, name)?;
        if !path.is_file() || patch_lifecycle::hash(&path)? != expected {
            health.patch_status = "damaged".into();
            health
                .details
                .push(format!("Patch file missing or changed: {name}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_missing_and_incomplete_game() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("game");
        assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "missing");
        std::fs::create_dir_all(root.join("audio")).unwrap();
        for name in ["Monkey2.exe", "monkey2.pak", "audio/SFXOriginal.xwb"] {
            std::fs::write(root.join(name), b"data").unwrap();
        }
        assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "present");
        std::fs::remove_file(root.join("monkey2.pak")).unwrap();
        assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "missing");
    }
    #[test]
    fn manual_patch_detects_changed_files() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("game");
        std::fs::create_dir_all(root.join("hebrew")).unwrap();
        let file = root.join("hebrew/MONKEY2.000");
        std::fs::write(&file, b"patched").unwrap();
        let record = t.path().join("manifest.json");
        let manifest = serde_json::json!({"game_root":root,"patch":"monkey2-se/he","status":"installed","files":[{"path":"hebrew/MONKEY2.000","sha256":patch_lifecycle::hash(&file).unwrap()}]});
        std::fs::write(&record, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let mut health = GameHealth {
            game_status: "present".into(),
            patch_status: "not_installed".into(),
            details: vec![],
            checked_at: String::new(),
        };
        verify_manual(&record, &root, &mut health).unwrap();
        assert_eq!(health.patch_status, "installed");
        std::fs::remove_file(&file).unwrap();
        verify_manual(&record, &root, &mut health).unwrap();
        assert_eq!(health.patch_status, "damaged");
        assert!(!file.exists());
    }

    #[test]
    #[ignore = "Read-only check of this machine's downloaded and manually patched game"]
    fn live_monkey2_refresh() {
        let home = std::env::home_dir().unwrap();
        let root=home.join("Library/Application Support/Steam/Steam.AppBundle/Steam/Contents/MacOS/steamapps/content/app_32460/depot_32461");
        assert_eq!(check_game(&root, "monkey2-se").unwrap().0, "present");
        let record=home.join("Library/Application Support/haggai/manual-patches/monkey2-steam-he-1.1-kitor2/manifest.json");
        let mut health = GameHealth {
            game_status: "present".into(),
            patch_status: "not_installed".into(),
            details: vec![],
            checked_at: String::new(),
        };
        verify_manual(&record, &root, &mut health).unwrap();
        assert_eq!(health.patch_status, "installed");
    }

    #[test]
    fn verifies_patch_bytes_without_changing_them() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("game");
        let backup = t.path().join("backup");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&backup).unwrap();
        let file = root.join("patched");
        std::fs::write(&file, b"patched").unwrap();
        let manifest = serde_json::json!({"schema_version":1,"target":root.canonicalize().unwrap(),"game":"test","language":"he","version":"1","status":"active","created_dirs":[],"files":[{"path":"patched","original_hash":null,"installed_hash":patch_lifecycle::hash(&file).unwrap(),"started":true,"restored":false},{"path":"removed","original_hash":"old","installed_hash":null,"started":true,"restored":false}]});
        std::fs::write(
            backup.join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            patch_lifecycle::verify_installed(&backup, &root, "test", "1")
                .unwrap()
                .is_empty()
        );
        std::fs::write(&file, b"changed").unwrap();
        assert_eq!(
            patch_lifecycle::verify_installed(&backup, &root, "test", "1")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(std::fs::read(&file).unwrap(), b"changed");
        std::fs::remove_file(&file).unwrap();
        assert!(
            patch_lifecycle::verify_installed(&backup, &root, "test", "1").unwrap()[0]
                .contains("missing")
        );
        assert!(patch_lifecycle::verify_installed(&backup, &root, "other", "1").is_err());
        assert!(patch_lifecycle::verify_installed(t.path(), &root, "test", "1").is_err());
    }
}
