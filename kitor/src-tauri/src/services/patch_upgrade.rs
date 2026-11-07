//! A nested rollback journal covers all old/new paths. Atomically replacing the
//! active manifest commits the update; before that, recovery restores old bytes.
use super::*;

pub fn recover_installation(dir: &Path) -> Result<(String, String, String, u64)> {
    let _lock = lock(dir.parent().ok_or("Missing transaction parent")?)?;
    recover(dir)?;
    let mut journal = load(dir)?;
    if journal.status == "restoring" {
        restore_inner(dir, &mut journal)?;
        cleanup_restored(dir, &journal)?;
    }
    Ok((
        journal.version,
        journal.language,
        journal.status,
        journal.release_revision,
    ))
}

pub(super) fn recover(dir: &Path) -> Result<()> {
    let pending = safe_path(dir, "pending-update")?;
    if !pending.exists() {
        cleanup_staging(dir)?;
        return cleanup_restored(dir, &load(dir)?);
    }
    let active = load(dir)?;
    let next: Journal =
        serde_json::from_slice(&fs::read(pending.join("next.json")).map_err(err)?).map_err(err)?;
    let mut rollback = load(&pending)?;
    if active.target != rollback.target
        || active.target != next.target
        || active.game != next.game
        || active.language != next.language
    {
        return Err("Update recovery identity mismatch".into());
    }
    if active.version != next.version || active.originals_dir != next.originals_dir {
        restore_inner(&pending, &mut rollback)?;
        // Discard originals introduced by an uncommitted update, never the
        // original backups still referenced by the active installation.
        let retained = active.files.iter().enumerate()
            .map(|(index, _)| original_file(dir, &active, index))
            .collect::<Result<Vec<_>>>()?;
        for (index, record) in next.files.iter().enumerate() {
            if record.original_hash.is_some() {
                let path = original_file(dir, &next, index)?;
                if !retained.contains(&path) && path.exists() {
                    fs::remove_file(path).map_err(err)?;
                }
            }
        }
    }
    // Detach cleanup atomically: a crash partway through recursive deletion
    // must not leave a half-deleted directory that looks like pending recovery.
    // Retired folders are already detached recovery transactions. Clear any
    // interrupted deletion before reusing this name.
    cleanup_staging(dir)?;
    let retired = safe_path(dir, &format!("retired-{}", next.originals_dir))?;
    fs::rename(&pending, &retired).map_err(err)?;
    sync_dir(dir)?;
    fs::remove_dir_all(retired).map_err(err)?;
    cleanup_restored(dir, &active)
}

// Call only while holding the installation lock. These directories are never
// authoritative: pending-update contains all durable rollback data.
fn cleanup_staging(dir: &Path) -> Result<()> {
    let journal = load(dir)?;
    let originals = safe_path(dir, &journal.originals_dir)?;
    for entry in fs::read_dir(dir).map_err(err)? {
        let entry = entry.map_err(err)?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if ["stage-update-", "prepare-update-", "retired-"]
            .iter().any(|prefix| name.starts_with(prefix)) {
            let path = safe_path(dir, &name)?;
            if path == originals || originals.starts_with(&path) {
                return Err("Staging cleanup overlaps retained originals".into());
            }
            if entry.file_type().map_err(err)?.is_dir() {
                fs::remove_dir_all(path).map_err(err)?;
            }
        }
    }
    Ok(())
}

/// Stage directly from verified original backups, then apply one recoverable change
/// set. No whole-game copies, unrestricted installer scripts, or live uninstall.
#[cfg(any(test, feature = "standalone-patch-tools"))]
pub fn update(
    dir: &Path,
    package_dir: &Path,
    manifest_name: &str,
    target: &Path,
    game: &str,
    language: &str,
    version: &str,
    platform: &str,
) -> Result<PathBuf> {
    update_with_progress(dir, package_dir, manifest_name, target, game, language, version, platform, &|_, _| {})
}

pub fn update_with_progress(
    dir: &Path,
    package_dir: &Path,
    manifest_name: &str,
    target: &Path,
    game: &str,
    language: &str,
    version: &str,
    platform: &str,
    report: &dyn Fn(u8, &str),
) -> Result<PathBuf> {
    update_inner(
        dir,
        package_dir,
        manifest_name,
        target,
        game,
        language,
        version,
        platform,
        report,
        &|_, _| Ok(()),
    )
}

fn update_inner(
    dir: &Path,
    package_dir: &Path,
    manifest_name: &str,
    target: &Path,
    game: &str,
    language: &str,
    version: &str,
    platform: &str,
    report: &dyn Fn(u8, &str),
    checkpoint: &dyn Fn(&Path, bool) -> Result<()>,
) -> Result<PathBuf> {
    let _lock = lock(dir.parent().ok_or("Missing transaction parent")?)?;
    recover(dir)?;
    let old = load(dir)?;
    let target = target.canonicalize().map_err(err)?;
    if old.target != target
        || old.game != game
        || old.language != language
        || old.status != "active"
    {
        return Err("Update does not match the active translation installation".into());
    }
    for (index, record) in old.files.iter().enumerate() {
        report(55, &format!("Verifying recovery files ({} of {}): {}", index + 1, old.files.len(), record.path));
        if !record.started
            || record.restored
            || current_hash(&safe_path(&target, &record.path)?)? != record.installed_hash
        {
            return Err(format!(
                "Installed translation changed; resolve before updating: {}",
                record.path
            ));
        }
        if let Some(expected) = &record.original_hash {
            check_hash(&original_file(dir, &old, index)?, expected)
                .map_err(|e| format!("Missing or invalid original backup: {e}"))?;
        }
    }
    let package: Package =
        serde_json::from_slice(&fs::read(safe_path(package_dir, manifest_name)?).map_err(err)?)
            .map_err(err)?;
    if package.schema_version != 1
        || package.scummvm_id != game
        || package.language != language
        || package.version != version
    {
        return Err("Patch manifest identity/version does not match the catalog".into());
    }
    if old.version == version && old.release_revision == package.release_revision {
        return Ok(dir.to_path_buf());
    }
    if old.version == version || package.release_revision <= old.release_revision {
        return Err(
            "Translation updates require a new version and a higher release_revision".into(),
        );
    }
    let plan = package
        .platforms
        .get(platform)
        .ok_or("Patch has no file plan for this platform")?;
    let files = if plan.files.is_empty() {
        &package.files
    } else {
        &plan.files
    };
    let mut affected = BTreeMap::new();
    for path in old
        .files
        .iter()
        .map(|r| &r.path)
        .chain(files.iter().map(|r| &r.path))
    {
        safe_path(&target, path)?;
        affected
            .entry(path.to_lowercase())
            .or_insert_with(|| path.clone());
    }
    for path in affected.keys() {
        if affected
            .keys()
            .any(|other| other.starts_with(&format!("{path}/")))
        {
            return Err("Update contains overlapping file/directory paths".into());
        }
    }
    for entry in fs::read_dir(dir.parent().unwrap()).map_err(err)? {
        let other_dir = entry.map_err(err)?.path();
        if other_dir == dir || !other_dir.join("manifest.json").is_file() {
            continue;
        }
        let other = load(&other_dir)?;
        if other.status != "restored"
            && other.files.iter().any(|r| {
                affected.keys().any(|p| {
                    let owned = r.path.to_lowercase();
                    p == &owned
                        || p.starts_with(&format!("{owned}/"))
                        || owned.starts_with(&format!("{p}/"))
                })
            })
        {
            return Err("Another patch owns files required by this update".into());
        }
    }
    let work = tempfile::Builder::new()
        .prefix("stage-update-")
        .tempdir_in(dir)
        .map_err(err)?;
    let outputs = work.path().join("outputs");
    fs::create_dir(&outputs).map_err(err)?;
    // Retained backups were verified above. Their recorded hashes are reusable
    // for package compatibility checks; no temporary original copies are needed.
    let original_path = |relative: &str| -> Result<PathBuf> {
        safe_path(&target, relative)?;
        if let Some((index, record)) = old.files.iter().enumerate()
            .find(|(_, r)| r.path.eq_ignore_ascii_case(relative)) {
            if record.original_hash.is_none() {
                return Err(format!("Required original is missing: {relative}"));
            }
            original_file(dir, &old, index)
        } else {
            safe_path(&target, relative)
        }
    };
    let mut next = Journal {
        release_revision: package.release_revision,
        schema_version: 1,
        target: target.clone(),
        game: game.into(),
        language: language.into(),
        version: version.into(),
        status: "active".into(),
        files: vec![],
        created_dirs: vec![],
        originals_dir: old.originals_dir.clone(),
    };
    if files.is_empty() {
        return Err("Patch file plan is empty".into());
    }
    let mut paths = HashSet::new();
    for (index, change) in files.iter().enumerate() {
        if !paths.insert(change.path.to_lowercase()) {
            return Err("Duplicate patch target path".into());
        }
        report(60, &format!("Preparing updated file ({} of {}): {}", index + 1, files.len(), change.path));
        let previous = old.files.iter().find(|r| r.path.eq_ignore_ascii_case(&change.path));
        let original = match previous {
            Some(record) => record.original_hash.clone(),
            None => current_hash(&safe_path(&target, &change.path)?)?,
        };
        match change.operation {
            Operation::Create if original.is_some() =>
                return Err(format!("Creation conflicts with existing file: {}", change.path)),
            Operation::Modify | Operation::Delete if original.is_none() =>
                return Err(format!("Required original is missing: {}", change.path)),
            _ => (),
        }
        if let Some(expected) = &change.original_sha256 {
            if original.as_ref() != Some(expected) {
                return Err(format!("Unsupported original version: {}", change.path));
            }
        }
        if let Some(expected) = &change.original_md5 {
            check_md5(&original_path(&change.path)?, expected)?;
        }
        let output = outputs.join(index.to_string());
        let installed_hash = if change.operation == Operation::Delete {
            if change.source.is_some() {
                return Err("Delete operations cannot have an output source".into());
            }
            None
        } else {
            stage_source_with_input(
                change.source.as_ref().ok_or("Missing patch output source")?,
                package_dir, &output, &original_path,
            )?;
            let checksum = hash(&output)?;
            if let Some(expected) = &change.installed_sha256 {
                if &checksum != expected {
                    return Err(format!("Patched output checksum mismatch: {}", change.path));
                }
            }
            if let Some(expected) = &change.installed_md5 {
                check_md5(&output, expected)?;
            }
            if original.is_some() {
                fs::set_permissions(&output,
                    fs::metadata(original_path(&change.path)?).map_err(err)?.permissions()).map_err(err)?;
            }
            Some(checksum)
        };
        next.files.push(Record {
            original_backup: None,
            path: change.path.clone(),
            original_hash: original,
            installed_hash,
            started: true,
            restored: false,
        });
    }
    // Keep legacy numeric backup references stable when the new plan reorders files.
    // Newly touched originals are published only after the recovery journal exists.
    let mut additions = Vec::new();
    for (index, record) in next.files.iter_mut().enumerate() {
        if record.original_hash.is_some() {
            if let Some((old_index, previous)) = old.files.iter().enumerate()
                .find(|(_, previous)| previous.path.eq_ignore_ascii_case(&record.path)) {
                record.original_backup = Some(previous.original_backup.clone()
                    .unwrap_or_else(|| old_index.to_string()));
            } else {
                let name = format!("update-{}-{index}", package.release_revision);
                additions.push((index, name.clone()));
                record.original_backup = Some(name);
            }
        }
    }
    next.originals_dir = old.originals_dir.clone();
    next.created_dirs = old.created_dirs.clone();
    let pending_temp = tempfile::Builder::new()
        .prefix("prepare-update-")
        .tempdir_in(dir)
        .map_err(err)?;
    let pending = pending_temp.path();
    fs::create_dir(pending.join("originals")).map_err(err)?;
    fs::create_dir(pending.join("outputs")).map_err(err)?;
    let mut rollback = Journal {
        release_revision: old.release_revision,
        schema_version: 1,
        target: target.clone(),
        game: game.into(),
        language: language.into(),
        version: old.version.clone(),
        status: "prepared".into(),
        files: vec![],
        created_dirs: vec![],
        originals_dir: default_originals_dir(),
    };
    for (index, relative) in affected.values().enumerate() {
        report(68, &format!("Preparing recovery copy ({} of {}): {relative}", index + 1, affected.len()));
        let live = safe_path(&target, relative)?;
        let original = current_hash(&live)?;
        if let Some(record) = old
            .files
            .iter()
            .find(|r| r.path.eq_ignore_ascii_case(relative))
        {
            if original != record.installed_hash {
                return Err("Game changed while staging update".into());
            }
        } else if let Some(record) = next
            .files
            .iter()
            .find(|r| r.path.eq_ignore_ascii_case(relative))
        {
            if original != record.original_hash {
                return Err("Game changed while staging update".into());
            }
        }
        if let Some(expected) = &original {
            report(68, &format!("Creating backup (file {} of {}): {relative}", index + 1, affected.len()));
            let backup = pending.join("originals").join(index.to_string());
            copy_durable(&live, &backup)?;
            check_hash(&backup, expected)?;
        }
        report(68, &format!("Preparing updated file ({} of {}): {relative}", index + 1, affected.len()));
        let (desired, installed_hash) = if let Some((index, record)) = next.files.iter()
            .enumerate().find(|(_, r)| r.path.eq_ignore_ascii_case(relative)) {
            (outputs.join(index.to_string()), record.installed_hash.clone())
        } else {
            // A path dropped by the new patch returns to its original state.
            let (index, record) = old.files.iter().enumerate()
                .find(|(_, r)| r.path.eq_ignore_ascii_case(relative)).unwrap();
            (original_file(dir, &old, index)?, record.original_hash.clone())
        };
        if let Some(expected) = &installed_hash {
            let output = pending.join("outputs").join(index.to_string());
            copy_durable(&desired, &output)?;
            check_hash(&output, expected)?;
        }
        let mut ancestor = live.parent();
        while let Some(folder) = ancestor {
            if folder == target {
                break;
            }
            if !folder.exists() {
                let rel = folder
                    .strip_prefix(&target)
                    .map_err(err)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if !rollback.created_dirs.contains(&rel) {
                    rollback.created_dirs.push(rel.clone());
                }
                if !next.created_dirs.contains(&rel) {
                    next.created_dirs.push(rel);
                }
            }
            ancestor = folder.parent();
        }
        rollback.files.push(Record {
            original_backup: None,
            path: relative.clone(),
            original_hash: original,
            installed_hash,
            started: false,
            restored: false,
        });
    }
    write_json(&pending.join("next.json"), &next)?;
    save(pending, &rollback)?;
    let prepared = pending_temp.keep();
    let pending = dir.join("pending-update");
    fs::rename(prepared, &pending).map_err(err)?;
    sync_dir(dir)?;
    let result = (|| {
        for (index, name) in &additions {
            let destination = safe_path(dir, &format!("{}/{name}", next.originals_dir))?;
            // Newly affected originals are already in the durable rollback
            // snapshot. Publish from that snapshot, never from mutable live files.
            let rollback_index = rollback.files.iter().position(|r|
                r.path.eq_ignore_ascii_case(&next.files[*index].path)).unwrap();
            copy_durable(&pending.join("originals").join(rollback_index.to_string()), &destination)?;
            check_hash(&destination, next.files[*index].original_hash.as_ref().unwrap())?;
        }
        Ok(())
    })().and_then(|_| checkpoint(&pending, false))
        .and_then(|_| {
            report(70, "Applying prepared update files…");
            apply(&pending, &mut rollback)
        })
        .and_then(|_| checkpoint(&pending, true))
        .and_then(|_| {
            for record in &rollback.files {
                if current_hash(&safe_path(&target, &record.path)?)? != record.installed_hash {
                    return Err(format!(
                        "Game changed before update commit: {}",
                        record.path
                    ));
                }
            }
            Ok(())
        })
        .and_then(|_| save(dir, &next));
    if let Err(failure) = result {
        return match recover(dir) {
            Ok(()) => Err(format!(
                "{failure}. Update recovered; check installed version before retrying."
            )),
            Err(recovery) => Err(format!(
                "{failure}. Recovery required at {}: {recovery}",
                pending.display()
            )),
        };
    }
    let _ = recover(dir);
    Ok(dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Fixture {
        _root: tempfile::TempDir,
        game: PathBuf,
        package: PathBuf,
        dir: PathBuf,
    }
    impl Fixture {
        fn new(platform: &str) -> Self {
            let root = tempfile::tempdir().unwrap();
            let game = root.path().join("game");
            let package = root.path().join("package");
            fs::create_dir(&game).unwrap();
            fs::create_dir(&package).unwrap();
            fs::write(game.join("data"), b"original").unwrap();
            fs::write(game.join("obsolete-original"), b"restore me").unwrap();
            fs::write(game.join("save"), b"save game").unwrap();
            let mut f = Self {
                _root: root,
                game,
                package,
                dir: PathBuf::new(),
            };
            f.plan(
                "legacy-label",
                0,
                json!([
                    f.copy("data", "modify", b"old translation"),
                    f.copy("obsolete-original", "modify", b"old replacement"),
                    f.copy("old/new", "create", b"old created")
                ]),
            );
            f.dir = install(
                &f.package,
                "patch.json",
                &f.game,
                &f._root.path().join("backups"),
                "test",
                "he",
                "legacy-label",
                platform,
            )
            .unwrap();
            fs::write(f.game.join("later"), b"unrelated later file").unwrap();
            f
        }
        fn copy(&self, path: &str, op: &str, bytes: &[u8]) -> serde_json::Value {
            let name = format!("{:x}", Sha256::digest(bytes));
            fs::write(self.package.join(&name), bytes).unwrap();
            json!({"path":path,"operation":op,"source":{"kind":"copy","path":name,"sha256":name}})
        }
        fn plan(&self, version: &str, revision: u64, files: serde_json::Value) {
            fs::write(self.package.join("patch.json"), serde_json::to_vec(&json!({
                "schema_version":1,"scummvm_id":"test","language":"he","version":version,
                "release_revision":revision,"filename":"patch.zip","platforms":{"windows":{},"osx":{}},"files":files
            })).unwrap()).unwrap();
        }
        fn next(&self) {
            let mut data = self.copy("data", "modify", b"new translation");
            data["original_sha256"] = json!(format!("{:x}", Sha256::digest(b"original")));
            self.plan(
                "RC2-kitor1",
                1,
                json!([data, self.copy("new/subtitle", "create", b"new subtitle")]),
            );
        }
        fn update(&self, platform: &str) -> Result<PathBuf> {
            update(
                &self.dir,
                &self.package,
                "patch.json",
                &self.game,
                "test",
                "he",
                "RC2-kitor1",
                platform,
            )
        }
        fn assert_old(&self) {
            assert_eq!(
                fs::read(self.game.join("data")).unwrap(),
                b"old translation"
            );
            assert_eq!(
                fs::read(self.game.join("obsolete-original")).unwrap(),
                b"old replacement"
            );
            assert_eq!(fs::read(self.game.join("old/new")).unwrap(), b"old created");
            assert!(!self.game.join("new/subtitle").exists());
            assert_eq!(load(&self.dir).unwrap().version, "legacy-label");
        }
    }

    #[test]
    fn update_and_retry_preserve_originals_and_uninstall_selectively() {
        for platform in ["windows", "osx"] {
            let f = Fixture::new(platform);
            f.next();
            assert_eq!(f.update(platform).unwrap(), f.dir);
            f.update(platform).unwrap();
            assert_eq!(fs::read(f.game.join("data")).unwrap(), b"new translation");
            assert_eq!(
                fs::read(f.game.join("obsolete-original")).unwrap(),
                b"restore me"
            );
            assert!(!f.game.join("old/new").exists());
            let journal = load(&f.dir).unwrap();
            assert_eq!(journal.release_revision, 1);
            assert_eq!(
                fs::read_dir(f.dir.join(&journal.originals_dir))
                    .unwrap()
                    .count(),
                2
            );
            assert_eq!(
                fs::read(original_file(&f.dir, &journal, 0).unwrap()).unwrap(),
                b"original"
            );
            // A second update still starts from the same original game bytes.
            let mut data = f.copy("data", "modify", b"third translation");
            data["original_sha256"] = json!(format!("{:x}", Sha256::digest(b"original")));
            f.plan("RC2-kitor2", 2, json!([data]));
            update(
                &f.dir,
                &f.package,
                "patch.json",
                &f.game,
                "test",
                "he",
                "RC2-kitor2",
                platform,
            )
            .unwrap();
            assert!(!f.game.join("new/subtitle").exists());
            uninstall(&f.dir, &f.game).unwrap();
            assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
            assert_eq!(fs::read(f.game.join("save")).unwrap(), b"save game");
            assert_eq!(
                fs::read(f.game.join("later")).unwrap(),
                b"unrelated later file"
            );
            assert!(!f.game.join("old").exists());
            assert!(!f.game.join("new").exists());
        }
    }

    #[test]
    fn reuses_backups_across_reordering_and_only_adds_new_originals() {
        for platform in ["windows", "osx"] {
            let f = Fixture::new(platform);
            // Also exercise installations created by the previous updater.
            fs::rename(f.dir.join("originals"), f.dir.join("originals-update-legacy")).unwrap();
            let mut old = load(&f.dir).unwrap();
            old.originals_dir = "originals-update-legacy".into();
            save(&f.dir, &old).unwrap();
            let original = original_file(&f.dir, &old, 0).unwrap();
            let modified = fs::metadata(&original).unwrap().modified().unwrap();
            fs::write(f.game.join("additional"), b"additional original").unwrap();
            f.plan("RC2-kitor1", 1, json!([
                f.copy("additional", "modify", b"additional translation"),
                f.copy("data", "modify", b"new translation")
            ]));
            // A failed update removes only newly introduced original backups.
            assert!(update_inner(&f.dir, &f.package, "patch.json", &f.game,
                "test", "he", "RC2-kitor1", platform, &|_, _| {}, &|_, applied| {
                    if applied { Err("injected failure".into()) } else { Ok(()) }
                }).is_err());
            f.assert_old();
            assert_eq!(fs::read_dir(f.dir.join(&old.originals_dir)).unwrap().count(), 2);
            assert!(!f.dir.join("pending-update").exists());
            f.update(platform).unwrap();
            let next = load(&f.dir).unwrap();
            assert_eq!(next.originals_dir, old.originals_dir);
            assert_eq!(original_file(&f.dir, &next, 1).unwrap(), original);
            assert_eq!(fs::metadata(&original).unwrap().modified().unwrap(), modified);
            assert_eq!(fs::read_dir(f.dir.join(&next.originals_dir)).unwrap().count(), 3);
            assert!(!f.dir.join("pending-update").exists());
            assert_eq!(fs::read_dir(&f.dir).unwrap().count(), 2); // manifest + originals
            uninstall(&f.dir, &f.game).unwrap();
            assert_eq!(fs::read(f.game.join("additional")).unwrap(), b"additional original");
            assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
        }
    }

    #[test]
    fn update_rejects_missing_backups_modified_files_bad_payload_and_downgrade() {
        for case in ["backup", "corrupt-backup", "original", "modified", "payload", "revision"] {
            let f = Fixture::new("windows");
            f.next();
            match case {
                "backup" => fs::remove_file(f.dir.join("originals/0")).unwrap(),
                "corrupt-backup" => fs::write(f.dir.join("originals/0"), b"corrupt!").unwrap(),
                "original" => {
                    let mut data = f.copy("data", "modify", b"new translation");
                    data["original_sha256"] = json!("wrong edition");
                    f.plan("RC2-kitor1", 1, json!([data]));
                }
                "modified" => fs::write(f.game.join("data"), b"Steam updated this").unwrap(),
                "payload" => {
                    let package: Package =
                        serde_json::from_slice(&fs::read(f.package.join("patch.json")).unwrap())
                            .unwrap();
                    if let Some(Source::Copy { path, .. }) = &package.files[0].source {
                        fs::write(f.package.join(path), b"corrupt").unwrap();
                    }
                }
                _ => f.plan(
                    "RC2-kitor1",
                    0,
                    json!([f.copy("data", "modify", b"downgrade")]),
                ),
            }
            assert!(f.update("windows").is_err(), "{case}");
            if case != "modified" {
                f.assert_old();
            } else {
                assert_eq!(
                    fs::read(f.game.join("data")).unwrap(),
                    b"Steam updated this"
                );
            }
        }
    }

    #[test]
    fn stages_delta_from_retained_original_without_temporary_backups() {
        for platform in ["windows", "osx"] {
            let f = Fixture::new(platform);
            let checksum = format!("{:x}", Sha256::digest(b"original"));
            let recipe = serde_json::to_vec(&json!({
                "schema_version": 1, "input_sha256": checksum,
                "output_size": 9,
                "operations": [{"offset": 0, "length": 8}, {"hex": "21"}]
            })).unwrap();
            fs::write(f.package.join("recipe.json"), &recipe).unwrap();
            f.plan("RC2-kitor1", 1, json!([{
                "path": "data", "operation": "modify", "original_sha256": checksum,
                "source": {"kind": "reconstruct", "path": "recipe.json",
                    "sha256": format!("{:x}", Sha256::digest(&recipe)), "input": "DATA"}
            }]));
            update_inner(&f.dir, &f.package, "patch.json", &f.game,
                "test", "he", "RC2-kitor1", platform, &|_, _| {}, &|pending, applied| {
                    if !applied {
                        // Recovery still contains the previous translation, not
                        // the original game bytes used to build the new output.
                        assert_eq!(fs::read(pending.join("originals/0")).unwrap(), b"old translation");
                        let stage = fs::read_dir(&f.dir).unwrap().map(|e| e.unwrap().path())
                            .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("stage-update-")).unwrap();
                        assert_eq!(fs::read_dir(&stage).unwrap().count(), 1);
                        assert!(stage.join("outputs").is_dir());
                        assert_eq!(fs::read(stage.join("outputs/0")).unwrap(), b"original!");
                    }
                    Ok(())
                }).unwrap();
            assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original!");
            uninstall(&f.dir, &f.game).unwrap();
            assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
        }
    }

    #[test]
    fn recovery_cleans_abandoned_staging_only_under_lock() {
        for platform in ["windows", "osx"] {
            let f = Fixture::new(platform);
            for name in ["stage-update-abandoned", "prepare-update-abandoned", "retired-originals"] {
                fs::create_dir(f.dir.join(name)).unwrap();
                fs::write(f.dir.join(name).join("partial"), b"temporary copy").unwrap();
            }
            fs::create_dir(f.dir.join("unrelated")).unwrap();
            let guard = lock(f.dir.parent().unwrap()).unwrap();
            assert!(recover_installation(&f.dir).is_err());
            assert!(f.dir.join("stage-update-abandoned/partial").is_file());
            drop(guard);
            recover_installation(&f.dir).unwrap();
            for name in ["stage-update-abandoned", "prepare-update-abandoned", "retired-originals"] {
                assert!(!f.dir.join(name).exists());
            }
            assert!(f.dir.join("unrelated").is_dir());
            assert_eq!(fs::read(f.dir.join("originals/0")).unwrap(), b"original");
            f.assert_old();
        }
    }

    #[test]
    fn recovery_finishes_interrupted_uninstall_and_backup_cleanup() {
        for platform in ["windows", "osx"] {
            for fully_restored in [false, true] {
                let f = Fixture::new(platform);
                let mut journal = load(&f.dir).unwrap();
                if fully_restored {
                    restore_inner(&f.dir, &mut journal).unwrap();
                    // Simulate interruption partway through deleting backups.
                    fs::remove_file(f.dir.join("originals/0")).unwrap();
                } else {
                    journal.status = "restoring".into();
                    save(&f.dir, &journal).unwrap();
                }
                assert_eq!(recover_installation(&f.dir).unwrap().2, "restored");
                assert!(!f.dir.join("originals").exists());
                assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
                assert_eq!(fs::read(f.game.join("later")).unwrap(), b"unrelated later file");
                uninstall(&f.dir, &f.game).unwrap();
            }
        }
    }

    #[test]
    fn failed_update_restores_previous_translation() {
        let f = Fixture::new("osx");
        f.next();
        let failure = update_inner(
            &f.dir,
            &f.package,
            "patch.json",
            &f.game,
            "test",
            "he",
            "RC2-kitor1",
            "osx",
            &|_, _| {},
            &|pending, applied| {
                if !applied {
                    fs::remove_file(pending.join("outputs/1")).unwrap();
                }
                Ok(())
            },
        );
        assert!(failure.is_err());
        f.assert_old();
        f.update("osx").unwrap();
    }

    #[test]
    fn interrupted_update_recovers_on_next_read_and_keeps_conflicts() {
        for conflict in [false, true] {
            let f = Fixture::new("osx");
            f.next();
            let interrupted = std::panic::catch_unwind(|| {
                let _ = update_inner(
                    &f.dir,
                    &f.package,
                    "patch.json",
                    &f.game,
                    "test",
                    "he",
                    "RC2-kitor1",
                    "osx",
                    &|_, _| {},
                    &|_, applied| {
                        if applied {
                            panic!("simulated process interruption before commit");
                        }
                        Ok(())
                    },
                );
            });
            assert!(interrupted.is_err());
            // Real process exits do not run TempDir destructors.
            fs::create_dir(f.dir.join("stage-update-interrupted")).unwrap();
            fs::write(f.dir.join("stage-update-interrupted/partial"), b"staged bytes").unwrap();
            if conflict {
                fs::write(f.game.join("data"), b"later edit").unwrap();
                assert!(recover_installation(&f.dir).is_err());
                assert!(f.dir.join("pending-update/manifest.json").is_file());
                assert!(f.dir.join("originals/0").is_file());
                assert!(f.dir.join("stage-update-interrupted/partial").is_file());
                assert_eq!(fs::read(f.game.join("data")).unwrap(), b"later edit");
            } else {
                assert_eq!(recover_installation(&f.dir).unwrap().0, "legacy-label");
                assert!(!f.dir.join("stage-update-interrupted").exists());
                f.assert_old();
                f.update("osx").unwrap();
            }
        }
    }

    #[test]
    fn committed_manifest_survives_interrupted_cleanup() {
        let f = Fixture::new("osx");
        f.next();
        let _ = std::panic::catch_unwind(|| {
            let _ = update_inner(
                &f.dir,
                &f.package,
                "patch.json",
                &f.game,
                "test",
                "he",
                "RC2-kitor1",
                "osx",
                &|_, _| {},
                &|pending, applied| {
                    if applied {
                        let next: Journal =
                            serde_json::from_slice(&fs::read(pending.join("next.json")).unwrap())
                                .unwrap();
                        save(&f.dir, &next).unwrap();
                        panic!("simulated interruption after commit");
                    }
                    Ok(())
                },
            );
        });
        assert_eq!(recover_installation(&f.dir).unwrap().0, "RC2-kitor1");
        assert_eq!(fs::read(f.game.join("data")).unwrap(), b"new translation");
        assert!(!f.dir.join("pending-update").exists());
        uninstall(&f.dir, &f.game).unwrap();
        assert_eq!(fs::read(f.game.join("data")).unwrap(), b"original");
    }
}
