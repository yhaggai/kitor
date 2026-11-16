use crate::models::{game, installed_patch};
use crate::services::{
    app_updates::AppUpdates,
    game_behavior, game_service,
    patch_lifecycle::{backups, safe_path},
    patch_service::PATCH_OPERATION,
    runtime_cleanup::remove_owned_tree,
};
use crate::AppState;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use std::path::{Path, PathBuf};
use tauri::State;

pub(super) static GAME_INSTALL_OPERATION: tokio::sync::RwLock<()> =
    tokio::sync::RwLock::const_new(());

async fn game_backups(
    state: &AppState,
    game: &game::Model,
    include_empty: bool,
) -> Result<Vec<backups::Entry>, String> {
    let root = Path::new(&game.install_path);
    let target = game_behavior::for_game(&game.scummvm_id)?.install_target(root)?;
    let target = target
        .ancestors()
        .find_map(|ancestor| {
            let resolved = ancestor.canonicalize().ok()?;
            Some(resolved.join(target.strip_prefix(ancestor).ok()?))
        })
        .unwrap_or(target);
    let legacy = installed_patch::Entity::find()
        .filter(installed_patch::Column::GameId.eq(game.id))
        .filter(installed_patch::Column::ScummvmId.eq(&game.scummvm_id))
        .all(&state.db)
        .await
        .map_err(|error| error.to_string())?;
    Ok(backups::list(&state.app_data_dir, false, include_empty)?
        .into_iter()
        .filter(|entry| {
            entry.summary.game == game.scummvm_id
                && match &entry.summary.target {
                    Some(path) => path == &target,
                    None => legacy.iter().any(|row| {
                        row.backup_path.as_deref().map(Path::new) == Some(entry.folder.as_path())
                    }),
                }
        })
        .collect())
}

async fn find_game(state: &AppState, id: i32) -> Result<game::Model, String> {
    game_service::get_game_by_id(&state.db, id)
        .await?
        .ok_or("Game not found".into())
}

#[tauri::command]
pub async fn list_game_backups(state: State<'_, AppState>) -> Result<Vec<backups::Backup>, String> {
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let registry = crate::services::registry_service::load_registry().ok();
    Ok(backups::list(&state.app_data_dir, true, false)?
        .into_iter()
        .map(|entry| {
            let mut backup = entry.summary;
            if let Some(game) = registry.as_ref().and_then(|registry| {
                registry
                    .games
                    .iter()
                    .find(|game| game.scummvm_id == backup.game)
            }) {
                backup.title.clone_from(&game.title);
                backup.title_he.clone_from(&game.title_he);
            }
            backup
        })
        .collect())
}

#[tauri::command]
pub async fn open_backup_folder(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let entry = backups::list(&state.app_data_dir, false, false)?
        .into_iter()
        .find(|entry| entry.summary.id == backup_id)
        .ok_or("Backup folder not found")?;
    open_folder(entry.folder)
}

fn open_folder(path: PathBuf) -> Result<(), String> {
    tauri_plugin_opener::open_path(path.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_game_backup(
    backup_id: String,
    confirmed: bool,
    state: State<'_, AppState>,
    updates: State<'_, AppUpdates>,
) -> Result<(), String> {
    let _activity = updates.activity()?;
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let Some(entry) = backups::list(&state.app_data_dir, false, false)?
        .into_iter()
        .find(|entry| entry.summary.id == backup_id)
    else {
        return Ok(());
    };
    backups::delete(entry, confirmed)
}

#[derive(serde::Serialize)]
pub struct GameStorageOptions {
    has_backup: bool,
    uninstall_path: Option<PathBuf>,
}

#[tauri::command]
pub async fn game_storage_options(
    game_id: i32,
    state: State<'_, AppState>,
) -> Result<GameStorageOptions, String> {
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let game = find_game(&state, game_id).await?;
    Ok(GameStorageOptions {
        has_backup: !game_backups(&state, &game, false).await?.is_empty(),
        uninstall_path: managed_installation(&state, &game).await?,
    })
}

#[tauri::command]
pub async fn open_game_backup_folder(
    game_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let game = find_game(&state, game_id).await?;
    let entries = game_backups(&state, &game, false).await?;
    let entry = entries
        .into_iter()
        .max_by_key(|entry| {
            std::fs::metadata(&entry.folder)
                .and_then(|meta| meta.modified())
                .ok()
        })
        .ok_or("Backup folder not found")?;
    open_folder(entry.folder)
}

// Recursive deletion is limited to a dedicated installation created in Kitor's
// managed games directory. Scanned folders may contain other games or user data.
async fn managed_installation(
    state: &AppState,
    game: &game::Model,
) -> Result<Option<PathBuf>, String> {
    let root = safe_path(&state.app_data_dir, "games")?;
    let path = Path::new(&game.install_path);
    let base = root.canonicalize().unwrap_or_else(|_| root.clone());
    let Ok(relative) = path
        .strip_prefix(&root)
        .or_else(|_| path.strip_prefix(&base))
    else {
        return Ok(None);
    };
    let Some(name) = relative
        .components()
        .next()
        .and_then(|part| part.as_os_str().to_str())
    else {
        return Ok(None);
    };
    if name != format!("{}-steam", game.scummvm_id) && name != format!("{}-gog", game.scummvm_id) {
        return Ok(None);
    }
    let folder = safe_path(&base, name)?;
    if path
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Ok(None);
    }
    let target = game_behavior::for_game(&game.scummvm_id)?.install_target(path)?;
    let canonical_folder = folder.canonicalize().unwrap_or_else(|_| folder.clone());
    if canonical_folder != folder {
        return Ok(None);
    }
    let canonical_target = target.canonicalize().unwrap_or(target);
    if !canonical_target.starts_with(&folder) && !canonical_target.starts_with(root.join(name)) {
        return Ok(None);
    }
    for other in game_service::get_all_games(&state.db).await? {
        let other_path = PathBuf::from(other.install_path);
        let other_path = other_path.canonicalize().unwrap_or_else(|_| {
            other_path
                .strip_prefix(&root)
                .map(|relative| base.join(relative))
                .unwrap_or(other_path)
        });
        if other.id != game.id
            && (other_path.starts_with(&folder) || folder.starts_with(&other_path))
        {
            return Ok(None);
        }
    }
    Ok(Some(folder))
}

#[tauri::command]
pub async fn uninstall_game(
    game_id: i32,
    expected_path: PathBuf,
    state: State<'_, AppState>,
    updates: State<'_, AppUpdates>,
) -> Result<(), String> {
    let _activity = updates.activity()?;
    let _install = GAME_INSTALL_OPERATION
        .try_write()
        .map_err(|_| "A game installation is running. Try again after it finishes.")?;
    let _operation = PATCH_OPERATION
        .try_lock()
        .map_err(|_| "A patch operation is running. Try again after it finishes.")?;
    let game = find_game(&state, game_id).await?;
    let folder = managed_installation(&state, &game)
        .await?
        .filter(|folder| folder == &expected_path)
        .ok_or("This game does not have a dedicated installation in Kitor's games folder")?;
    let entries = game_backups(&state, &game, true).await?;
    remove_owned_tree(&folder).map_err(|error| format!("Could not uninstall game: {error}"))?;
    for entry in entries {
        backups::retire_uninstalled(entry).map_err(|error| {
            format!(
                "Game files were removed. Retry uninstall to finish deleting its backup: {error}"
            )
        })?;
    }
    let transaction = state.db.begin().await.map_err(|error| error.to_string())?;
    installed_patch::Entity::delete_many()
        .filter(installed_patch::Column::GameId.eq(game_id))
        .filter(installed_patch::Column::ScummvmId.eq(&game.scummvm_id))
        .exec(&transaction)
        .await
        .map_err(|error| error.to_string())?;
    game::Entity::delete_by_id(game_id)
        .exec(&transaction)
        .await
        .map_err(|error| error.to_string())?;
    transaction
        .commit()
        .await
        .map_err(|error| error.to_string())
}
