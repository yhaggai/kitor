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
