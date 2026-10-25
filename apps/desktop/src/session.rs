//! Token de sesión loopback (32 B CSPRNG → hex).

use getrandom::getrandom;

/// Genera un token opaco de 32 bytes (64 hex). Solo para inyección al webview.
pub fn generate_session_token() -> anyhow::Result<String> {
    let mut bytes = [0u8; 32];
    getrandom(&mut bytes).map_err(|e| anyhow::anyhow!("CSPRNG sesión: {e}"))?;
    Ok(hex::encode(bytes))
}

fn is_session_token_hex(token_hex: &str) -> bool {
    token_hex.len() == 64 && token_hex.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Script de inicialización del webview: Bearer disponible para fetch (no EventSource).
/// Rechaza tokens no-hex (evita inyección JS si se reutiliza la API).
pub fn init_script(token_hex: &str) -> anyhow::Result<String> {
    if !is_session_token_hex(token_hex) {
        anyhow::bail!("token de sesión debe ser exactamente 32 B en hex (64 chars)");
    }
    Ok(format!(
        r#"(function(){{
  Object.defineProperty(window, "__ZEDAZO_SESSION_TOKEN__", {{
    value: "{token_hex}",
    writable: false,
    configurable: false,
    enumerable: false
  }});
}})();"#
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_is_32_bytes_hex() {
        let t = generate_session_token().unwrap();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        let u = generate_session_token().unwrap();
        assert_ne!(t, u);
    }

    #[test]
    fn init_script_embeds_token() {
        let t = "ab".repeat(32);
        let s = init_script(&t).unwrap();
        assert!(s.contains(&t));
        assert!(s.contains("__ZEDAZO_SESSION_TOKEN__"));
    }

    #[test]
    fn init_script_rejects_non_hex() {
        assert!(init_script("not-hex").is_err());
        assert!(init_script(&"a".repeat(63)).is_err());
        assert!(init_script(&format!("{}!", "a".repeat(63))).is_err());
    }
}
