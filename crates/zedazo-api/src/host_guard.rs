//! Guard Host/Origin común (S-01): loopback, disabled, token y escritorio.

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::app::AppState;

/// Política de Host/Origin (puerto de escucha + `ZEDAZO_ALLOWED_HOSTS`).
#[derive(Clone, Debug)]
pub struct HostPolicy {
    pub listen_port: u16,
    pub extra_hosts: Vec<String>,
}

impl HostPolicy {
    pub fn for_port(port: u16) -> Self {
        Self {
            listen_port: port,
            extra_hosts: parse_allowed_hosts_env(),
        }
    }

    pub fn from_bind(bind: &str) -> Self {
        let port = bind
            .rsplit_once(':')
            .and_then(|(_, p)| p.parse().ok())
            .unwrap_or(8080);
        Self::for_port(port)
    }

    pub fn allows_host(&self, host: &str) -> bool {
        let host = host.trim();
        if host.is_empty() {
            return false;
        }
        let port = self.listen_port;
        let expected = [
            format!("127.0.0.1:{port}"),
            format!("localhost:{port}"),
            format!("[::1]:{port}"),
        ];
        if expected.iter().any(|e| e.eq_ignore_ascii_case(host)) {
            return true;
        }
        self.extra_hosts
            .iter()
            .any(|e| e.eq_ignore_ascii_case(host))
    }

    pub fn allows_origin(&self, origin: &str) -> bool {
        let origin = origin.trim();
        if origin.is_empty() {
            return false;
        }
        let port = self.listen_port;
        let expected = [
            format!("http://127.0.0.1:{port}"),
            format!("http://localhost:{port}"),
            format!("http://[::1]:{port}"),
        ];
        if expected.iter().any(|e| e == origin) {
            return true;
        }
        // Extra hosts → orígenes http(s)://host
        for h in &self.extra_hosts {
            let http = format!("http://{h}");
            let https = format!("https://{h}");
            if origin == http || origin == https {
                return true;
            }
        }
        false
    }
}

fn parse_allowed_hosts_env() -> Vec<String> {
    std::env::var("ZEDAZO_ALLOWED_HOSTS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn method_is_unsafe(method: &Method) -> bool {
    !matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

fn check(headers: &HeaderMap, method: &Method, policy: &HostPolicy) -> Result<(), StatusCode> {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !policy.allows_host(host) {
        return Err(StatusCode::FORBIDDEN);
    }
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Preflight / métodos no seguros: Origin ajeno → 403 (S-01).
    if (!origin.is_empty())
        && (method_is_unsafe(method) || *method == Method::OPTIONS)
        && !policy.allows_origin(origin)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

/// Middleware: Host allowlist; Origin ajeno en métodos no seguros → 403.
pub async fn host_origin_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    match check(req.headers(), req.method(), &state.host_policy) {
        Ok(()) => next.run(req).await,
        Err(status) => status.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts() {
        let p = HostPolicy {
            listen_port: 8080,
            extra_hosts: vec!["localhost:8443".into(), "127.0.0.1:8443".into()],
        };
        assert!(p.allows_host("127.0.0.1:8080"));
        assert!(p.allows_host("localhost:8443"));
        assert!(!p.allows_host("evil.example:8080"));
        assert!(p.allows_origin("http://127.0.0.1:8080"));
        assert!(p.allows_origin("https://localhost:8443"));
        assert!(!p.allows_origin("https://evil.example"));
    }
}
