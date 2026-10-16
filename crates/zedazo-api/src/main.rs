//! Zedazo API — adaptador HTTP sobre zedazo-core (ADR-0015).

use tracing_subscriber::EnvFilter;
use zedazo_api::app;
use zedazo_api::auth;
use zedazo_api::retention;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--health-check") {
        return health_check_once().await;
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("zedazo_api=info".parse()?))
        .init();

    let state = app::AppState::from_env()?;
    let bind = std::env::var("ZEDAZO_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    auth::validate_bind_auth(&bind, &state.auth)?;

    retention::reconcile_on_startup(&state).await?;

    // Barrido periódico de retención (TTL aproximado; no es eliminación al instante exacto).
    {
        let st = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                retention::retention_sweep(&st).await;
            }
        });
    }
    let app = app::router(state.clone());

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("zedazo-api escuchando en http://{bind}");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Comprobación de salud para Docker HEALTHCHECK (sin autenticación ni deps HTTP extra).
async fn health_check_once() -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let bind = std::env::var("ZEDAZO_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let addr = bind.rsplit_once('@').map(|(_, h)| h).unwrap_or(&bind);
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    let req = format!("GET /api/v1/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await?;
    let mut buf = vec![0u8; 512];
    let n = stream.read(&mut buf).await?;
    let body = String::from_utf8_lossy(&buf[..n]);
    if !body.starts_with("HTTP/1.1 200") && !body.starts_with("HTTP/1.0 200") {
        anyhow::bail!("health check falló: {}", body.lines().next().unwrap_or(""));
    }
    Ok(())
}
