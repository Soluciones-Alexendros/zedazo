//! Tests para redacción de PII/secretos en logs (S-01).

use zedazo_api::observability::redact_value;

#[test]
fn redact_email() {
    let input = "Contact: user@example.com, phone: +34 600 123 456";
    let redacted = redact_value(input);
    assert!(!redacted.contains("user@example.com"));
    assert!(redacted.contains("[REDACTED]"));
    assert!(!redacted.contains("+34 600 123 456"));
}

#[test]
fn redact_marker() {
    let input = "Error processing MARKER_UID_123 and MARKER_EMAIL_456";
    let redacted = redact_value(input);
    assert!(!redacted.contains("MARKER_UID_123"));
    assert!(!redacted.contains("MARKER_EMAIL_456"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_token() {
    let input = "auth: token=abc123def456ghi789jkl012mno345pqr";
    let redacted = redact_value(input);
    assert!(!redacted.contains("abc123def456ghi789jkl012mno345pqr"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_bearer() {
    let input = "Authorization: bearer abc123def456";
    let redacted = redact_value(input);
    assert!(!redacted.contains("abc123def456"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_cookie() {
    let input = "cookie: zedazo_auth=abc123def456ghi789";
    let redacted = redact_value(input);
    assert!(!redacted.contains("abc123def456ghi789"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_authorization_header() {
    let input = r#"authorization: "Bearer abc123def456""#;
    let redacted = redact_value(input);
    assert!(!redacted.contains("abc123def456"));
    assert!(redacted.contains("[REDACTED]"));
}

#[test]
fn redact_phone_variants() {
    let inputs = [
        "+34 600 123 456",
        "+34600123456",
        "600 123 456",
        "(600) 123-456",
    ];
    for input in inputs {
        let redacted = redact_value(input);
        assert!(!redacted.contains(input), "input: {}", input);
        assert!(redacted.contains("[REDACTED]"), "input: {}", input);
    }
}

#[test]
fn redact_multiple_patterns() {
    let input =
        "user@example.com called +34 600 123 456 with token=abc123def456ghi789jkl012mno345pqr";
    let redacted = redact_value(input);
    let redacted_count = redacted.matches("[REDACTED]").count();
    assert_eq!(redacted_count, 3);
}

#[test]
fn redact_no_false_positive() {
    // Estos NO deberían ser redactados
    let safe_inputs = [
        "normal text without sensitive data",
        "uid-123-456",             // no es MARKER_
        "user_at_example_dot_com", // no es email válido
        "token_short",             // token muy corto
    ];
    for input in safe_inputs {
        let redacted = redact_value(input);
        assert_eq!(
            redacted, input,
            "input seguro redactado incorrectamente: {}",
            input
        );
    }
}

#[test]
fn redaction_layer_in_subscriber() {
    // Verificar que el layer se puede añadir al subscriber
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::registry;
    use zedazo_api::observability::RedactionLayer;

    let _subscriber = registry::Registry::default().with(RedactionLayer);
    // Si compila, el layer se integra correctamente
}
