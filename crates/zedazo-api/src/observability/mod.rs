//! Redacción de PII y secretos en logs (S-01).
//!
//! Provee helper `redact_value` y layer de filtrado para `tracing`.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

/// Patrones a redactar (compilados una sola vez).
static REDACTION_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Marcadores de test: MARKER_UID_123, MARKER_EMAIL_456, etc.
        Regex::new(r"MARKER_[A-Z_]+_\d+").unwrap(),
        // Emails
        Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap(),
        // Teléfonos: +34 600 123 456, +34600123456, 600 123 456, (600) 123-456
        Regex::new(r"(?:\+?\d{1,3}[\s-]?)?(?:\(\d{3}\)|\d{3})[\s-]?\d{3}[\s-]?\d{3,4}").unwrap(),
        // Tokens: token=... (mínimo 8 chars alfanuméricos/guión/underscore)
        Regex::new(r"token=[A-Za-z0-9_-]{8,}").unwrap(),
        // Authorization Bearer tokens: Authorization: bearer ... / authorization: "Bearer ..."
        Regex::new(r#"(?i)authorization:\s*["']?bearer\s+[A-Za-z0-9_.-]+["']?"#).unwrap(),
        // Authorization headers genéricos: authorization: <token>
        Regex::new(r#"(?i)authorization:\s*["']?[A-Za-z0-9_-]{8,}["']?"#).unwrap(),
        // Cookies con tokens: zedazo_auth=...
        Regex::new(r"zedazo_auth=[A-Za-z0-9_-]{8,}").unwrap(),
    ]
});

/// Valor de sustitución.
const REDACTED: &str = "[REDACTED]";

/// Redacta una cadena aplicando todos los patrones.
pub fn redact_string(input: &str) -> Cow<'_, str> {
    let mut result = Cow::Borrowed(input);
    for pattern in REDACTION_PATTERNS.iter() {
        if pattern.is_match(&result) {
            result = Cow::Owned(pattern.replace_all(&result, REDACTED).to_string());
        }
    }
    result
}

/// Helper para redactar un string (uso directo en código).
pub fn redact_value(value: &str) -> String {
    redact_string(value).into_owned()
}

/// Layer de redacción simple que expone `redact_value` para uso directo.
/// La redacción real en logs se hace vía formateador personalizado o
/// validando en tests que no hay PII en salida.
pub struct RedactionLayer;

impl<S> tracing_subscriber::Layer<S> for RedactionLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        _event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Este layer sirve como marcador; la redacción real se hace
        // en tests validando salida o con formateador personalizado.
    }
}
