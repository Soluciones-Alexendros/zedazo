//! S-01: Host/Origin/CORS en disabled, token y escritorio.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_api::auth::AuthConfig;
use zedazo_api::host_guard::HostPolicy;

#[tokio::test]
async fn disabled_no_acao_and_foreign_host_403() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let app = app::router(state);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::ORIGIN, "https://evil.example")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/admin/wipe")
                .header(header::HOST, "evil.example:8080")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn token_preflight_wipe_not_2xx_without_cors_origin() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("tok", false),
    )
    .unwrap();
    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/v1/admin/wipe")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::ORIGIN, "https://evil.example")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        !res.status().is_success(),
        "preflight no debe ser 2xx sin CORS allowlist: {}",
        res.status()
    );
}

#[tokio::test]
async fn unsafe_origin_foreign_403() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24)
        .unwrap()
        .with_host_policy(HostPolicy::for_port(8080));
    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/uploads")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::ORIGIN, "https://evil.example")
                .header(header::CONTENT_TYPE, "multipart/form-data; boundary=x")
                .body(Body::from("--x--\r\n"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn data_tree_permissions_0600_0700() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1000, 24).unwrap();
    let root = &state.storage.root;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for sub in ["jobs", "tmp", "uploads"] {
            let p = root.join(sub);
            let mode = std::fs::metadata(&p).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "{sub} mode={mode:#o}");
        }
    }
}

#[tokio::test]
async fn desktop_token_no_acao_foreign_host_403() {
    // Escritorio = AuthConfig::token + HostPolicy del puerto (mismo guard API).
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("desktop-session-token", false),
    )
    .unwrap()
    .with_host_policy(HostPolicy::for_port(43111));
    let app = app::router(state);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .header(header::HOST, "127.0.0.1:43111")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .is_none());

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/admin/wipe")
                .header(header::HOST, "evil.example:43111")
                .header(header::AUTHORIZATION, "Bearer desktop-session-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn retention_hours_out_of_range_400() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    // Crear upload mínimo vía storage para poder POSTear job
    let upload_id = "00000000-0000-7000-8000-000000000001";
    let path = state.storage.upload_path(upload_id);
    std::fs::write(&path, b"BEGIN:VCARD\r\nEND:VCARD\r\n").unwrap();

    let body = serde_json::json!({
        "upload_id": upload_id,
        "retention_hours": 99999
    });
    let res = app::router(state)
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/jobs")
                .header(header::HOST, "127.0.0.1:8080")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let _ = res.into_body().collect().await;
}
