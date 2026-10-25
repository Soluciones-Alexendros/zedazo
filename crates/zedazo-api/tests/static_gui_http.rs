//! E2E mínimo: un proceso `serve` con GUI estática, sin Caddy.

use std::fs;
use std::time::Duration;

use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zedazo_api::app::AppState;
use zedazo_api::serve::{serve, ApiConfig};

async fn http_get(addr: &str, path: &str) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    let raw = String::from_utf8_lossy(&buf);
    let status = raw
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let (head, body) = raw.split_once("\r\n\r\n").unwrap_or((raw.as_ref(), ""));
    (status, head.to_string(), body.to_string())
}

#[tokio::test]
async fn binary_like_serve_gui_without_caddy() {
    let data = tempdir().unwrap();
    let gui = tempdir().unwrap();
    fs::write(
        gui.path().join("index.html"),
        "<!doctype html><title>zedazo</title><p>sin-caddy</p>",
    )
    .unwrap();

    let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let cfg = ApiConfig {
        state,
        static_dir: Some(gui.path().to_path_buf()),
    };
    let handle = tokio::spawn(async move {
        serve(listener, cfg, async move {
            let _ = rx.await;
        })
        .await
    });

    // Espera breve a que acepte conexiones.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let (st, head, body) = http_get(&addr, "/").await;
    assert_eq!(st, 200, "GUI: {head}");
    assert!(body.contains("sin-caddy"));
    assert!(head
        .to_ascii_lowercase()
        .contains("x-content-type-options: nosniff"));

    let (st, _, body) = http_get(&addr, "/api/v1/health").await;
    assert_eq!(st, 200);
    assert!(body.contains("ok") || body.contains("status") || body.contains('{'));

    let (st, _, body) = http_get(&addr, "/api/v1/missing").await;
    assert_eq!(st, 404);
    assert!(!body.contains("sin-caddy"));

    tx.send(()).unwrap();
    handle.await.unwrap().unwrap();
}
