//! Signed public app updates, coordinated with game and runtime installation.
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{ipc::Channel, AppHandle};
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::sync::{Mutex, OwnedRwLockWriteGuard, RwLock, RwLockReadGuard};

#[derive(Default)]
pub struct AppUpdates {
    operation: Mutex<()>,
    pending: Mutex<Option<Update>>,
    work: Arc<RwLock<()>>,
    installed: AtomicBool,
    restart_guard: Mutex<Option<OwnedRwLockWriteGuard<()>>>,
}
impl AppUpdates {
    pub fn activity(&self) -> Result<RwLockReadGuard<'_, ()>, String> {
        self.work
            .try_read()
            .map_err(|_| "Kitor is updating. Wait for it to restart.".into())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    current_version: String,
    can_install: bool,
    update: Option<UpdateInfo>,
}
#[derive(Serialize)]
pub struct UpdateInfo {
    version: String,
    notes: String,
}
#[derive(Clone, Serialize)]
#[serde(tag = "event", rename_all = "camelCase")]
pub enum Progress {
    Downloading { downloaded: u64, total: Option<u64> },
    Verifying,
    Installing,
}

pub async fn check(app: &AppHandle, state: &AppUpdates) -> Result<CheckResult, String> {
    let _operation = state
        .operation
        .try_lock()
        .map_err(|_| "An app update operation is already running.")?;
    if state.installed.load(Ordering::Acquire) {
        return Err("Restart Kitor to finish updating.".into());
    }
    *state.pending.lock().await = None;
    let current = app.package_info().version.clone();
    let mut result = CheckResult {
        current_version: current.to_string(),
        can_install: !cfg!(debug_assertions),
        update: None,
    };
    let updater = app
        .updater_builder()
        .timeout(Duration::from_secs(30))
        .configure_client(|client| {
            client
                .connect_timeout(Duration::from_secs(20))
                .read_timeout(Duration::from_secs(60))
        })
        .build()
        .map_err(|_| "Could not initialize app updates.")?;
    let Some(update) = updater.check().await.map_err(|_| {
        "Could not check for app updates. Check your connection and try again later."
    })?
    else {
        return Ok(result);
    };
    result.update = Some(UpdateInfo {
        version: update.version.clone(),
        notes: update.body.clone().unwrap_or_default(),
    });
    *state.pending.lock().await = Some(update);
    Ok(result)
}

pub async fn install(
    state: &AppUpdates,
    version: &str,
    progress: Channel<Progress>,
) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Err("App updates can only be installed by a release build of Kitor.".into());
    }
    let _operation = state
        .operation
        .try_lock()
        .map_err(|_| "An app update operation is already running.")?;
    if state.installed.load(Ordering::Acquire) {
        return Err("Restart Kitor to finish updating.".into());
    }
    let guard = state.work.clone().try_write_owned().map_err(|_| {
        "Finish active game downloads, patch installs, or restores before updating Kitor."
    })?;
    let mut pending = state.pending.lock().await;
    let update = pending
        .as_mut()
        .ok_or("Check for updates again before installing.")?;
    if update.version != version {
        return Err("The selected update changed. Check for updates again.".into());
    }
    let mut downloaded = 0;
    let bytes = update
        .download(
            |length, total| {
                downloaded += length as u64;
                let _ = progress.send(Progress::Downloading { downloaded, total });
            },
            || {
                let _ = progress.send(Progress::Verifying);
            },
        )
        .await;
    let bytes = bytes.map_err(|_| "The app update could not be downloaded or its signature could not be verified. Check your connection and try again.")?;
    let _ = progress.send(Progress::Installing);
    update.install(bytes).map_err(|_| "Kitor could not install the update. On macOS, move Kitor to Applications and eject its installer disk image before trying again.")?;
    state.installed.store(true, Ordering::Release);
    *state.restart_guard.lock().await = Some(guard);
    *pending = None;
    Ok(())
}

pub fn restart(app: &AppHandle, state: &AppUpdates) -> Result<(), String> {
    if !state.installed.load(Ordering::Acquire) {
        return Err("No installed app update is waiting to restart.".into());
    }
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn updates_and_game_mutations_exclude_each_other() {
        let state = AppUpdates::default();
        let download = state.activity().unwrap();
        assert!(state.work.clone().try_write_owned().is_err());
        drop(download);
        let update = state.work.clone().try_write_owned().unwrap();
        assert!(state.activity().is_err());
        drop(update);
        assert!(state.activity().is_ok());
    }
}
