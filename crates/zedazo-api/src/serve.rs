//! Arranque HTTP compartido (binario Docker, futuro desktop Tauri).

use std::future::Future;
use std::path::PathBuf;

use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::Router;
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::app::{self, AppState};
use crate::security_headers;

/// Configuración de proceso para [`serve`].
#[derive(Clone)]
pub struct ApiConfig {
    pub state: AppState,
    /// Directorio de export estático de la GUI. `None` = GUI off (defecto Docker).
    pub static_dir: Option<PathBuf>,
}

impl ApiConfig {
    /// Lee `ZEDAZO_*` (estado + `ZEDAZO_STATIC_DIR` opcional). GUI off si la env no está.
    pub fn from_env() -> anyhow::Result<Self> {
        let state = AppState::from_env()?;
        let static_dir = std::env::var_os("ZEDAZO_STATIC_DIR")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from);
        Ok(Self { state, static_dir })
    }

    /// Construye el router (API ± GUI estática + security headers si hay GUI).
    pub fn into_router(self) -> Router {
        build_router(self.state, self.static_dir)
    }
}

/// Sirve la API (y opcionalmente la GUI) hasta que `shutdown` se complete.
pub async fn serve(
    listener: TcpListener,
    mut cfg: ApiConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    // S-01: alinear allowlist al puerto real del listener (p. ej. :0 en tests).
    let port = listener.local_addr()?.port();
    let mut policy = crate::host_guard::HostPolicy::for_port(port);
    policy.extra_hosts = cfg.state.host_policy.extra_hosts.clone();
    cfg.state = cfg.state.with_host_policy(policy);
    let app = cfg.into_router();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}

fn build_router(state: AppState, static_dir: Option<PathBuf>) -> Router {
    let api = app::router(state);
    match static_dir {
        None => api,
        Some(dir) => api
            .fallback(move |req: Request| {
                let dir = dir.clone();
                async move { gui_fallback(dir, req).await }
            })
            .layer(middleware::from_fn(security_headers::middleware)),
    }
}

async fn gui_fallback(dir: PathBuf, req: Request) -> Response {
    let path = req.uri().path();
    if path == "/api" || path.starts_with("/api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }

    let index = ServeFile::new(dir.join("index.html"));
    let svc = ServeDir::new(&dir)
        .append_index_html_on_directories(true)
        .fallback(index);
    match svc.oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::header;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::tempdir;
    use tower::ServiceExt;

    fn fixture_gui(root: &std::path::Path) {
        fs::write(
            root.join("index.html"),
            "<!doctype html><title>zedazo</title><h1>gui</h1>",
        )
        .unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("assets/app.js"), "console.log(1)").unwrap();
        fs::create_dir_all(root.join("empty")).unwrap();
    }

    async fn body_text(res: Response) -> String {
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    #[tokio::test]
    async fn static_dir_serves_index_and_assets() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: Some(gui.path().to_path_buf()),
        }
        .into_router();

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(body_text(res).await.contains("gui"));

        let res = app
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/assets/app.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(body_text(res).await.contains("console.log"));
    }

    #[tokio::test]
    async fn spa_fallback_only_get_head_never_api() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: Some(gui.path().to_path_buf()),
        }
        .into_router();

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/procesar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(body_text(res).await.contains("gui"));

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .method("HEAD")
                    .uri("/ejecuciones")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .method("POST")
                    .uri("/procesar")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);

        let res = app
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/api/v1/no-such-route")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
        let ctype = res
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap().to_string())
            .unwrap_or_default();
        let body = body_text(res).await;
        assert!(
            !body.contains("<!doctype") && !body.contains("<html"),
            " /api/* no debe devolver HTML: {body:?}"
        );
        assert!(
            !ctype.contains("text/html"),
            " /api/* no debe ser text/html: {ctype}"
        );
    }

    #[tokio::test]
    async fn path_traversal_rejected() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: Some(gui.path().to_path_buf()),
        }
        .into_router();

        for uri in [
            "/../Cargo.toml",
            "/assets/../../Cargo.toml",
            "/%2e%2e/Cargo.toml",
            "/assets/%2e%2e/%2e%2e/Cargo.toml",
        ] {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .header(axum::http::header::HOST, "127.0.0.1:8080")
                        .uri(uri)
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = res.status();
            let body = body_text(res).await;
            assert!(
                !body.contains("[package]"),
                "no debe filtrar Cargo.toml vía {uri} (status={status}, body={body:?})"
            );
        }
    }

    #[tokio::test]
    async fn no_directory_listing() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: Some(gui.path().to_path_buf()),
        }
        .into_router();

        let res = app
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/empty/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = res.status();
        let body = body_text(res).await;
        let lower = body.to_ascii_lowercase();
        assert!(
            !lower.contains("directory listing") && !lower.contains("<ul>"),
            "no listing: {body:?}"
        );
        // Sin index en /empty/: ServeDir cae al fallback SPA (index.html raíz) o 404.
        assert!(
            status == StatusCode::OK || status == StatusCode::NOT_FOUND,
            "status={status}"
        );
    }

    #[tokio::test]
    async fn gui_off_by_default_no_static() {
        let data = tempdir().unwrap();
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: None,
        }
        .into_router();

        let res = app
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn security_headers_when_serving_gui() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let app = ApiConfig {
            state,
            static_dir: Some(gui.path().to_path_buf()),
        }
        .into_router();

        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.headers().get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );

        let res = app
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
        assert_eq!(
            res.headers().get(header::CONTENT_SECURITY_POLICY).unwrap(),
            security_headers::CONTENT_SECURITY_POLICY
        );
    }

    #[tokio::test]
    async fn graceful_shutdown_completes() {
        let data = tempdir().unwrap();
        let state = AppState::new(data.path().to_path_buf(), 1_000_000, 24).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let cfg = ApiConfig {
            state,
            static_dir: None,
        };
        let handle = tokio::spawn(async move {
            serve(listener, cfg, async move {
                let _ = rx.await;
            })
            .await
        });
        tx.send(()).unwrap();
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), handle)
            .await
            .expect("shutdown timeout")
            .expect("join");
        assert!(result.is_ok());
    }
}
