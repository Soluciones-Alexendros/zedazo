//! S-05: logs/debug redactados; códigos sin valor TEL; manifest sin dígitos.

use tracing_test::traced_test;
use zedazo_api::errors::{error_code, redact_detail};

#[test]
fn redact_strips_tel_digits() {
    let s = redact_detail("parse TEL +34600000000 failed");
    assert!(!s.contains("34600"));
    assert!(!s.contains("34600000000"));
}

#[test]
fn error_code_parse() {
    let err = std::io::Error::new(std::io::ErrorKind::InvalidData, "parse vcard bad");
    assert_eq!(error_code(&err), "parse_error");
}

#[traced_test]
#[test]
fn tracing_redact_helper_usable() {
    let detail = redact_detail("contexto +34600000000");
    tracing::debug!(detail = %detail, "job fallido");
    assert!(!logs_contain("+34600"));
}

#[test]
fn manifest_error_is_code_not_tel_value() {
    // El worker guarda solo `error_code` en manifest.error (jobs/mod.rs).
    let err = std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "error de parseo en línea 3: TEL +34600000000",
    );
    let code = error_code(&err);
    assert_eq!(code, "parse_error");
    let manifest = serde_json::json!({ "error": code });
    let raw = manifest.to_string();
    assert!(!raw.contains("34600"), "{raw}");
    assert!(!raw.contains("+34600"), "{raw}");
}
