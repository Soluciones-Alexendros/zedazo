//! Autenticación single-user (ADR-0016).

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::app::AppState;

/// Cabeceras explícitas: tower-http prohíbe `Allow-Headers: *` con credentials.
fn cors_allow_headers() -> [HeaderName; 4] {
    [
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        header::ACCEPT,
        header::COOKIE,
    ]
}

fn cors_allow_methods() -> [Method; 5] {
    [
        Method::GET,
        Method::POST,
        Method::DELETE,
        Method::OPTIONS,
        Method::HEAD,
    ]
}

pub const COOKIE_NAME: &str = "zedazo_auth";

#[derive(Clone, Debug)]
pub enum AuthMode {
    Disabled,
    Token(String),
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub mode: AuthMode,
    pub cookie_secure: bool,
}

impl AuthConfig {
    pub fn disabled() -> Self {
        Self {
            mode: AuthMode::Disabled,
            cookie_secure: false,
        }
    }

    pub fn token(token: impl Into<String>, cookie_secure: bool) -> Self {
        Self {
            mode: AuthMode::Token(token.into()),
            cookie_secure,
        }
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let mode_raw = std::env::var("ZEDAZO_AUTH_MODE").unwrap_or_else(|_| "disabled".into());
        let cookie_secure = std::env::var("ZEDAZO_AUTH_COOKIE_SECURE")
            .ok()
            .and_then(|s| match s.to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" => Some(true),
                "0" | "false" | "no" => Some(false),
                _ => None,
            })
            .unwrap_or(true);

        match mode_raw.to_ascii_lowercase().as_str() {
            "disabled" => Ok(Self {
                mode: AuthMode::Disabled,
                cookie_secure: false,
            }),
            "token" => {
                let token = std::env::var("ZEDAZO_AUTH_TOKEN").unwrap_or_default();
                if token.is_empty() {
                    anyhow::bail!("ZEDAZO_AUTH_MODE=token requiere ZEDAZO_AUTH_TOKEN no vacío");
                }
                Ok(Self {
                    mode: AuthMode::Token(token),
                    cookie_secure,
                })
            }
            other => anyhow::bail!("ZEDAZO_AUTH_MODE desconocido: {other} (disabled|token)"),
        }
    }

    pub fn expected_token(&self) -> Option<&str> {
        match &self.mode {
            AuthMode::Disabled => None,
            AuthMode::Token(t) => Some(t.as_str()),
        }
    }
}

/// Abortar si auth disabled fuera de loopback (salvo override explícito Docker local).
pub fn validate_bind_auth(bind: &str, auth: &AuthConfig) -> anyhow::Result<()> {
    if matches!(auth.mode, AuthMode::Disabled) && !bind_is_loopback(bind) {
        let allow = std::env::var("ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK")
            .ok()
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        if !allow {
            anyhow::bail!(
                "ZEDAZO_AUTH_MODE=disabled solo permitido con bind loopback (ZEDAZO_BIND={bind}). \
                 En Docker local con puertos publicados solo a 127.0.0.1, define \
                 ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK=true"
            );
        }
        tracing::warn!(
            bind,
            "auth disabled en bind no-loopback (ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK)"
        );
    }
    Ok(())
}

pub fn bind_is_loopback(bind: &str) -> bool {
    if let Some(rest) = bind.strip_prefix('[') {
        if let Some((host, _)) = rest.split_once(']') {
            return matches!(host, "::1" | "0:0:0:0:0:0:0:1");
        }
    }
    let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or(bind);
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

fn tokens_equal(a: &str, b: &str) -> bool {
    let ha = Sha256::digest(a.as_bytes());
    let hb = Sha256::digest(b.as_bytes());
    ha.iter()
        .zip(hb.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

pub fn extract_credential(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let bearer = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "));
        if let Some(t) = bearer {
            let t = t.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    cookie_value(headers, COOKIE_NAME)
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in cookie.split(';') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix(name) {
            if let Some(val) = rest.strip_prefix('=') {
                return Some(val.to_string());
            }
        }
    }
    None
}

pub fn request_authorized(headers: &HeaderMap, auth: &AuthConfig) -> bool {
    match &auth.mode {
        AuthMode::Disabled => true,
        AuthMode::Token(expected) => extract_credential(headers)
            .map(|c| tokens_equal(&c, expected))
            .unwrap_or(false),
    }
}

pub async fn require_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if request_authorized(req.headers(), &state.auth) {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
    }
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub token: String,
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginBody>) -> Response {
    let Some(expected) = state.auth.expected_token() else {
        return (StatusCode::SERVICE_UNAVAILABLE, "auth disabled").into_response();
    };
    if !tokens_equal(&body.token, expected) {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    }
    let mut res = StatusCode::NO_CONTENT.into_response();
    if let Ok(val) = HeaderValue::from_str(&set_cookie_header(
        &body.token,
        state.auth.cookie_secure,
        false,
    )) {
        res.headers_mut().insert(header::SET_COOKIE, val);
    }
    res
}

pub async fn logout(State(state): State<AppState>) -> Response {
    let mut res = StatusCode::NO_CONTENT.into_response();
    if let Ok(val) = HeaderValue::from_str(&set_cookie_header("", state.auth.cookie_secure, true)) {
        res.headers_mut().insert(header::SET_COOKIE, val);
    }
    res
}

fn set_cookie_header(value: &str, secure: bool, clear: bool) -> String {
    let mut parts = vec![
        format!("{COOKIE_NAME}={value}"),
        "Path=/".into(),
        "HttpOnly".into(),
        "SameSite=Strict".into(),
    ];
    if secure {
        parts.push("Secure".into());
    }
    if clear {
        parts.push("Max-Age=0".into());
    }
    parts.join("; ")
}

pub fn cors_layer_for(auth: &AuthConfig) -> CorsLayer {
    match &auth.mode {
        // Espejar origen + credentials: el cliente GUI usa `credentials: "include"`.
        // No usar AllowOrigin::any ni Allow-Headers: * (incompatibles con credentials).
        AuthMode::Disabled => CorsLayer::new()
            .allow_origin(AllowOrigin::mirror_request())
            .allow_methods(cors_allow_methods())
            .allow_headers(cors_allow_headers())
            .allow_credentials(true),
        AuthMode::Token(_) => {
            let origins = std::env::var("ZEDAZO_CORS_ORIGIN").unwrap_or_default();
            if origins.trim().is_empty() {
                CorsLayer::new()
            } else {
                let list: Vec<HeaderValue> = origins
                    .split(',')
                    .filter_map(|o| HeaderValue::from_str(o.trim()).ok())
                    .collect();
                if list.is_empty() {
                    CorsLayer::new()
                } else {
                    CorsLayer::new()
                        .allow_origin(list)
                        .allow_methods(cors_allow_methods())
                        .allow_headers(cors_allow_headers())
                        .allow_credentials(true)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_bind_detection() {
        assert!(bind_is_loopback("127.0.0.1:8080"));
        assert!(bind_is_loopback("localhost:8080"));
        assert!(bind_is_loopback("[::1]:8080"));
        assert!(!bind_is_loopback("0.0.0.0:8080"));
        assert!(!bind_is_loopback("192.168.1.10:8080"));
    }

    #[test]
    fn fail_closed_disabled_on_public_bind() {
        let auth = AuthConfig::disabled();
        assert!(validate_bind_auth("127.0.0.1:8080", &auth).is_ok());
        assert!(validate_bind_auth("0.0.0.0:8080", &auth).is_err());
        std::env::set_var("ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK", "true");
        assert!(validate_bind_auth("0.0.0.0:8080", &auth).is_ok());
        std::env::remove_var("ZEDAZO_AUTH_ALLOW_DISABLED_NON_LOOPBACK");
    }
}
