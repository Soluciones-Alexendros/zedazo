//! Cache-Control: no-store en endpoints sensibles (S-02).

use axum::body::Body;
use axum::http::header::CACHE_CONTROL;
use axum::http::{Request, StatusCode};
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_api::auth::AuthConfig;

fn test_state(dir: &std::path::Path) -> AppState {
    AppState::new_with_auth(
        dir.to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap()
}

async fn get_with_auth(state: AppState, uri: &str) -> axum::response::Response {
    app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .uri(uri)
                .header(axum::http::header::AUTHORIZATION, "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn contacts_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(
        state,
        "/api/v1/jobs/01900000-0000-7000-8000-000000000001/contacts",
    )
    .await;

    // 404 porque no existe el job, pero la cabecera debe estar igualmente
    assert!(matches!(
        res.status(),
        StatusCode::NOT_FOUND | StatusCode::OK
    ));
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn jobs_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(state, "/api/v1/jobs").await;

    assert_eq!(res.status(), StatusCode::OK);
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn duplicates_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(
        state,
        "/api/v1/jobs/01900000-0000-7000-8000-000000000002/duplicates",
    )
    .await;

    assert!(matches!(
        res.status(),
        StatusCode::NOT_FOUND | StatusCode::OK
    ));
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn stats_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(
        state,
        "/api/v1/jobs/01900000-0000-7000-8000-000000000003/stats",
    )
    .await;

    assert!(matches!(
        res.status(),
        StatusCode::NOT_FOUND | StatusCode::OK
    ));
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn audit_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(
        state,
        "/api/v1/jobs/01900000-0000-7000-8000-000000000004/audit",
    )
    .await;

    assert!(matches!(
        res.status(),
        StatusCode::NOT_FOUND | StatusCode::OK
    ));
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn version_endpoint_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(state, "/api/v1/version").await;

    assert_eq!(res.status(), StatusCode::OK);
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}

#[tokio::test]
async fn health_endpoint_is_cacheable() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());
    let res = get_with_auth(state, "/api/v1/health").await;

    assert_eq!(res.status(), StatusCode::OK);
    let cc = res.headers().get(CACHE_CONTROL);
    // health no debe llevar no-store (puede no tener cabecera)
    if let Some(cc) = cc {
        assert_ne!(cc.to_str().unwrap(), "no-store, private");
    }
}

#[tokio::test]
async fn auth_login_has_no_store() {
    let dir = tempdir().unwrap();
    let state = test_state(dir.path());

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"token":"wrong"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    // 401 pero la cabecera debe estar presente
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let cc = res
        .headers()
        .get(CACHE_CONTROL)
        .expect("Cache-Control debe estar presente");
    assert_eq!(cc.to_str().unwrap(), "no-store, private");
}
