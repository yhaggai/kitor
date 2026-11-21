use super::*;

#[tokio::test]
async fn failed_http_and_invalid_zip_preserve_existing_download() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for status in ["404 Not Found", "200 OK"] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/patch.zip", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            socket.read(&mut request).await.unwrap();
            socket.write_all(format!("HTTP/1.1 {}\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found", status).as_bytes()).await.unwrap();
        });
        let dir = std::env::temp_dir().join(format!("kitor-http-test-{}-{}",
            std::process::id(), time::OffsetDateTime::now_utc().unix_timestamp_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("patch.zip");
        std::fs::write(&path, b"previous download").unwrap();
        let error = download_with_progress(&url, &path, |_| {}).await.unwrap_err();
        assert!(error.contains(if status.starts_with("404") { "HTTP 404" } else { "not a valid ZIP" }), "{}", error);
        assert_eq!(std::fs::read(&path).unwrap(), b"previous download");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        server.await.unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[tokio::test]
async fn reports_transfer_progress_with_known_and_unknown_sizes() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    for known_size in [true, false] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/patch.bin", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            socket.read(&mut request).await.unwrap();
            let length = if known_size { "Content-Length: 6\r\n" } else { "" };
            socket.write_all(format!("HTTP/1.1 200 OK\r\n{length}Connection: close\r\n\r\nabc").as_bytes()).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            socket.write_all(b"def").await.unwrap();
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("patch.bin");
        let reports = std::sync::Mutex::new(Vec::new());
        download_with_progress(&url, &path, |p| reports.lock().unwrap().push(p)).await.unwrap();
        server.await.unwrap();
        let reports = reports.into_inner().unwrap();
        assert!(reports.len() >= 3);
        assert_eq!(reports[0].downloaded, 0);
        assert_eq!(reports[0].total, if known_size { 6 } else { 0 });
        assert!(reports[1].downloaded > 0);
        let last = reports.last().unwrap();
        assert_eq!((last.downloaded, last.total, last.percentage), (6, 6, 100));
        assert_eq!(std::fs::read(path).unwrap(), b"abcdef");
    }
}

#[tokio::test]
async fn cancelling_a_stalled_download_removes_partial_file() {
    use super::super::install_cancellation::{begin_game_install, cancel_game_install, end_game_install, scope};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/patch.zip", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        socket.read(&mut request).await.unwrap();
        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\npartial").await.unwrap();
        // Leave the client waiting for the rest of the response.
        std::future::pending::<()>().await;
    });
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("patch.zip");
    std::fs::write(&path, b"previous download").unwrap();

    let run_id = "stalled-download-test".to_string();
    begin_game_install(run_id.clone()).unwrap();
    let started = tokio::sync::Notify::new();
    let download = scope(Some(run_id.clone()), download_with_progress(&url, &path, |_| {
        started.notify_one();
    }));
    let cancel = async {
        started.notified().await;
        // Let the transfer read the first bytes and wait for the rest.
        tokio::task::yield_now().await;
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        cancel_game_install(run_id.clone());
    };
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(download, cancel).0
    }).await;
    end_game_install(run_id);
    server.abort();

    assert_eq!(result.unwrap().unwrap_err(), "Installation cancelled");
    assert_eq!(std::fs::read(&path).unwrap(), b"previous download");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[tokio::test]
#[ignore = "requires network access to a public GitHub release"]
async fn downloads_public_tentacle_release() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("patch.zip");
    std::fs::write(&path, b"Not Found").unwrap();
    download_with_progress(
        "https://github.com/yhaggai/kitor/releases/download/patch-tentacle-he-v2.0-beta2-kitor1-download2/tentacle-he-2.0-beta2-kitor1-download2.zip",
        &path,
        |_| {},
    ).await.unwrap();
    let extracted = dir.path().join("extracted");
    crate::services::file_service::extract_zip(&path, &extracted).unwrap();
    assert!(extracted.join("_platforms/windows/classic/patch.json").is_file());
    assert!(extracted.join("_platforms/osx/classic/patch.json").is_file());
}
