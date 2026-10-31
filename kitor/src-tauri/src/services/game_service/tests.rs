use super::*;
use crate::services::game_adapters::is_space_quest;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = tempfile::Builder::new().prefix("kitor-detection-")
            .tempdir().unwrap().keep();
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

#[tokio::test]
async fn library_includes_game_without_any_patches() {
    let fixture = Fixture::new();
    let db = crate::db::init_db(&fixture.0.join("db")).await.unwrap();
    let game = game::ActiveModel {
        scummvm_id: Set("unpatched-library-fixture".to_string()),
        title: Set("Game without a patch".to_string()),
        install_path: Set(fixture.0.to_string_lossy().to_string()),
        added_date: Set(chrono::Utc::now().naive_utc()),
        ..Default::default()
    }.insert(&db).await.unwrap();

    let library = get_games_with_patches(&db, fixture.0.clone()).await.unwrap();
    assert_eq!(library.len(), 1);
    assert_eq!(library[0].id, game.id);
    assert!(!library[0].has_available_patch);
    assert!(library[0].available_patches.is_empty());
    assert!(library[0].installed_patches.is_empty());
    assert!(library[0].requires_scummvm);
}

#[test]
#[ignore = "requires local collection; set KITOR_SPACE_QUEST_DIR"]
fn detects_space_quest_collection() {
    let root = PathBuf::from(std::env::var("KITOR_SPACE_QUEST_DIR").unwrap());
    let signatures: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/space_quest_detection.json"
    )).unwrap();
    for signature in signatures.as_array().unwrap() {
        let id = signature["id"].as_str().unwrap();
        let source = root.join(format!("{id}-steam"));
        let detected = detect_game_in_folder(&source).unwrap().unwrap();
        assert_eq!(detected.scummvm_id, id);
        assert_eq!(PathBuf::from(detected.install_path), source);
        let fixture = Fixture::new();
        for name in signature["required"].as_array().unwrap() {
            let name = name.as_str().unwrap();
            std::fs::copy(source.join(name), fixture.0.join(name.to_lowercase())).unwrap();
        }
        assert_eq!(detect_game_in_folder(&fixture.0).unwrap().unwrap().scummvm_id, id);
        for other in ["sq1", "sq1sci", "sq2", "sq3", "sq4", "sq5", "sq6"] {
            assert_eq!(is_space_quest(&fixture.0, other), id == other);
        }
        let file = fixture.0.join(signature["signature"].as_str().unwrap().to_lowercase());
        std::fs::write(file, b"unrecognized game data").unwrap();
        assert!(detect_game_in_folder(&fixture.0).unwrap().is_none());
    }
}

#[test]
fn sq2_rejects_shared_agi_names_without_matching_contents() {
    let fixture = Fixture::new();
    for name in ["LOGDIR", "PICDIR", "SNDDIR", "VIEWDIR", "VOL.0", "VOL.1", "VOL.2"] {
        std::fs::write(fixture.0.join(name), b"another AGI game").unwrap();
    }
    assert!(!is_space_quest(&fixture.0, "sq2"));
    assert!(detect_game_in_folder(&fixture.0).unwrap().is_none());
}

#[test]
#[ignore = "requires local SQ2 data; set KITOR_SQ2_DIR"]
fn detects_local_sq2_and_case_insensitive_copy() {
    let path = PathBuf::from(std::env::var("KITOR_SQ2_DIR").unwrap());
    let detected = detect_game_in_folder(&path).unwrap().unwrap();
    assert_eq!(detected.scummvm_id, "sq2");
    assert_eq!(PathBuf::from(detected.install_path), path);
    let fixture = Fixture::new();
    for name in ["LOGDIR", "PICDIR", "SNDDIR", "VIEWDIR", "VOL.0", "VOL.1", "VOL.2"] {
        std::fs::copy(path.join(name), fixture.0.join(name.to_lowercase())).unwrap();
    }
    assert_eq!(detect_game_in_folder(&fixture.0).unwrap().unwrap().scummvm_id, "sq2");
    std::fs::remove_file(fixture.0.join("vol.0")).unwrap();
    assert!(detect_game_in_folder(&fixture.0).unwrap().is_none());
}

#[tokio::test]
async fn grim_remastered_detection_and_scan() {
    let fixture = Fixture::new();
    let root = fixture.0.join("Games/Grim Fandango Remastered");
    std::fs::create_dir_all(&root).unwrap();
    assert!(detect_game_in_folder(&root).unwrap().is_none());
    // A directory with the executable's name must not count as a game.
    let executable = root.join("grimfandango.EXE");
    std::fs::create_dir(&executable).unwrap();
    assert!(detect_game_in_folder(&root).unwrap().is_none());
    std::fs::remove_dir(&executable).unwrap();
    std::fs::write(&executable, b"fixture").unwrap();
    let found = detect_game_for_id(&root, "grim-fandango-remastered").unwrap().unwrap();
    assert_eq!(PathBuf::from(found.install_path), root);
    assert!(found.available_patches_count > 0);
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    for selected in [root.parent().unwrap(), root.as_path()] {
        scan_directory(&db, selected.to_str().unwrap(), fixture.0.clone()).await.unwrap();
    }
    let games = Game::find().all(&db).await.unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].scummvm_id, "grim-fandango-remastered");
    assert_eq!(Path::new(&games[0].install_path), root);
}

#[test]
fn detects_toonstruck_data_and_gog_bundle() {
    let fixture = Fixture::new();
    let bundle = fixture.0.join("Toonstruck.app");
    let data = bundle.join("Contents/Resources/game/game");
    let misc = data.join("mIsC");
    std::fs::create_dir_all(&misc).unwrap();
    std::fs::write(misc.join("ToonFont.CAF"), b"fixture").unwrap();
    assert!(detect_game_in_folder(&bundle).unwrap().is_none());
    std::fs::create_dir(misc.join("TOONSTRK.RES")).unwrap();
    assert!(detect_game_in_folder(&bundle).unwrap().is_none());
    std::fs::remove_dir(misc.join("TOONSTRK.RES")).unwrap();
    std::fs::write(misc.join("TOONSTRK.RES"), b"fixture").unwrap();
    for selected in [&bundle, &data] {
        let detected = detect_game_in_folder(selected).unwrap().unwrap();
        assert_eq!(detected.scummvm_id, "toonstruck");
        assert_eq!(PathBuf::from(detected.install_path), data);
    }
}

#[tokio::test]
async fn scans_toonstruck_gog_windows_library() {
    let fixture = Fixture::new();
    let root = fixture.0.join("Games/Toonstruck");
    let data = root.join("Toonstruck");
    std::fs::create_dir_all(data.join("MISC")).unwrap();
    for name in ["TOONFONT.CAF", "TOONSTRK.RES"] {
        std::fs::write(data.join("MISC").join(name), b"fixture").unwrap();
    }
    assert_eq!(PathBuf::from(detect_game_in_folder(&root).unwrap().unwrap().install_path), data);
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    let games = scan_directory(&db, root.parent().unwrap().to_str().unwrap(), fixture.0.clone()).await.unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].scummvm_id, "toonstruck");
    assert_eq!(Path::new(&games[0].install_path), data);
}

#[tokio::test]
#[ignore = "requires local GOG Toonstruck; set KITOR_TOONSTRUCK_APP"]
async fn scans_local_toonstruck_installation() {
    let fixture = Fixture::new();
    let bundle = PathBuf::from(std::env::var("KITOR_TOONSTRUCK_APP").unwrap());
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    let games = scan_directory(&db, bundle.to_str().unwrap(), fixture.0.clone()).await.unwrap();
    assert_eq!(games.len(), 1);
    assert_eq!(games[0].scummvm_id, "toonstruck");
    assert_eq!(Path::new(&games[0].install_path), bundle.join("Contents/Resources/game/game"));
    assert!(scan_directory(&db, bundle.to_str().unwrap(), fixture.0.clone()).await.unwrap().is_empty());
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

#[tokio::test]
async fn owlboy_scan_resolves_data_and_deduplicates() {
    let fixture = Fixture::new();
    let bundle = fixture.0.join("Owlboy/Owlboy.app");
    let data = bundle.join("Contents/Resources");
    std::fs::create_dir_all(data.join("Data/Localizations")).unwrap();
    std::fs::create_dir_all(data.join("Data/Maps")).unwrap();
    std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
    for name in ["Owlboy.exe", "FNA.dll", "Data/versioninfo.txt", "Data/Localizations/de.csv"] {
        std::fs::write(data.join(name), b"fixture").unwrap();
    }
    assert!(detect_game_in_folder(&bundle).unwrap().is_none());
    std::fs::write(bundle.join("Contents/MacOS/Owlboy"), b"fixture").unwrap();
    for selected in [bundle.parent().unwrap(), &bundle, &data] {
        let found = detect_game_in_folder(selected).unwrap().unwrap();
        assert_eq!(found.scummvm_id, "owlboy");
        assert_eq!(Path::new(&found.install_path), data.canonicalize().unwrap());
    }
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    for _ in 0..2 {
        scan_directory(&db, fixture.0.to_str().unwrap(), fixture.0.clone()).await.unwrap();
    }
    let games = Game::find().all(&db).await.unwrap();
    assert_eq!(games.iter().filter(|g|g.scummvm_id == "owlboy").count(), 1);
}

#[tokio::test]
async fn tentacle_detection_scan_is_idempotent() {
    let fixture = Fixture::new();
    let root = fixture.0.join("Day of the Tentacle Remastered");
    std::fs::create_dir_all(root.join("Dott.app/Contents/Resources")).unwrap();
    std::fs::create_dir_all(root.join("Dott.app/Contents/MacOS")).unwrap();
    std::fs::write(root.join("Dott.app/Contents/MacOS/Dott"), b"runner").unwrap();
    std::fs::write(root.join("Dott.app/Contents/Resources/tenta.cle"),
        [b"KAPL\0\0\x80?".as_slice(), &[0;40]].concat()).unwrap();
    for path in [&root, &root.join("Dott.app"), &root.join("Dott.app/Contents/Resources")] {
        let detected = detect_game_in_folder(path).unwrap().unwrap();
        assert_eq!(detected.scummvm_id, "day-of-the-tentacle-remastered");
        assert_eq!(PathBuf::from(detected.install_path), root.canonicalize().unwrap());
    }
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    for _ in 0..2 { scan_directory(&db, root.to_str().unwrap(), fixture.0.clone()).await.unwrap(); }
    assert_eq!(Game::find().all(&db).await.unwrap().len(), 2);
}

#[cfg(target_os = "windows")]
#[tokio::test]
async fn tentacle_windows_exposes_both_patches() {
    let fixture = Fixture::new();
    let root = fixture.0.join("DOTT");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("Dott.exe"), b"fixture").unwrap();
    std::fs::write(root.join("tenta.cle"),
        [b"KAPL\0\0\x80?".as_slice(), &[0; 40]].concat()).unwrap();
    let db = crate::db::init_db(&fixture.0.join("db")).await.unwrap();
    for _ in 0..2 {
        scan_directory(&db, root.to_str().unwrap(), fixture.0.clone()).await.unwrap();
    }
    let games = get_games_with_patches(&db, fixture.0.clone()).await.unwrap();
    assert_eq!(games.len(), 2);
    for id in ["tentacle", "day-of-the-tentacle-remastered"] {
        let game = games.iter().find(|game| game.scummvm_id == id).unwrap();
        assert!(game.has_available_patch);
        assert_eq!(game.available_patches.len(), 1);
        assert_eq!(game.requires_scummvm, id == "tentacle");
    }
}

#[tokio::test]
#[ignore = "requires KITOR_TENTACLE_PATH; scans read-only using a temporary library database"]
async fn tentacle_local_scan() {
    let root = PathBuf::from(std::env::var("KITOR_TENTACLE_PATH").unwrap());
    let temp = tempfile::tempdir().unwrap();
    let found = detect_game_in_folder(&root).unwrap().unwrap();
    assert_eq!(found.scummvm_id, "day-of-the-tentacle-remastered");
    let db = crate::db::init_db(&temp.path().join("db")).await.unwrap();
    for _ in 0..2 { scan_directory(&db, root.to_str().unwrap(), temp.path().to_path_buf()).await.unwrap(); }
    assert_eq!(Game::find().all(&db).await.unwrap().len(), 2);
}

#[tokio::test]
#[ignore = "requires KITOR_OWLBOY_PATH; uses only a temporary library database"]
async fn owlboy_local_detection_and_scan() {
    let path = PathBuf::from(std::env::var("KITOR_OWLBOY_PATH").unwrap());
    let temp = tempfile::tempdir().unwrap();
    let found = detect_game_in_folder(&path).unwrap().unwrap();
    assert_eq!(found.scummvm_id,"owlboy");
    let db = crate::db::init_db(&temp.path().join("database")).await.unwrap();
    for _ in 0..2 { scan_directory(&db,path.to_str().unwrap(),temp.path().to_path_buf()).await.unwrap(); }
    let games = Game::find().all(&db).await.unwrap();
    assert_eq!(games.iter().filter(|g|g.scummvm_id == "owlboy").count(),1);
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

#[tokio::test]
async fn detects_and_scans_drifter_bundle_and_parent_folders() {
    let fixture = Fixture::new();
    let game_folder = fixture.0.join("The Drifter");
    let bundle = game_folder.join("TheDrifter.app");
    std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
    std::fs::create_dir_all(bundle.join("Contents/Resources/Data")).unwrap();
    assert!(detect_game_in_folder(&game_folder).unwrap().is_none());
    std::fs::write(bundle.join("Contents/MacOS/The Drifter"), b"runner").unwrap();
    assert!(detect_game_in_folder(&bundle).unwrap().is_none());
    std::fs::write(bundle.join("Contents/Resources/Data/globalgamemanagers"), b"data").unwrap();
    let expected = bundle.canonicalize().unwrap();
    let db = crate::db::init_db(&fixture.0.join("database")).await.unwrap();
    for selected in [&fixture.0, &game_folder, &bundle] {
        let games = scan_directory(&db, selected.to_str().unwrap(), fixture.0.clone()).await.unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].scummvm_id, "drifter");
        assert_eq!(Path::new(&games[0].install_path), expected);
        remove_game(&db, games[0].id).await.unwrap();
    }
}

#[test]
#[ignore = "requires a local Drifter installation; set KITOR_DRIFTER_PATH"]
fn detects_local_drifter_installation() {
    let path = PathBuf::from(std::env::var("KITOR_DRIFTER_PATH").unwrap());
    let detected = detect_game_in_folder(&path).unwrap().unwrap();
    assert_eq!(detected.scummvm_id, "drifter");
    assert!(Path::new(&detected.install_path).join("Contents/MacOS/The Drifter").is_file());
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

#[test]
#[ignore = "requires the installed Steam SQ4 CD; set KITOR_SQ4_DIR"]
fn detects_local_sq4_cd() {
    let dir = PathBuf::from(std::env::var("KITOR_SQ4_DIR").unwrap());
    let game = detect_game_in_folder(&dir).unwrap().unwrap();
    assert_eq!(game.scummvm_id, "sq4");
    assert_eq!(PathBuf::from(game.install_path), dir);
    assert!(!is_space_quest(&dir, "sq5"));
}

#[tokio::test]
#[ignore = "requires the Steam CD edition; set KITOR_SQ4_DIR"]
async fn steam_library_scan_adds_all_space_quest_editions_once() {
    let game = PathBuf::from(std::env::var("KITOR_SQ4_DIR").unwrap());
    let collection = game.parent().unwrap().parent().unwrap();
    let sandbox = Fixture::new();
    let db = crate::db::init_db(&sandbox.0.join("db")).await.unwrap();
    scan_directory(&db, collection.parent().unwrap().to_str().unwrap(), sandbox.0.clone()).await.unwrap();
    let found = get_games_with_patches(&db, sandbox.0.clone()).await.unwrap();
    let sq4 = found.iter().find(|g| g.scummvm_id == "sq4").unwrap();
    assert_eq!(Path::new(&sq4.install_path), game);
    assert!(sq4.has_available_patch);
    for id in ["sq1", "sq1sci", "sq2", "sq3", "sq4", "sq5", "sq6"] {
        assert_eq!(found.iter().filter(|g| g.scummvm_id == id).count(), 1, "{id}");
    }
    assert!(!found.iter().find(|g| g.scummvm_id == "sq6").unwrap().has_available_patch);
    assert!(scan_directory(&db, collection.to_str().unwrap(), sandbox.0.clone()).await.unwrap().is_empty());
}
