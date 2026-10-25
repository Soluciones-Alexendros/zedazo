//! Cabeceras de seguridad canónicas (paridad con Caddy; ADR-0021 / W2).

use axum::extract::Request;
use axum::http::{header, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

/// CSP alineada con `deploy/Caddyfile*` (GUI same-origin).
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'";

/// Aplica las cabeceras de seguridad a la respuesta.
pub fn apply_headers(res: &mut Response) {
    let headers = res.headers_mut();
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
}

/// Middleware Axum: añade cabeceras de seguridad a todas las respuestas.
pub async fn middleware(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    apply_headers(&mut res);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn middleware_sets_caddy_parity_headers() {
        let app = Router::new()
            .route("/ping", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(middleware));

        let res = app
            .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers().get(header::X_CONTENT_TYPE_OPTIONS).unwrap(),
            "nosniff"
        );
        assert_eq!(
            res.headers().get(header::REFERRER_POLICY).unwrap(),
            "no-referrer"
        );
        assert_eq!(res.headers().get(header::X_FRAME_OPTIONS).unwrap(), "DENY");
        assert_eq!(
            res.headers().get(header::CONTENT_SECURITY_POLICY).unwrap(),
            CONTENT_SECURITY_POLICY
        );
    }
}
