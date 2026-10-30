//! Zedazo API — adaptador HTTP sobre zedazo-core (ADR-0015).

use tracing_subscriber::EnvFilter;
use zedazo_api::auth;
use zedazo_api::retention;
use zedazo_api::serve::{serve, ApiConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args().any(|a| a == "--health-check") {
        return health_check_once().await;
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("zedazo_api=info".parse()?))
        .init();

    let cfg = ApiConfig::from_env()?;
    let bind = std::env::var("ZEDAZO_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    auth::validate_bind_auth(&bind, &cfg.state.auth)?;

    retention::reconcile_on_startup(&cfg.state).await?;

    // Barrido periódico de retención (TTL aproximado; no es eliminación al instante exacto).
    {
        let st = cfg.state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                retention::retention_sweep(&st).await;
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    if let Some(ref dir) = cfg.static_dir {
        tracing::info!(
            "zedazo-api escuchando en http://{bind} (GUI estática: {})",
            dir.display()
        );
    } else {
        tracing::info!("zedazo-api escuchando en http://{bind} (GUI off)");
    }

    serve(listener, cfg, shutdown_signal()).await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("installing Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("installing signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Comprobación de salud para Docker HEALTHCHECK (sin autenticación ni deps HTTP extra).
async fn health_check_once() -> anyhow::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let bind = std::env::var("ZEDAZO_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let addr = bind.rsplit_once('@').map(|(_, h)| h).unwrap_or(&bind);
    // Connect puede usar 0.0.0.0:puerto; el Host debe ser loopback (S-01 host_guard).
    let port = addr
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let connect_addr = if addr.starts_with("0.0.0.0:") || addr.starts_with("[::]:") {
        format!("127.0.0.1:{port}")
    } else {
        addr.to_string()
    };
    let host_header = format!("127.0.0.1:{port}");
    let mut stream = tokio::net::TcpStream::connect(&connect_addr).await?;
    let req =
        format!("GET /api/v1/health HTTP/1.1\r\nHost: {host_header}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await?;
    let mut buf = vec![0u8; 512];
    let n = stream.read(&mut buf).await?;
    let body = String::from_utf8_lossy(&buf[..n]);
    if !body.starts_with("HTTP/1.1 200") && !body.starts_with("HTTP/1.0 200") {
        anyhow::bail!("health check falló: {}", body.lines().next().unwrap_or(""));
    }
    Ok(())
}
