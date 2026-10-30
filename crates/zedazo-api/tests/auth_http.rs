//! Auth token / cookie / fail-closed (ADR-0016).

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use http_body_util::BodyExt;
use tempfile::tempdir;
use tower::ServiceExt;
use zedazo_api::app::{self, AppState};
use zedazo_api::auth::{self, AuthConfig};

async fn body_text(res: axum::response::Response) -> String {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[tokio::test]
async fn token_mode_rejects_without_credential() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap();

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .uri("/api/v1/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn token_mode_accepts_bearer() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap();

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .uri("/api/v1/version")
                .header(header::AUTHORIZATION, "Bearer secret-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn health_remains_public() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap();

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn login_sets_cookie_and_authorizes() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap();
    let app = app::router(state);

    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"token":"secret-token"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::NO_CONTENT);
    let set_cookie = login
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("zedazo_auth=secret-token"));
    assert!(set_cookie.contains("HttpOnly"));

    let cookie = set_cookie.split(';').next().unwrap();
    let res = app
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .uri("/api/v1/version")
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn login_rejects_bad_token() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", false),
    )
    .unwrap();

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("POST")
                .uri("/api/v1/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"token":"wrong"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    let _ = body_text(res).await;
}

#[test]
fn fail_closed_public_bind() {
    let auth = AuthConfig::disabled();
    assert!(auth::validate_bind_auth("0.0.0.0:8080", &auth).is_err());
    assert!(auth::validate_bind_auth("127.0.0.1:8080", &auth).is_ok());
}

#[tokio::test]
async fn protected_routes_reject_without_credential() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", true),
    )
    .unwrap();
    let app = app::router(state);
    let paths = [
        ("GET", "/api/v1/jobs"),
        ("GET", "/api/v1/jobs/01900000-0000-7000-8000-000000000001"),
        (
            "DELETE",
            "/api/v1/jobs/01900000-0000-7000-8000-000000000001",
        ),
        (
            "GET",
            "/api/v1/jobs/01900000-0000-7000-8000-000000000001/events",
        ),
        (
            "GET",
            "/api/v1/jobs/01900000-0000-7000-8000-000000000001/artifacts/vcf",
        ),
        ("POST", "/api/v1/admin/wipe"),
    ];
    for (method, uri) in paths {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "{method} {uri} debe exigir credencial"
        );
    }
}

#[tokio::test]
async fn logout_clears_cookie_flags() {
    let dir = tempdir().unwrap();
    let state = AppState::new_with_auth(
        dir.path().to_path_buf(),
        1_000_000,
        32,
        24,
        AuthConfig::token("secret-token", true),
    )
    .unwrap();
    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("POST")
                .uri("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let set_cookie = res
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(set_cookie.contains("Max-Age=0"));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Secure"));
}

/// S-01: auth disabled → sin CORS (Host validado aparte).
#[tokio::test]
async fn disabled_mode_has_no_cors_acao() {
    let dir = tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf(), 1_000_000, 24).unwrap();
    let origin = "http://127.0.0.1:3000";

    let res = app::router(state)
        .oneshot(
            Request::builder()
                .header(axum::http::header::HOST, "127.0.0.1:8080")
                .method("GET")
                .uri("/api/v1/health")
                .header(header::ORIGIN, origin)
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
}
