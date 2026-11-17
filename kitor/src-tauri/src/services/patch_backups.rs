//! Inventory is independent of the library: removing a game retains recovery data.
use super::super::runtime_cleanup::remove_owned_tree;
use super::*;

#[derive(Serialize)]
pub struct Backup {
    pub id: String,
    pub game: String,
    pub title: String,
    pub title_he: Option<String>,
    pub target: Option<PathBuf>,
    pub version: Option<String>,
    pub bytes: u64,
    pub needs_confirmation: bool,
    pub can_delete: bool,
}

pub struct Entry {
    pub summary: Backup,
    pub folder: PathBuf,
    transaction: Option<PathBuf>,
    detached: bool,
}

const DISCARDED: &str = "discarded-originals";

fn children(path: &Path) -> Result<Vec<PathBuf>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(err(error)),
    };
    let mut dirs = Vec::new();
    for entry in entries {
        let entry = entry.map_err(err)?;
        if entry.file_type().map_err(err)?.is_dir() {
            dirs.push(entry.path());
        }
    }
    Ok(dirs)
}

fn size(path: &Path) -> Result<u64> {
    let mut bytes = 0u64;
    for entry in fs::read_dir(path).map_err(err)? {
        let entry = entry.map_err(err)?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(err)?;
        if metadata.is_dir() {
            bytes = bytes.saturating_add(size(&entry.path())?);
        } else if metadata.is_file() {
            bytes = bytes.saturating_add(metadata.len());
        }
    }
    Ok(bytes)
}

fn originals(dir: &Path, journal: &Journal) -> Result<PathBuf> {
    if journal.originals_dir != "originals"
        && !(journal.originals_dir.starts_with("originals-update-")
            && !journal.originals_dir.contains('/'))
    {
        return Err("Invalid backup folder in patch manifest".into());
    }
    safe_path(dir, &journal.originals_dir)
}

pub fn list(app_data: &Path, measure: bool, include_empty: bool) -> Result<Vec<Entry>> {
    let mut backups = Vec::new();
    for kind in ["patch-transactions", "backups"] {
        let root = safe_path(app_data, kind)?;
        for group in children(&root)? {
            for dir in children(&group)? {
                let id = format!(
                    "{kind}/{}/{}",
                    group.file_name().unwrap().to_string_lossy(),
                    dir.file_name().unwrap().to_string_lossy()
                );
                if kind == "backups" {
                    backups.push(Entry {
                        summary: Backup {
                            id,
                            game: group.file_name().unwrap().to_string_lossy().into_owned(),
                            title: group.file_name().unwrap().to_string_lossy().into_owned(),
                            title_he: None,
                            target: None,
                            version: None,
                            bytes: if measure { size(&dir)? } else { 0 },
                            needs_confirmation: true,
                            can_delete: true,
                        },
                        folder: dir,
                        transaction: None,
                        detached: false,
                    });
                    continue;
                }

                let Ok(journal) = load(&dir) else { continue };
                let detached_folder = safe_path(&dir, DISCARDED)?;
                let detached = detached_folder.is_dir();
                let folder = if detached {
                    detached_folder
                } else {
                    originals(&dir, &journal)?
                };
                let has_payload = folder.is_dir()
                    && (detached || fs::read_dir(&folder).map_err(err)?.next().is_some());
                if !include_empty && !has_payload {
                    continue;
                }
                backups.push(Entry {
                    summary: Backup {
                        id,
                        title: journal.game.clone(),
                        title_he: None,
                        game: journal.game,
                        needs_confirmation: !detached
                            && journal.status != "superseded"
                            && !matches!(journal.target.try_exists(), Ok(false)),
                        can_delete: detached
                            || (!safe_path(&dir, "pending-update")?.exists()
                                && (matches!(journal.status.as_str(), "active" | "superseded")
                                    || matches!(journal.target.try_exists(), Ok(false)))),
                        target: Some(journal.target),
                        version: Some(journal.version),
                        bytes: if measure && has_payload {
                            size(&folder)?
                        } else {
                            0
                        },
                    },
                    folder,
                    transaction: Some(dir),
                    detached,
                });
            }
        }
    }
    backups.sort_by(|a, b| {
        b.summary
            .bytes
            .cmp(&a.summary.bytes)
            .then(a.summary.id.cmp(&b.summary.id))
    });
    Ok(backups)
}

pub fn delete(entry: Entry, confirmed: bool) -> Result<()> {
    if entry.summary.needs_confirmation && !confirmed {
        return Err(
            "This backup may still be needed. Review the confirmation and try again.".into(),
        );
    }
    let Some(dir) = entry.transaction else {
        return remove_owned_tree(&entry.folder);
    };
    let _lock = lock(dir.parent().ok_or("Missing transaction parent")?)?;
    let detached = safe_path(&dir, DISCARDED)?;
    if entry.detached {
        return remove_owned_tree(&detached);
    }
    let journal = load(&dir)?;
    if safe_path(&dir, "pending-update")?.exists() {
        return Err("Finish the interrupted patch update before deleting this backup".into());
    }
    let removed = matches!(journal.target.try_exists(), Ok(false));
    if !removed {
        if !matches!(journal.status.as_str(), "active" | "superseded") {
            return Err("Finish patch recovery before deleting this backup".into());
        }
        if journal.status == "active" && !confirmed {
            return Err(
                "The game is still installed. Review the confirmation and try again.".into(),
            );
        }
    }
    let folder = originals(&dir, &journal)?;
    // Keep the installation manifest for verification and patch ownership.
    // Rename first so an interrupted deletion cannot look like a usable backup.
    fs::rename(folder, &detached).map_err(err)?;
    sync_dir(&dir)?;
    remove_owned_tree(&detached)
}

pub fn retire_uninstalled(entry: Entry) -> Result<()> {
    let Some(dir) = entry.transaction else {
        return delete(entry, true);
    };
    let _lock = lock(dir.parent().ok_or("Missing transaction parent")?)?;
    let mut journal = load(&dir)?;
    if !matches!(journal.target.try_exists(), Ok(false)) {
        return Err("The translation target still exists".into());
    }
    journal.status = "superseded".into();
    save(&dir, &journal)?;
    for path in [
        safe_path(&dir, "pending-update")?,
        safe_path(&dir, "outputs")?,
        safe_path(&dir, DISCARDED)?,
        originals(&dir, &journal)?,
    ] {
        remove_owned_tree(&path)?;
    }
    Ok(())
}

pub(super) fn require_originals(dir: &Path, journal: &Journal) -> Result<()> {
    if matches!(journal.status.as_str(), "active" | "restoring")
        && journal
            .files
            .iter()
            .any(|record| record.original_hash.is_some())
        && !originals(dir, journal)?.is_dir()
    {
        return Err("Missing original backup. Repair or reinstall the game to restore or update its translation.".into());
    }
    Ok(())
}
