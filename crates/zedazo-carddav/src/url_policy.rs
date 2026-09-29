//! Política de URL: HTTPS (TLS 1.2+) o HTTP solo a loopback (ADR-0018).

use std::net::IpAddr;

use reqwest::Url;

use crate::error::{CardDavError, CardDavResult};

/// Parsea y valida una URL de servidor CardDAV.
pub fn parse_and_validate(raw: &str) -> CardDavResult<Url> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(CardDavError::MissingUrl);
    }
    let url = Url::parse(trimmed).map_err(|e| CardDavError::InvalidUrl(e.to_string()))?;
    validate_server_url(&url)?;
    Ok(url)
}

/// HTTPS siempre; HTTP claro solo si el host es loopback.
pub fn validate_server_url(url: &Url) -> CardDavResult<()> {
    match url.scheme() {
        "https" => {
            if url.host_str().is_none() {
                return Err(CardDavError::InvalidUrl("https requiere un host".into()));
            }
            Ok(())
        }
        "http" => {
            let host = url.host_str().unwrap_or("");
            if is_loopback_host(host) {
                Ok(())
            } else {
                Err(CardDavError::CleartextForbidden)
            }
        }
        other => Err(CardDavError::UnsupportedScheme(other.to_string())),
    }
}

/// Host loopback: `localhost`, `127.0.0.0/8`, `::1`.
pub fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    let unbracketed = host.trim_matches(|c| c == '[' || c == ']');
    match unbracketed.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}

/// Origen `scheme://host[:port]` con barra final.
pub fn origin_with_slash(url: &Url) -> CardDavResult<Url> {
    let mut origin = url.origin().ascii_serialization();
    if !origin.ends_with('/') {
        origin.push('/');
    }
    Url::parse(&origin).map_err(|e| CardDavError::InvalidUrl(e.to_string()))
}

/// Asegura barra final en colecciones DAV.
pub fn ensure_trailing_slash(url: &Url) -> Url {
    let mut s = url.to_string();
    if s.ends_with('/') {
        url.clone()
    } else {
        s.push('/');
        Url::parse(&s).unwrap_or_else(|_| url.clone())
    }
}

/// Une un `href` DAV al origen de `base`.
///
/// Los hrefs absolutos o protocol-relative (`//host/…`) que cambian de origen
/// se rechazan: un 207 malicioso no debe reenviar Basic a loopback u otro host.
pub fn resolve_href(base: &Url, href: &str) -> CardDavResult<Url> {
    resolve_href_inner(base, href, false)
}

/// Une un `Location` de redirección. Si `allow_cross_origin` es true (solo
/// `/.well-known/carddav`), se admite cambio de host HTTPS (p. ej. iCloud).
pub fn resolve_redirect(
    base: &Url,
    location: &str,
    allow_cross_origin: bool,
) -> CardDavResult<Url> {
    resolve_href_inner(base, location, allow_cross_origin)
}

fn resolve_href_inner(base: &Url, href: &str, allow_cross_origin: bool) -> CardDavResult<Url> {
    let href = href.trim();
    if href.is_empty() {
        return Err(CardDavError::Protocol("href DAV vacío".into()));
    }
    let joined = base
        .join(href)
        .map_err(|e| CardDavError::InvalidUrl(e.to_string()))?;
    if same_origin(base, &joined) {
        return Ok(joined);
    }
    if !allow_cross_origin {
        return Err(CardDavError::CrossOrigin {
            from: origin_key(base),
            to: joined.to_string(),
        });
    }
    // well-known: HTTPS a un host no-loopback, o mismo tipo loopback.
    let from_loop = is_loopback_host(base.host_str().unwrap_or(""));
    let to_loop = is_loopback_host(joined.host_str().unwrap_or(""));
    if to_loop && !from_loop {
        return Err(CardDavError::CrossOrigin {
            from: origin_key(base),
            to: joined.to_string(),
        });
    }
    if joined.scheme() != "https" && !to_loop {
        return Err(CardDavError::CleartextForbidden);
    }
    Ok(joined)
}

/// Mismo esquema, host (ASCII case-insensitive) y puerto efectivo.
pub fn same_origin(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme()
        && a.host_str().map(str::to_ascii_lowercase) == b.host_str().map(str::to_ascii_lowercase)
        && a.port_or_known_default() == b.port_or_known_default()
}

fn origin_key(url: &Url) -> String {
    format!(
        "{}://{}{}",
        url.scheme(),
        url.host_str().unwrap_or(""),
        url.port().map(|p| format!(":{p}")).unwrap_or_default()
    )
}

/// Compara dos URLs de colección ignorando la barra final.
pub fn same_collection(a: &Url, b: &Url) -> bool {
    trim_slash(a.as_str()) == trim_slash(b.as_str())
}

fn trim_slash(s: &str) -> &str {
    s.trim_end_matches('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_remoto_ok() {
        parse_and_validate("https://cloud.example.test/remote.php/dav/").unwrap();
    }

    #[test]
    fn http_loopback_ok() {
        parse_and_validate("http://127.0.0.1:8080/dav/").unwrap();
        parse_and_validate("http://localhost/dav/").unwrap();
        parse_and_validate("http://[::1]/dav/").unwrap();
    }

    #[test]
    fn http_remoto_prohibido() {
        let err = parse_and_validate("http://cloud.example.test/dav/").unwrap_err();
        assert!(matches!(err, CardDavError::CleartextForbidden));
    }

    #[test]
    fn ftp_prohibido() {
        let err = parse_and_validate("ftp://127.0.0.1/dav").unwrap_err();
        assert!(matches!(err, CardDavError::UnsupportedScheme(_)));
    }

    #[test]
    fn resolve_path_absolute() {
        let base = Url::parse("https://cloud.example.test/remote.php/dav/").unwrap();
        let joined = resolve_href(&base, "/remote.php/dav/principals/users/ada/").unwrap();
        assert_eq!(
            joined.as_str(),
            "https://cloud.example.test/remote.php/dav/principals/users/ada/"
        );
    }

    #[test]
    fn resolve_href_rechaza_loopback_inyectado() {
        let base = Url::parse("https://cloud.example.test/remote.php/dav/").unwrap();
        let err = resolve_href(&base, "http://127.0.0.1:9/secret").unwrap_err();
        assert!(matches!(err, CardDavError::CrossOrigin { .. }));
        let err = resolve_href(&base, "//127.0.0.1/steal").unwrap_err();
        assert!(matches!(err, CardDavError::CrossOrigin { .. }));
        let err = resolve_href(&base, "https://evil.example.test/dav/").unwrap_err();
        assert!(matches!(err, CardDavError::CrossOrigin { .. }));
    }

    #[test]
    fn resolve_href_acepta_absoluto_mismo_origen() {
        let base = Url::parse("https://cloud.example.test/remote.php/dav/").unwrap();
        let joined = resolve_href(
            &base,
            "https://cloud.example.test/remote.php/dav/addressbooks/users/ada/",
        )
        .unwrap();
        assert!(joined.path().contains("/addressbooks/"));
    }

    #[test]
    fn well_known_redirect_https_ok_loopback_no() {
        let base = Url::parse("https://contacts.icloud.com/.well-known/carddav").unwrap();
        let ok = resolve_redirect(&base, "https://p01-contacts.icloud.com/", true).unwrap();
        assert_eq!(ok.host_str(), Some("p01-contacts.icloud.com"));
        let err = resolve_redirect(&base, "http://127.0.0.1:8080/", true).unwrap_err();
        assert!(matches!(
            err,
            CardDavError::CrossOrigin { .. } | CardDavError::CleartextForbidden
        ));
    }
}
