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
