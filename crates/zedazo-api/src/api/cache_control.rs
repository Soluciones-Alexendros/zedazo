//! Middleware de `Cache-Control: no-store` para endpoints sensibles (S-02).
//!
//! Aplica a `/api/v1/*` excepto `/health` y `/metrics`.

use axum::extract::Request;
use axum::http::header::CACHE_CONTROL;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Rutas que NO deben recibir no-store (públicas, cacheables).
const CACHEABLE_PATHS: &[&str] = &["/api/v1/health", "/api/v1/metrics"];

/// Middleware que añade `Cache-Control: no-store, private` a endpoints sensibles.
pub async fn cache_control_middleware(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let mut response = next.run(req).await;

    // No aplicar a rutas cacheables
    if CACHEABLE_PATHS
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{p}/")))
    {
        return response;
    }

    // No aplicar a respuestas no exitosas (errores 4xx/5xx no deberían cachearse pero ya lo indica el status)
    // Aplicar a todas las respuestas de /api/v1/ (incluyendo errores)
    if path.starts_with("/api/v1/") {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static("no-store, private"));
    }

    response
}

/// Verifica si una ruta debe llevar no-store (para tests).
pub fn should_no_store(path: &str) -> bool {
    if !path.starts_with("/api/v1/") {
        return false;
    }
    !CACHEABLE_PATHS
        .iter()
        .any(|p| path == *p || path.starts_with(&format!("{p}/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_cacheable() {
        assert!(!should_no_store("/api/v1/health"));
        assert!(!should_no_store("/api/v1/metrics"));
    }

    #[test]
    fn sensitive_endpoints_no_store() {
        assert!(should_no_store("/api/v1/contacts"));
        assert!(should_no_store("/api/v1/jobs"));
        assert!(should_no_store("/api/v1/jobs/abc/contacts"));
        assert!(should_no_store("/api/v1/stats"));
        assert!(should_no_store("/api/v1/audit"));
        assert!(should_no_store("/api/v1/auth/login"));
    }

    #[test]
    fn non_api_paths_not_affected() {
        assert!(!should_no_store("/"));
        assert!(!should_no_store("/procesar"));
        assert!(!should_no_store("/static/app.js"));
    }
}
