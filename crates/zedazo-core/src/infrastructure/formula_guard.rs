//! Neutralización de fórmulas CSV/TSV (S-07 / OWASP CSV injection).

/// Prefija `'` si la celda empieza por `=`, `+`, `-`, `@`, tab o CR.
/// Excepción: TEL E.164 `^\+[1-9]\d{6,14}$` (no se neutraliza).
pub fn neutralize_cell(value: &str) -> String {
    if is_safe_e164_tel(value) {
        return value.to_string();
    }
    let needs = value
        .chars()
        .next()
        .is_some_and(|c| matches!(c, '=' | '+' | '-' | '@' | '\t' | '\r'));
    if needs {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

/// `^\+[1-9]\d{6,14}$` → 7..=15 dígitos tras `+`, el primero ≠ 0.
fn is_safe_e164_tel(value: &str) -> bool {
    let b = value.as_bytes();
    if !(8..=16).contains(&b.len()) || b[0] != b'+' {
        return false;
    }
    if !b[1].is_ascii_digit() || b[1] == b'0' {
        return false;
    }
    b[2..].iter().all(u8::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_dangerous_and_tel() {
        let cases: &[(&str, &str)] = &[
            ("=CMD()", "'=CMD()"),
            ("+1234", "'+1234"),
            ("-1+1", "'-1+1"),
            ("@SUM(A1)", "'@SUM(A1)"),
            ("\tTAB", "'\tTAB"),
            ("\rCR", "'\rCR"),
            // TEL E.164 válido: no prefijo
            ("+34600000000", "+34600000000"),
            ("+12025550123", "+12025550123"),
            // TEL que no cumple → sí se neutraliza
            ("+0123", "'+0123"),
            ("+12", "'+12"),
            ("++34600", "'++34600"),
            // Inocuo
            ("Juan", "Juan"),
            ("", ""),
            ("  spaced", "  spaced"),
        ];
        for &(input, expected) in cases {
            assert_eq!(neutralize_cell(input), expected, "input={input:?}");
        }
    }
}
