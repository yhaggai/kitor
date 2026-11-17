//! Cancellation shared by the commands in one installer dialog run.
use std::{future::Future, sync::Mutex};
use tokio::sync::watch;

const CANCELLED: &str = "Installation cancelled";
static ACTIVE: Mutex<Option<(String, watch::Sender<bool>)>> = Mutex::new(None);

tokio::task_local! {
    static CANCELLATION: watch::Receiver<bool>;
}

#[tauri::command]
pub fn begin_game_install(run_id: String) -> Result<(), String> {
    let mut active = ACTIVE.lock().unwrap();
    if active.is_some() {
        return Err("Another game installation is running".into());
    }
    *active = Some((run_id, watch::channel(false).0));
    Ok(())
}

#[tauri::command]
pub fn cancel_game_install(run_id: String) {
    if let Some((id, signal)) = ACTIVE.lock().unwrap().as_ref() {
        if *id == run_id {
            signal.send_replace(true);
        }
    }
}

#[tauri::command]
pub fn end_game_install(run_id: String) {
    let mut active = ACTIVE.lock().unwrap();
    if active.as_ref().is_some_and(|(id, _)| *id == run_id) {
        *active = None;
    }
}

pub async fn scope<T>(
    run_id: Option<String>,
    operation: impl Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let Some(run_id) = run_id else {
        return operation.await;
    };
    let signal = ACTIVE
        .lock()
        .unwrap()
        .as_ref()
        .filter(|(id, _)| *id == run_id)
        .map(|(_, signal)| signal.subscribe())
        .ok_or(CANCELLED)?;
    CANCELLATION
        .scope(signal, async {
            checkpoint()?;
            operation.await
        })
        .await
}

pub fn checkpoint() -> Result<(), String> {
    if CANCELLATION
        .try_with(|signal| *signal.borrow())
        .unwrap_or(false)
    {
        Err(CANCELLED.into())
    } else {
        Ok(())
    }
}

// Only race work that can safely be dropped. File publication, database writes,
// and patch apply/rollback must run to a consistent state.
pub async fn interruptible<T>(
    operation: impl Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let Ok(mut signal) = CANCELLATION.try_with(Clone::clone) else {
        return operation.await;
    };
    tokio::select! {
        biased;
        _ = signal.wait_for(|cancelled| *cancelled) => Err(CANCELLED.into()),
        result = operation => result,
    }
}
