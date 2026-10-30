//! Servidor loopback same-process: Host/Origin 403 + Bearer vía AuthConfig token.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use zedazo_api::app::AppState;
use zedazo_api::auth::AuthConfig;
use zedazo_api::serve::ApiConfig;

/// Contexto del guard Host/Origin (solo escritorio).
#[derive(Clone, Debug)]
pub struct LoopbackGuard {
    pub port: u16,
}

impl LoopbackGuard {
    fn allowed_host(&self, host: &str) -> bool {
        let host = host.trim();
        // Sin puerto implícito 80 — exigimos host:port exacto del listener.
        let expected = [
            format!("127.0.0.1:{}", self.port),
            format!("localhost:{}", self.port),
            format!("[::1]:{}", self.port),
        ];
        expected.iter().any(|e| e.eq_ignore_ascii_case(host))
    }

    fn allowed_origin(&self, origin: &str) -> bool {
        let origin = origin.trim();
        let expected = [
            format!("http://127.0.0.1:{}", self.port),
            format!("http://localhost:{}", self.port),
            format!("http://[::1]:{}", self.port),
        ];
        expected.iter().any(|e| e == origin)
    }
}

/// 403 si Host no es loopback:puerto; si Origin está presente, debe coincidir.
pub async fn host_origin_guard(
    State(guard): State<Arc<LoopbackGuard>>,
    req: Request,
    next: Next,
) -> Response {
    let headers = req.headers();
    match check_host_origin(headers, &guard) {
        Ok(()) => next.run(req).await,
        Err(status) => status.into_response(),
    }
}

fn check_host_origin(headers: &HeaderMap, guard: &LoopbackGuard) -> Result<(), StatusCode> {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if host.is_empty() || !guard.allowed_host(host) {
        return Err(StatusCode::FORBIDDEN);
    }
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        if !origin.is_empty() && !guard.allowed_origin(origin) {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(())
}

/// Construye el router de escritorio: API + GUI + guard Host/Origin (sin CORS abierto).
pub fn desktop_router(cfg: ApiConfig, guard: LoopbackGuard) -> Router {
    let g = Arc::new(guard);
    cfg.into_router()
        .layer(middleware::from_fn_with_state(g, host_origin_guard))
}

/// Arranca `127.0.0.1:0` con token de sesión; devuelve addr, token y canal de shutdown.
pub async fn start_desktop_server(
    data_dir: PathBuf,
    static_dir: PathBuf,
    session_token: String,
) -> anyhow::Result<(
    SocketAddr,
    oneshot::Sender<()>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
)> {
    let auth = AuthConfig::token(session_token, false);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    if !addr.ip().is_loopback() {
        anyhow::bail!("bind no-loopback rechazado: {addr}");
    }

    let state = AppState::new_with_auth(data_dir, 52_428_800, 4, 24, auth)?
        .with_host_policy(zedazo_api::host_guard::HostPolicy::for_port(addr.port()));

    // S-12: misma reconciliación/retención que el binario API (primer tick = barrido al arrancar).
    zedazo_api::retention::reconcile_on_startup(&state).await?;
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                zedazo_api::retention::retention_sweep(&st).await;
            }
        });
    }

    let cfg = ApiConfig {
        state,
        static_dir: Some(static_dir),
    };

    // Guard Host/Origin ya en zedazo-api::app::router; LoopbackGuard extra por defensa en profundidad.
    let guard = LoopbackGuard { port: addr.port() };
    let app = desktop_router(cfg, guard);

    let (tx, rx) = oneshot::channel::<()>();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    });

    Ok((addr, tx, handle))
}

/// Error al resolver la GUI estática para `ServeDir`.
#[derive(Debug, Clone)]
pub struct StaticDirError {
    pub message: String,
    pub tried: Vec<PathBuf>,
}

impl std::fmt::Display for StaticDirError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for StaticDirError {}

fn require_index(dir: &std::path::Path) -> Result<(), StaticDirError> {
    let index = dir.join("index.html");
    if index.is_file() {
        return Ok(());
    }
    Err(StaticDirError {
        message: format!(
            "GUI no encontrada: falta index.html en {}. \
Defina ZEDAZO_STATIC_DIR (desarrollo) o empaquete bundle.resources gui/**/* (AppImage).",
            dir.display()
        ),
        tried: vec![dir.to_path_buf()],
    })
}

/// Selección pura: override de env → recursos del paquete → candidatos de desarrollo.
/// Cada candidata debe contener `index.html`; no se acepta un stub vacío como éxito implícito.
pub fn resolve_static_dir_from(
    env_override: Option<PathBuf>,
    resource_gui: Option<PathBuf>,
    dev_candidates: &[PathBuf],
) -> Result<PathBuf, StaticDirError> {
    let mut tried = Vec::new();

    if let Some(p) = env_override {
        tried.push(p.clone());
        require_index(&p).map_err(|mut e| {
            e.tried = tried.clone();
            e
        })?;
        return Ok(p);
    }

    if let Some(p) = resource_gui {
        tried.push(p.clone());
        if p.join("index.html").is_file() {
            return Ok(p);
        }
    }

    for c in dev_candidates {
        tried.push(c.clone());
        if c.join("index.html").is_file() {
            return Ok(c.clone());
        }
    }

    Err(StaticDirError {
        message: format!(
            "GUI no encontrada (sin ZEDAZO_STATIC_DIR ni resources ni out/). Probado: {}",
            tried
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        tried,
    })
}

/// Resuelve el directorio GUI para el servidor loopback (ADR-0021).
///
/// Orden: `ZEDAZO_STATIC_DIR` → `$resource_dir/gui` (bundle.resources) →
/// `apps/web/out` / `apps/desktop/gui` relativos al manifest (solo checkout de desarrollo).
pub fn resolve_static_dir(app: &tauri::AppHandle) -> Result<PathBuf, StaticDirError> {
    use tauri::Manager;

    let env_override = std::env::var_os("ZEDAZO_STATIC_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);

    let resource_gui = app.path().resource_dir().ok().map(|r| r.join("gui"));

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev = [manifest.join("../../apps/web/out"), manifest.join("gui")];

    resolve_static_dir_from(env_override, resource_gui, &dev)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session;
    use axum::body::Body;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::tempdir;
    use tower::ServiceExt;
    use zedazo_api::app::AppState;
    use zedazo_api::auth::AuthConfig;
    use zedazo_api::serve::ApiConfig;

    fn fixture_gui(root: &std::path::Path) {
        fs::write(
            root.join("index.html"),
            "<!doctype html><title>zedazo</title><h1>gui</h1>",
        )
        .unwrap();
    }

    fn app_with_token(
        data: &std::path::Path,
        gui: &std::path::Path,
        token: &str,
        port: u16,
    ) -> Router {
        let state = AppState::new_with_auth(
            data.to_path_buf(),
            1_000_000,
            4,
            24,
            AuthConfig::token(token, false),
        )
        .unwrap()
        .with_host_policy(zedazo_api::host_guard::HostPolicy::for_port(port));
        desktop_router(
            ApiConfig {
                state,
                static_dir: Some(gui.to_path_buf()),
            },
            LoopbackGuard { port },
        )
    }

    #[tokio::test]
    async fn protected_without_bearer_is_401() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let app = app_with_token(
            data.path(),
            gui.path(),
            "tokensecret32byteshexpad00000001",
            9_001,
        );

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .header(header::HOST, "127.0.0.1:9001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn protected_with_bearer_ok() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let token = "tokensecret32byteshexpad00000001";
        let app = app_with_token(data.path(), gui.path(), token, 9_001);

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/version")
                    .header(header::HOST, "127.0.0.1:9001")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn foreign_host_is_403() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let token = "tokensecret32byteshexpad00000001";
        let app = app_with_token(data.path(), gui.path(), token, 9_001);

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(header::HOST, "evil.example:9001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn host_without_port_is_403() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let app = app_with_token(
            data.path(),
            gui.path(),
            "tokensecret32byteshexpad00000001",
            9_001,
        );

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(header::HOST, "127.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn token_mode_has_no_open_cors() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let app = app_with_token(
            data.path(),
            gui.path(),
            "tokensecret32byteshexpad00000001",
            9_001,
        );

        let res = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/v1/health")
                    .header(header::HOST, "127.0.0.1:9001")
                    .header(header::ORIGIN, "http://evil.example")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Guard Host/Origin o CORS vacío: nunca ACAO=* / evil.
        assert_ne!(res.status(), StatusCode::OK);
        let acao = res
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            acao.is_empty() || (acao != "*" && !acao.contains("evil")),
            "CORS abierto inesperado: {acao:?}"
        );
    }

    #[tokio::test]
    async fn foreign_origin_is_403() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let token = "tokensecret32byteshexpad00000001";
        let app = app_with_token(data.path(), gui.path(), token, 9_001);

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(header::HOST, "127.0.0.1:9001")
                    .header(header::ORIGIN, "http://evil.example")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn loopback_origin_allowed() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let app = app_with_token(
            data.path(),
            gui.path(),
            "tokensecret32byteshexpad00000001",
            9_001,
        );

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .header(header::HOST, "127.0.0.1:9001")
                    .header(header::ORIGIN, "http://127.0.0.1:9001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn bind_is_loopback_only() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let token = session::generate_session_token().unwrap();
        let (addr, tx, handle) =
            start_desktop_server(data.path().to_path_buf(), gui.path().to_path_buf(), token)
                .await
                .unwrap();
        assert!(addr.ip().is_loopback());
        assert_ne!(addr.port(), 0);
        let _ = tx.send(());
        let _ = handle.await;
    }

    #[tokio::test]
    async fn gui_served_on_loopback_host() {
        let data = tempdir().unwrap();
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let app = app_with_token(
            data.path(),
            gui.path(),
            "tokensecret32byteshexpad00000001",
            9_001,
        );

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::HOST, "127.0.0.1:9001")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains("gui"));
    }

    #[test]
    fn resolve_prefers_env_override_with_index() {
        let gui = tempdir().unwrap();
        fixture_gui(gui.path());
        let got = resolve_static_dir_from(Some(gui.path().to_path_buf()), None, &[]).unwrap();
        assert_eq!(got, gui.path());
    }

    #[test]
    fn resolve_rejects_env_without_index() {
        let empty = tempdir().unwrap();
        let err = resolve_static_dir_from(Some(empty.path().to_path_buf()), None, &[]).unwrap_err();
        assert!(err.message.contains("index.html"));
    }

    #[test]
    fn resolve_uses_resource_gui_before_dev() {
        let resource = tempdir().unwrap();
        let gui = resource.path().join("gui");
        fs::create_dir_all(&gui).unwrap();
        fixture_gui(&gui);
        let fake_dev = tempdir().unwrap();
        fixture_gui(fake_dev.path());
        let got =
            resolve_static_dir_from(None, Some(gui.clone()), &[fake_dev.path().to_path_buf()])
                .unwrap();
        assert_eq!(got, gui);
    }

    #[test]
    fn resolve_falls_back_to_dev_candidate() {
        let dev = tempdir().unwrap();
        fixture_gui(dev.path());
        let missing = tempdir().unwrap();
        let got = resolve_static_dir_from(
            None,
            Some(missing.path().join("gui")),
            &[dev.path().to_path_buf()],
        )
        .unwrap();
        assert_eq!(got, dev.path());
    }

    #[test]
    fn resolve_errors_when_nothing_has_index() {
        let a = tempdir().unwrap();
        let b = tempdir().unwrap();
        let err =
            resolve_static_dir_from(None, Some(a.path().join("gui")), &[b.path().to_path_buf()])
                .unwrap_err();
        assert!(!err.tried.is_empty());
        assert!(err.message.contains("GUI no encontrada"));
    }
}
