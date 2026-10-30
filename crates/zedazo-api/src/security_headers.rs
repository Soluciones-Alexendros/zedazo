//! Cabeceras de seguridad canónicas (paridad con Caddy; ADR-0021 / W2).
//!
//! `CONTENT_SECURITY_POLICY` es la política base (Caddy + respuestas no HTML).
//! En `text/html`, el middleware añade hashes `sha256-…` de cada `<script>` inline
//! (p. ej. el payload `__next_f` del export estático de Next) sin usar
//! `'unsafe-inline'`.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, HeaderMap, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Hash CSP del script de arranque de tema (`THEME_BOOT_SCRIPT` en apps/web).
pub const THEME_BOOT_SCRIPT_HASH: &str = "sha256-wZHa1Ad+sG4CVkMj20Js/YFPWl/cxb0RiIYwFGB1XXY=";

/// Prefijo CSP hasta `script-src` (sin los hashes).
const CSP_BEFORE_SCRIPT_SRC: &str =
    "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; ";

/// Sufijo CSP tras `script-src`.
const CSP_AFTER_SCRIPT_SRC: &str =
    "; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'";

/// CSP alineada con `deploy/Caddyfile*` (GUI same-origin). Política base: sin
/// `'unsafe-inline'` en `script-src`; el theme-boot va por hash fijo.
///
/// Debe coincidir byte a byte con los Caddyfile (tests de paridad).
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'sha256-wZHa1Ad+sG4CVkMj20Js/YFPWl/cxb0RiIYwFGB1XXY='; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'";

fn insert_transport_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
}

fn insert_csp(headers: &mut HeaderMap, csp: HeaderValue) {
    insert_transport_headers(headers);
    headers.insert(header::CONTENT_SECURITY_POLICY, csp);
}

/// Aplica cabeceras de seguridad con la CSP base (no HTML / API).
pub fn apply_headers(res: &mut Response) {
    insert_csp(
        res.headers_mut(),
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
}

/// Middleware Axum: cabeceras canónicas; en HTML, CSP con hashes de scripts inline.
pub async fn middleware(req: Request, next: Next) -> Response {
    let res = next.run(req).await;
    let is_html = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.to_ascii_lowercase().starts_with("text/html"));

    if !is_html {
        let mut res = res;
        apply_headers(&mut res);
        return res;
    }

    let status = res.status();
    let mut headers = res.headers().clone();
    let bytes = match res.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            let mut err = Response::builder()
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()));
            apply_headers(&mut err);
            return err;
        }
    };

    let html = String::from_utf8_lossy(&bytes);
    let csp = content_security_policy_for_html(&html);
    let csp_value = HeaderValue::from_str(&csp)
        .unwrap_or_else(|_| HeaderValue::from_static(CONTENT_SECURITY_POLICY));
    insert_csp(&mut headers, csp_value);

    let mut out = Response::builder()
        .status(status)
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::empty()));
    *out.headers_mut() = headers;
    out
}

/// Política CSP para un documento HTML: base + hashes de cada script inline.
pub fn content_security_policy_for_html(html: &str) -> String {
    content_security_policy_with_script_hashes(
        inline_script_bodies(html).into_iter().map(csp_sha256),
    )
}

fn content_security_policy_with_script_hashes(hashes: impl IntoIterator<Item = String>) -> String {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    seen.insert(THEME_BOOT_SCRIPT_HASH.to_string());
    for h in hashes {
        seen.insert(h);
    }
    let mut script_src = String::from("script-src 'self'");
    for hash in &seen {
        script_src.push_str(" '");
        script_src.push_str(hash);
        script_src.push('\'');
    }
    format!("{CSP_BEFORE_SCRIPT_SRC}{script_src}{CSP_AFTER_SCRIPT_SRC}")
}

fn csp_sha256(script: &str) -> String {
    let digest = Sha256::digest(script.as_bytes());
    format!("sha256-{}", base64_std(&digest))
}

/// Cuerpos de `<script>` sin atributo `src` (ignora mayúsculas en el tag).
fn inline_script_bodies(html: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let lower = html.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(rel) = lower[search_from..].find("<script") {
        let start = search_from + rel;
        let after_tag = start + "<script".len();
        let Some(gt) = lower[after_tag..].find('>') else {
            break;
        };
        let open_end = after_tag + gt;
        let attrs = &lower[after_tag..open_end];
        let has_src = attrs.contains("src=");

        let body_start = open_end + 1;
        let Some(close_rel) = lower[body_start..].find("</script>") else {
            break;
        };
        let body_end = body_start + close_rel;
        if !has_src {
            let body = &html[body_start..body_end];
            if !body.is_empty() {
                out.push(body);
            }
        }
        search_from = body_end + "</script>".len();
    }
    out
}

fn base64_std(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = (u32::from(data[i]) << 16) | (u32::from(data[i + 1]) << 8) | u32::from(data[i + 2]);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push(T[(n & 63) as usize] as char);
        i += 3;
    }
    match data.len() - i {
        1 => {
            let n = u32::from(data[i]) << 16;
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = (u32::from(data[i]) << 16) | (u32::from(data[i + 1]) << 8);
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push(T[((n >> 6) & 63) as usize] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    fn assert_caddy_parity(path: &str, contents: &str) {
        assert!(
            contents.contains("X-Content-Type-Options nosniff"),
            "{path}: falta X-Content-Type-Options"
        );
        assert!(
            contents.contains("Referrer-Policy no-referrer"),
            "{path}: falta Referrer-Policy"
        );
        assert!(
            contents.contains("X-Frame-Options DENY"),
            "{path}: falta X-Frame-Options"
        );
        let quoted = format!("Content-Security-Policy \"{CONTENT_SECURITY_POLICY}\"");
        assert!(
            contents.contains(&quoted),
            "{path}: CSP de Caddy ≠ CONTENT_SECURITY_POLICY"
        );
    }

    #[test]
    fn base_csp_embeds_theme_boot_hash_constant() {
        assert!(CONTENT_SECURITY_POLICY.contains(THEME_BOOT_SCRIPT_HASH));
        assert_eq!(
            content_security_policy_with_script_hashes(std::iter::empty()),
            CONTENT_SECURITY_POLICY
        );
    }

    #[test]
    fn caddyfiles_match_security_headers() {
        assert_caddy_parity("Caddyfile", include_str!("../../../deploy/Caddyfile"));
        assert_caddy_parity(
            "Caddyfile.local",
            include_str!("../../../deploy/Caddyfile.local"),
        );
        assert_caddy_parity(
            "Caddyfile.public",
            include_str!("../../../deploy/Caddyfile.public"),
        );
    }

    #[test]
    fn sha256_encoder_matches_known_next_flight_stub() {
        assert_eq!(
            csp_sha256("(self.__next_f=self.__next_f||[]).push([0])"),
            "sha256-OBTN3RiyCV4Bq7dFqZ5a2pAXjnCcCYeTJMO2I/LYKeo="
        );
    }

    #[test]
    fn html_csp_includes_next_flight_hashes_without_unsafe_inline() {
        let html = r#"<!doctype html><script src="/_next/a.js"></script>
<script>(self.__next_f=self.__next_f||[]).push([0])</script>
<script>self.__next_f.push([1,"x"])</script>"#;
        let csp = content_security_policy_for_html(html);
        assert!(csp.contains(THEME_BOOT_SCRIPT_HASH));
        assert!(csp.contains("sha256-OBTN3RiyCV4Bq7dFqZ5a2pAXjnCcCYeTJMO2I/LYKeo="));
        assert!(csp.contains(&csp_sha256(r#"self.__next_f.push([1,"x"])"#)));
        let script_src = csp
            .split(';')
            .map(str::trim)
            .find(|d| d.starts_with("script-src"))
            .expect("script-src");
        assert!(
            !script_src.contains("unsafe-inline"),
            "script-src no debe llevar unsafe-inline: {script_src}"
        );
        assert!(csp.contains("script-src 'self'"));
    }

    #[test]
    fn skips_external_script_tags() {
        let html = r#"<script src="/x.js"></script><script src='/y.js' async></script>"#;
        assert!(inline_script_bodies(html).is_empty());
        let csp = content_security_policy_for_html(html);
        assert!(csp.contains(THEME_BOOT_SCRIPT_HASH));
        assert_eq!(
            csp.matches("sha256-").count(),
            1,
            "no debe hashear scripts externos: {csp}"
        );
    }

    #[tokio::test]
    async fn middleware_sets_caddy_parity_headers() {
        let app = Router::new()
            .route("/ping", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(middleware));

        let res = app
            .oneshot(
                Request::builder()
                    .header(axum::http::header::HOST, "127.0.0.1:8080")
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
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

    #[tokio::test]
    async fn middleware_html_adds_inline_script_hashes() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    (
                        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                        Body::from(
                            r#"<!doctype html><script>(self.__next_f=self.__next_f||[]).push([0])</script>"#,
                        ),
                    )
                }),
            )
            .layer(axum::middleware::from_fn(middleware));

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
        assert_eq!(res.status(), StatusCode::OK);
        let csp = res
            .headers()
            .get(header::CONTENT_SECURITY_POLICY)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("sha256-OBTN3RiyCV4Bq7dFqZ5a2pAXjnCcCYeTJMO2I/LYKeo="));
        let script_src = csp
            .split(';')
            .map(str::trim)
            .find(|d| d.starts_with("script-src"))
            .expect("script-src");
        assert!(!script_src.contains("unsafe-inline"));
    }
}
