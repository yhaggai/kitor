//! On-demand, read-only disk verification; recovery records are never discarded.
use super::{game_service, patch_lifecycle, patch_service};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let behavior = super::game_behavior::for_game(id)?;
    if !behavior.health_files.is_empty() {
        let mut missing = vec![];
        for name in &behavior.health_files {
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
    let detected = game_service::detect_game_for_id(root, id)?;
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
                let target = super::game_behavior::for_game(&game.scummvm_id)?.install_target(&root)?;
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
            let behavior = super::game_behavior::for_game(&game.scummvm_id)?;
            if let Some(manual) = inspect_manual(&behavior, &app_data, &root, &game.scummvm_id) {
                health.patch_status = manual.patch_status;
                health.details.extend(manual.details);
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

pub(super) fn inspect_manual(behavior: &super::game_behavior::GameBehavior, app_data: &Path, root: &Path, game_id: &str) -> Option<GameHealth> {
    let record = behavior.manual_patch_manifest.as_ref().map(|p| app_data.join(p)).filter(|p| p.is_file())?;
    let mut health = GameHealth {
        game_status: if root.is_dir() { "present" } else { "missing" }.into(),
        patch_status: "not_installed".into(), details: Vec::new(), checked_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(error) = verify_manual(&record, root, game_id, &mut health) {
        health.patch_status = "unverified".into(); health.details.push(error);
    }
    (health.patch_status != "not_installed").then_some(health)
}

fn verify_manual(record: &Path, root: &Path, game_id: &str, health: &mut GameHealth) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(record).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let matches = value["game_root"]
        .as_str()
        .and_then(|p| Path::new(p).canonicalize().ok())
        == Some(root.canonicalize().map_err(|e| e.to_string())?);
    if !matches || value["status"] != "installed" || value["patch"].as_str().and_then(|p| p.split_once('/')).is_none_or(|(id, _)| id != game_id) {
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
mod tests;
