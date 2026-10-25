//! Códigos de error API (S-05): sin valores sensibles en manifest/respuesta.

/// Clasifica un error de dominio/IO en código estable.
pub fn error_code(err: &dyn std::error::Error) -> &'static str {
    let msg = err.to_string().to_ascii_lowercase();
    if msg.contains("parse") || msg.contains("vcard") || msg.contains("sintaxis") {
        "parse_error"
    } else if msg.contains("cancel") || msg.contains("interrump") {
        "cancelled"
    } else if msg.contains("no space") || msg.contains("enospc") || msg.contains("disco") {
        "disk_full"
    } else if msg.contains("config") {
        "config_error"
    } else {
        "io_error"
    }
}

/// Redacta secuencias de ≥6 dígitos (p. ej. TEL) para logs debug.
pub fn redact_detail(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut digit_buf = String::new();
    let flush_digits = |buf: &mut String, out: &mut String| {
        if buf.len() >= 6 {
            out.push_str("[redacted]");
        } else {
            out.push_str(buf);
        }
        buf.clear();
    };
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            digit_buf.push(ch);
        } else {
            if ch == '+' && digit_buf.is_empty() {
                digit_buf.push(ch);
                continue;
            }
            if digit_buf.starts_with('+') && digit_buf.len() == 1 && !ch.is_ascii_digit() {
                out.push('+');
                digit_buf.clear();
            } else {
                flush_digits(&mut digit_buf, &mut out);
            }
            out.push(ch);
        }
    }
    flush_digits(&mut digit_buf, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_phone() {
        let s = redact_detail("fallo en TEL +34600000000 resto");
        assert!(!s.contains("34600000000"), "{s}");
        assert!(s.contains("[redacted]"), "{s}");
    }
}
