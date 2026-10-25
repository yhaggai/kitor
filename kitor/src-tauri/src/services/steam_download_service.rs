//! Curated Windows depot downloads through the user's authenticated Steam client.
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::Mutex;
static DOWNLOAD: Mutex<()> = Mutex::const_new(());

fn completion(line: &str, app: u32, depot: u32) -> bool {
    line.contains("Depot download complete :")
        && line
            .replace('\\', "/")
            .contains(&format!("/app_{app}/depot_{depot}\""))
}
fn copy_checked(source: &Path, dest: &Path) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(source).map_err(|e| e.to_string())?;
    if meta.file_type().is_symlink() {
        return Err("Steam data contains a symbolic link".into());
    }
    if meta.is_dir() {
        std::fs::create_dir(dest).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            copy_checked(&entry.path(), &dest.join(entry.file_name()))?;
        }
    } else if meta.is_file() {
        std::fs::copy(source, dest).map_err(|e| e.to_string())?;
        fn hash(path: &Path) -> Result<Vec<u8>, String> {
            let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
            let mut h = Sha256::new();
            let mut buf = [0u8; 65536];
            loop {
                let n = file.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                h.update(&buf[..n]);
            }
            Ok(h.finalize().to_vec())
        }
        if hash(source)? != hash(dest)? {
            return Err("Imported file verification failed".into());
        }
    } else {
        return Err("Unexpected Steam file type".into());
    }
    Ok(())
}

async fn download_monkey(
    app_id: u32,
    state: &crate::AppState,
) -> Result<crate::models::game::Model, String> {
    if !cfg!(target_os = "macos") || app_id != 32460 {
        return Err(
            "Windows downloads on Mac are currently supported only for Monkey Island 2.".into(),
        );
    }
    // Never move an existing installation out from under its patch backups or saves.
    if let Some(game) = super::game_service::get_all_games(&state.db)
        .await?
        .into_iter()
        .find(|g| g.scummvm_id == "monkey2-se")
    {
        if Path::new(&game.install_path).is_dir() {
            return Ok(game);
        }
        return Err("This game's registered folder is missing. Remove that library entry before downloading again.".into());
    }
    let home = std::env::var_os("HOME").ok_or("Cannot locate your home folder")?;
    let steam = PathBuf::from(home).join("Library/Application Support/Steam");
    let runtime = steam.join("Steam.AppBundle/Steam/Contents/MacOS");
    let executable = runtime.join("steam_osx");
    if !executable.is_file() {
        return Err("Install Steam and sign in, then try again.".into());
    }
    download_depot(&steam, &executable, &state.app_data_dir, 32460, 32461).await?;
    let source = [
        runtime.join("steamapps/content/app_32460/depot_32461"),
        steam.join("steamapps/content/app_32460/depot_32461"),
    ]
    .into_iter()
    .find(|p| {
        p.join("Monkey2.exe").is_file()
            && p.join("monkey2.pak").is_file()
            && p.join("audio/SFXOriginal.xwb").is_file()
    })
    .ok_or("Steam reported completion, but the expected game files were not found")?;
    let games = state.app_data_dir.join("games");
    let destination = games.join("monkey2-se-steam");
    let import = destination.clone();
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        std::fs::create_dir_all(&games).map_err(|e| e.to_string())?;
        if import.exists() { return Err("The managed game folder already exists. Add it to your library instead of overwriting it.".into()); }
        let staging = tempfile::tempdir_in(&games).map_err(|e| e.to_string())?;
        // Only Steam game content; never import separately installed patches or saves.
        for name in ["Monkey2.exe", "monkey2.pak", "steam_api.dll", "InstallScript.vdf", "audio", "installers"] {
            copy_checked(&source.join(name), &staging.path().join(name))?;
        }
        std::fs::rename(staging.path(), &import).map_err(|e| e.to_string())?;
        Ok(())
    }).await.map_err(|e| e.to_string())??;
    super::game_service::add_game_manually(
        &state.db,
        "monkey2-se".into(),
        "Monkey Island 2: LeChuck's Revenge Special Edition".into(),
        destination.to_string_lossy().into_owned(),
        state.app_data_dir.clone(),
    )
    .await
}

fn log_identity(metadata: &std::fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        metadata.ino()
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        0
    }
}

async fn download_depot(
    steam: &Path,
    executable: &Path,
    progress_dir: &Path,
    app: u32,
    depot: u32,
) -> Result<(), String> {
    let log = steam.join("logs/console_log.txt");
    std::fs::create_dir_all(progress_dir).map_err(|e| e.to_string())?;
    let request = progress_dir.join(format!("steam-download-{app}-{depot}.json"));
    // Keep the original offset after a timeout/restart. Retrying attaches to the
    // same Steam request instead of issuing a second in-flight download.
    let (mut offset, mut identity): (u64, u64) = if request.exists() {
        serde_json::from_slice(&std::fs::read(&request).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Cannot read pending Steam request: {e}"))?
    } else {
        let initial = std::fs::metadata(&log)
            .map(|m| (m.len(), log_identity(&m)))
            .unwrap_or((0, 0));
        std::fs::write(
            &request,
            serde_json::to_vec(&initial).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        match tokio::process::Command::new(executable)
            .args([
                "-console",
                "+download_depot",
                &app.to_string(),
                &depot.to_string(),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                tokio::spawn(async move {
                    let _ = child.wait().await;
                });
            }
            Err(e) => {
                let _ = std::fs::remove_file(&request);
                return Err(format!("Could not start Steam: {e}"));
            }
        }
        initial
    };
    let mut pending = String::new();
    let mut complete = false;
    for _ in 0..1800 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        if let Ok(mut file) = std::fs::File::open(&log) {
            let metadata = file.metadata().map_err(|e| e.to_string())?;
            let length = metadata.len();
            let current_identity = log_identity(&metadata);
            if length < offset || current_identity != identity {
                identity = current_identity;
                offset = 0;
                pending.clear();
            }
            file.seek(SeekFrom::Start(offset))
                .map_err(|e| e.to_string())?;
            let mut bytes = Vec::new();
            file.take(1024 * 1024)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            offset += bytes.len() as u64;
            pending.push_str(&String::from_utf8_lossy(&bytes));
            while let Some(end) = pending.find('\n') {
                let line: String = pending.drain(..=end).collect();
                if completion(&line, app, depot) {
                    complete = true;
                    break;
                }
                if line.contains(&depot.to_string())
                    && (line.contains("Depot download failed")
                        || line.contains("not owned")
                        || line.contains("Access Denied"))
                {
                    let _ = std::fs::remove_file(&request);
                    return Err("Steam could not download this game. Check that Steam is online and signed into an account that owns this collection or game, then retry.".into());
                }
            }
            if pending.len() > 1024 * 1024 {
                return Err("Unexpected Steam log format. Check the Steam console.".into());
            }
        }
        if complete {
            break;
        }
    }
    if !complete {
        return Err("Timed out waiting for Steam. The download may still be running in Steam; check its console before retrying.".into());
    }
    std::fs::remove_file(&request).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_is_depot_specific() {
        assert!(completion("Depot download complete : \"/Steam/MacOS\\steamapps\\content\\app_32460\\depot_32461\" (manifest 123)",32460,32461));
        assert!(!completion(
            "Downloading depot 32461 (40 files)",
            32460,
            32461
        ));
        assert!(!completion(
            "Depot download complete : \"/app_32460/depot_324610\"",
            32460,
            32461
        ));
        assert!(!completion(
            "Depot download complete : \"/app_99/depot_32461\"",
            32460,
            32461
        ));
    }
    #[test]
    fn import_preserves_bytes() {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("source");
        std::fs::create_dir(&s).unwrap();
        std::fs::write(s.join("game"), b"game data").unwrap();
        let d = t.path().join("dest");
        copy_checked(&s, &d).unwrap();
        assert_eq!(std::fs::read(d.join("game")).unwrap(), b"game data");
    }
    #[cfg(unix)]
    #[test]
    fn import_rejects_links() {
        let t = tempfile::tempdir().unwrap();
        let s = t.path().join("link");
        std::os::unix::fs::symlink("/etc/hosts", &s).unwrap();
        assert!(copy_checked(&s, &t.path().join("dest")).is_err());
    }
}

#[cfg(all(test, target_os = "macos"))]
mod live_test {
    #[tokio::test]
    #[ignore = "Requires signed-in Steam owning app 32460; downloads and imports about 2 GB"]
    async fn steam_download_and_register() {
        let dir = tempfile::tempdir().unwrap();
        let state = crate::AppState {
            db: crate::db::init_db(&dir.path().to_path_buf()).await.unwrap(),
            app_data_dir: dir.path().to_path_buf(),
        };
        let game = super::download(32460, vec!["monkey2-se".into()], &state)
            .await
            .unwrap()
            .remove(0);
        assert_eq!(game.scummvm_id, "monkey2-se");
        assert!(std::path::Path::new(&game.install_path)
            .join("audio/SFXOriginal.xwb")
            .is_file());
        assert!(!std::path::Path::new(&game.install_path)
            .join("hebrew")
            .exists());
        let again = super::download(32460, vec!["monkey2-se".into()], &state)
            .await
            .unwrap()
            .remove(0);
        assert_eq!(again.id, game.id);
    }
}

#[derive(serde::Deserialize)]
struct ImportFile {
    name: String,
    sha256: String,
}
#[derive(serde::Deserialize)]
struct ImportRecipe {
    id: String,
    title: String,
    folder: String,
    files: Vec<ImportFile>,
}
fn recipes() -> Vec<ImportRecipe> {
    serde_json::from_str(include_str!("../../data/space_quest_steam.json"))
        .expect("bundled Steam recipes")
}
fn reject_links(path: &Path) -> Result<(), String> {
    // Resolve the system temp/home prefix first at call sites where necessary;
    // explicit source components must never redirect outside the Steam tree.
    for ancestor in path.ancestors().take_while(|p| p.parent().is_some()) {
        if std::fs::symlink_metadata(ancestor)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(format!(
                "Symbolic link in game path: {}",
                ancestor.display()
            ));
        }
    }
    Ok(())
}
fn verify_recipe(root: &Path, recipe: &ImportRecipe) -> Result<(), String> {
    reject_links(root)?;
    for file in &recipe.files {
        let path = root.join(&file.name);
        let meta =
            std::fs::symlink_metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() == 0 {
            return Err(format!("Invalid game file: {}", path.display()));
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        if format!("{:x}", Sha256::digest(bytes)) != file.sha256 {
            return Err(format!(
                "Unverified edition or modified file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}
fn import_recipe(source: &Path, destination: &Path, recipe: &ImportRecipe) -> Result<(), String> {
    verify_recipe(source, recipe)?;
    reject_links(destination)?;
    let parent = destination.parent().ok_or("Missing import parent")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    if destination.exists() {
        return Err(format!(
            "{} already exists; add it manually instead of overwriting it",
            destination.display()
        ));
    }
    let staging = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    for file in &recipe.files {
        copy_checked(&source.join(&file.name), &staging.path().join(&file.name))?;
    }
    verify_recipe(staging.path(), recipe)?;
    std::fs::rename(staging.path(), destination).map_err(|e| e.to_string())
}

/// A store product is a download source, not a Kitor game identity.
/// Serialize imports too, so two cards cannot replace one another's installation.
pub async fn download(
    app_id: u32,
    game_ids: Vec<String>,
    state: &crate::AppState,
) -> Result<Vec<crate::models::game::Model>, String> {
    if !cfg!(target_os = "macos") {
        return Err("This download flow requires macOS".into());
    }
    let _guard = DOWNLOAD.try_lock().map_err(|_| {
        "A Steam download/import is already running in Kitor. Reopen its progress dialog."
    })?;
    if app_id == 32460 && game_ids == ["monkey2-se"] {
        return Ok(vec![download_monkey(app_id, state).await?]);
    }
    let all = recipes();
    if app_id != 10110
        || game_ids.is_empty()
        || game_ids.iter().any(|id| !all.iter().any(|r| &r.id == id))
    {
        return Err("Unsupported Steam product or game edition".into());
    }
    let existing = super::game_service::get_all_games(&state.db).await?;
    let mut result = Vec::new();
    let mut needed = Vec::new();
    for recipe in all.iter().filter(|r| game_ids.contains(&r.id)) {
        if let Some(game) = existing.iter().find(|g| g.scummvm_id == recipe.id) {
            if !Path::new(&game.install_path).is_dir() {
                return Err(format!(
                    "{} has a missing registered folder. Repair or remove its library entry first.",
                    recipe.title
                ));
            }
            result.push(game.clone());
        } else {
            needed.push(recipe);
        }
    }
    if needed.is_empty() {
        return Ok(result);
    }
    let steam = PathBuf::from(std::env::var_os("HOME").ok_or("Cannot locate home folder")?)
        .join("Library/Application Support/Steam");
    let runtime = steam.join("Steam.AppBundle/Steam/Contents/MacOS");
    let cache = state
        .app_data_dir
        .join("steam-sources/10110-6725209980292758062");
    if !needed
        .iter()
        .all(|r| verify_recipe(&cache.join(&r.id), r).is_ok())
    {
        let candidates = [
            runtime.join("steamapps/content/app_10110/depot_10111"),
            steam.join("steamapps/content/app_10110/depot_10111"),
        ];
        let valid_source = || {
            candidates
                .iter()
                .find(|p| {
                    all.iter()
                        .all(|r| verify_recipe(&p.join(&r.folder), r).is_ok())
                })
                .cloned()
        };
        // Exact known bytes can be reused even without an appmanifest or old console logs.
        let source = if let Some(source) = valid_source() {
            source
        } else {
            let executable = [
                runtime.join("steam_osx"),
                PathBuf::from("/Applications/Steam.app/Contents/MacOS/steam_osx"),
            ]
            .into_iter()
            .find(|p| p.is_file())
            .ok_or("Install Steam and sign in first")?;
            download_depot(&steam, &executable, &state.app_data_dir, 10110, 10111).await?;
            valid_source().ok_or("Steam completed, but this collection's files do not match the verified edition. No games were imported.")?
        };
        for recipe in &all {
            let destination = cache.join(&recipe.id);
            if verify_recipe(&destination, recipe).is_ok() {
                continue;
            }
            import_recipe(&source.join(&recipe.folder), &destination, recipe)?;
        }
    }
    for recipe in needed {
        let destination = state
            .app_data_dir
            .join("games")
            .join(format!("{}-steam", recipe.id));
        // A previous registration failure leaves a pristine, recoverable import.
        if !destination.exists() {
            import_recipe(&cache.join(&recipe.id), &destination, recipe)?;
        } else {
            verify_recipe(&destination, recipe).map_err(|_| format!("{} already exists and differs from the original. Add it manually; Kitor will not overwrite saves or patches.", destination.display()))?;
        }
        let game = super::game_service::add_game_manually(
            &state.db,
            recipe.id.clone(),
            recipe.title.clone(),
            destination.to_string_lossy().into_owned(),
            state.app_data_dir.clone(),
        )
        .await
        .map_err(|e| {
            format!(
                "Files retained at {}. Add this folder manually if retry fails: {e}",
                destination.display()
            )
        })?;
        result.push(game);
    }
    Ok(result)
}

#[cfg(test)]
mod collection_tests {
    use super::*;
    #[test]
    fn imports_only_verified_files_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let source = root.join("source");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(source.join("resource.001"), b"original").unwrap();
        std::fs::write(source.join("unrelated-patch"), b"excluded").unwrap();
        let recipe = ImportRecipe {
            id: "sq3".into(),
            title: "SQ3".into(),
            folder: "sq3".into(),
            files: vec![ImportFile {
                name: "resource.001".into(),
                sha256: format!("{:x}", Sha256::digest(b"original")),
            }],
        };
        let destination = root.join("game");
        import_recipe(&source, &destination, &recipe).unwrap();
        assert!(!destination.join("unrelated-patch").exists());
        std::fs::write(destination.join("save"), b"save").unwrap();
        assert!(import_recipe(&source, &destination, &recipe).is_err());
        assert_eq!(std::fs::read(destination.join("save")).unwrap(), b"save");
        std::fs::write(source.join("resource.001"), b"modified").unwrap();
        assert!(import_recipe(&source, &root.join("bad"), &recipe).is_err());
        assert!(!root.join("bad").exists());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("real")).unwrap();
        std::os::unix::fs::symlink(root.join("real"), root.join("link")).unwrap();
        assert!(reject_links(&root.join("link/nested")).is_err());
    }
    #[tokio::test]
    async fn pending_request_resumes_without_spawning_even_after_log_rotation() {
        for rotate in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let logs = dir.path().join("logs");
            std::fs::create_dir(&logs).unwrap();
            let log = logs.join("console_log.txt");
            std::fs::write(&log, "old log\n").unwrap();
            let meta = std::fs::metadata(&log).unwrap();
            let marker = dir.path().join("steam-download-10110-10111.json");
            std::fs::write(
                &marker,
                serde_json::to_vec(&(meta.len(), log_identity(&meta))).unwrap(),
            )
            .unwrap();
            if rotate {
                std::fs::rename(&log, logs.join("previous.txt")).unwrap();
            }
            let prefix = if rotate { "" } else { "old log\n" };
            std::fs::write(&log, format!("{prefix}Depot download complete : \"/Steam/app_10110/depot_10111\" (manifest 123)\n")).unwrap();
            // This cannot be spawned: success proves retry resumed the pending request.
            download_depot(
                dir.path(),
                Path::new("/nonexistent-steam"),
                dir.path(),
                10110,
                10111,
            )
            .await
            .unwrap();
            assert!(!marker.exists());
        }
    }
    #[test]
    fn bundled_recipes_are_distinct_and_contained() {
        let recipes = recipes();
        assert_eq!(recipes.len(), 5);
        let mut ids = std::collections::HashSet::new();
        for recipe in recipes {
            assert!(ids.insert(recipe.id));
            assert!(!recipe.files.is_empty());
            for file in recipe.files {
                assert_eq!(Path::new(&file.name).components().count(), 1);
                assert!(matches!(
                    Path::new(&file.name).components().next(),
                    Some(std::path::Component::Normal(_))
                ));
                assert_eq!(file.sha256.len(), 64);
            }
        }
    }
    #[cfg(target_os = "macos")]
    #[tokio::test]
    #[ignore = "Requires the owned Space Quest Steam depot downloaded locally"]
    async fn live_collection_imports_and_reuses_each_game() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let state = crate::AppState {
            db: crate::db::init_db(&root).await.unwrap(),
            app_data_dir: root.clone(),
        };
        let first = download(10110, vec!["sq1".into()], &state).await.unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].scummvm_id, "sq1");
        let original_path = first[0].install_path.clone();
        // Existing installations may be patched or have saves; retry must preserve them.
        std::fs::write(Path::new(&original_path).join("VOL.0"), b"patched data").unwrap();
        std::fs::write(Path::new(&original_path).join("save.test"), b"save").unwrap();
        let all = download(
            10110,
            recipes().iter().map(|r| r.id.clone()).collect(),
            &state,
        )
        .await
        .unwrap();
        assert_eq!(all.len(), 5);
        assert_eq!(
            super::super::game_service::get_all_games(&state.db)
                .await
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            std::fs::read(Path::new(&original_path).join("VOL.0")).unwrap(),
            b"patched data"
        );
        assert_eq!(
            std::fs::read(Path::new(&original_path).join("save.test")).unwrap(),
            b"save"
        );
        for game in all.iter().filter(|g| g.scummvm_id != "sq1") {
            let recipe = recipes()
                .into_iter()
                .find(|r| r.id == game.scummvm_id)
                .unwrap();
            verify_recipe(Path::new(&game.install_path), &recipe).unwrap();
            assert!(Path::new(&game.install_path).starts_with(root.join("games")));
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod manual_import {
    /// Explicit operator utility, separate from the disposable integration test.
    #[tokio::test]
    #[ignore = "Writes to KITOR_STEAM_IMPORT_LIBRARY only when explicitly supplied"]
    async fn import_collection_into_explicit_library() {
        let path = std::env::var_os("KITOR_STEAM_IMPORT_LIBRARY")
            .expect("Set the intended Kitor data directory explicitly");
        let root = std::path::PathBuf::from(path).canonicalize().unwrap();
        let state = crate::AppState {
            db: crate::db::init_db(&root).await.unwrap(),
            app_data_dir: root,
        };
        let games = super::download(
            10110,
            super::recipes().into_iter().map(|r| r.id).collect(),
            &state,
        )
        .await
        .unwrap();
        for game in games {
            println!("{}: {}", game.scummvm_id, game.install_path);
        }
    }
}
