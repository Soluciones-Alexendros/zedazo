//! Cliente HTTP CardDAV (reqwest + rustls, TLS 1.2+).
//!
//! Un request en vuelo por colección. Reintenta 429/503 honrando `Retry-After`.

use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::redirect::Policy;
use reqwest::{Method, StatusCode, Url};

use crate::config::CardDavConfig;
use crate::error::{CardDavError, CardDavResult};
use crate::filter::vcard_matches_categories;
use crate::url_policy::{
    ensure_trailing_slash, origin_with_slash, resolve_href, resolve_redirect, same_collection,
    validate_server_url,
};
use crate::xml::{
    looks_like_vcard, parse_multistatus, propfind_addressbook_home_set, propfind_collection,
    propfind_current_user_principal, propfind_sync_state,
};

const MAX_REDIRECTS: usize = 8;
const MAX_RETRIES: u32 = 4;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

struct Outbound<'a> {
    method: Method,
    url: Url,
    depth: Option<&'a str>,
    body: Option<String>,
    allow_cross_host: bool,
    send_auth: bool,
    extra_headers: Vec<(String, String)>,
}

/// Addressbook remoto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addressbook {
    pub url: Url,
    pub displayname: Option<String>,
    pub etag: Option<String>,
    pub ctag: Option<String>,
    pub sync_token: Option<String>,
}

/// vCard descargado (cuerpo crudo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardObject {
    pub href: Url,
    pub etag: Option<String>,
    pub vcard: String,
}

/// Resultado de descubrimiento RFC 6764 + 6352.
#[derive(Debug, Clone)]
pub struct Discovery {
    pub dav_root: Url,
    pub principal: Option<Url>,
    pub home_set: Option<Url>,
    pub addressbooks: Vec<Addressbook>,
}

/// Resultado de un pull (lectura; el filtro de categoría es client-side).
#[derive(Debug, Clone)]
pub struct PullResult {
    pub addressbook: Addressbook,
    pub cards: Vec<CardObject>,
}

/// Estado de colección para watch (CTag / RFC 6578 sync-token / ETag).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncState {
    pub addressbook: Url,
    pub ctag: Option<String>,
    pub sync_token: Option<String>,
    pub etag: Option<String>,
}

impl SyncState {
    /// Huella estable: sync-token > CTag > ETag.
    pub fn fingerprint(&self) -> String {
        self.sync_token
            .clone()
            .or_else(|| self.ctag.clone())
            .or_else(|| self.etag.clone())
            .unwrap_or_default()
    }

    /// True si el token remoto cambió respecto de `previous`.
    pub fn changed_from(&self, previous: &Self) -> bool {
        self.fingerprint() != previous.fingerprint()
    }
}

/// Resultado de PUT/DELETE (nunca last-write-wins).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    pub href: Url,
    pub etag: Option<String>,
    pub status: u16,
}

impl PullResult {
    /// Concatena los vCards en un único documento VCF.
    pub fn concatenated_vcf(&self) -> String {
        let mut out = String::new();
        for card in &self.cards {
            let body = card.vcard.trim();
            if body.is_empty() {
                continue;
            }
            out.push_str(body);
            if !body.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }
}

/// Cliente CardDAV (lectura, escritura opt-in, watch por polling).
pub struct CardDavClient {
    http: Client,
    config: CardDavConfig,
}

impl CardDavClient {
    /// Construye el cliente: rustls, TLS ≥ 1.2, sin `insecure-skip-verify`.
    pub fn new(config: CardDavConfig) -> CardDavResult<Self> {
        validate_server_url(&config.url)?;
        if let Some(ab) = &config.addressbook {
            validate_server_url(ab)?;
        }

        let http = Client::builder()
            .use_rustls_tls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .redirect(Policy::none())
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(format!("zedazo-carddav/{}", env!("CARGO_PKG_VERSION")))
            .https_only(false) // HTTP se filtra en url_policy (solo loopback).
            .build()?;

        Ok(Self { http, config })
    }

    /// Lista addressbooks (URL explícita o descubrimiento).
    pub fn list_addressbooks(&self) -> CardDavResult<Vec<Addressbook>> {
        if let Some(url) = &self.config.addressbook {
            return Ok(vec![self.describe_addressbook(url)?]);
        }
        Ok(self.discover()?.addressbooks)
    }

    /// Descubre DAV root, principal, home-set y addressbooks.
    pub fn discover(&self) -> CardDavResult<Discovery> {
        let dav_root = self.resolve_dav_root()?;
        tracing::info!(url = %dav_root, "carddav: DAV root");

        let principal = match self.current_user_principal(&dav_root) {
            Ok(url) => Some(url),
            Err(err) => {
                tracing::debug!("carddav: current-user-principal no disponible: {err}");
                None
            }
        };

        let home_probe = principal.as_ref().unwrap_or(&dav_root);
        let home_set = match self.addressbook_home_set(home_probe) {
            Ok(url) => Some(url),
            Err(err) => {
                tracing::debug!("carddav: addressbook-home-set no disponible: {err}");
                None
            }
        };

        let list_root = home_set.as_ref().unwrap_or(&dav_root);
        let mut addressbooks = self.list_addressbooks_at(list_root)?;
        if addressbooks.is_empty() {
            // El propio DAV root podría ser el addressbook.
            if let Ok(book) = self.describe_addressbook(list_root) {
                if book.displayname.is_some() || same_collection(list_root, &dav_root) {
                    addressbooks.push(book);
                }
            }
        }

        Ok(Discovery {
            dav_root,
            principal,
            home_set,
            addressbooks,
        })
    }

    /// Descarga vCards de un addressbook (GET secuencial).
    pub fn pull(&self, addressbook: Option<&Url>) -> CardDavResult<PullResult> {
        self.pull_filtered(addressbook, &[])
    }

    /// Pull y filtro client-side por categorías N1/N2 (`CATEGORIES` del vCard).
    pub fn pull_filtered(
        &self,
        addressbook: Option<&Url>,
        categories: &[String],
    ) -> CardDavResult<PullResult> {
        let mut books = self.list_addressbooks()?;
        let selected = match addressbook.or(self.config.addressbook.as_ref()) {
            Some(url) => books
                .into_iter()
                .find(|b| same_collection(&b.url, url))
                .unwrap_or(Addressbook {
                    url: ensure_trailing_slash(url),
                    displayname: None,
                    etag: None,
                    ctag: None,
                    sync_token: None,
                }),
            None => match books.len() {
                0 => {
                    return Err(CardDavError::NoAddressbook(self.config.url.to_string()));
                }
                1 => books.remove(0),
                n => {
                    return Err(CardDavError::AmbiguousAddressbook { count: n });
                }
            },
        };

        let mut cards = self.download_cards(&selected.url)?;
        if !categories.is_empty() {
            cards.retain(|c| vcard_matches_categories(&c.vcard, categories));
        }
        Ok(PullResult {
            addressbook: selected,
            cards,
        })
    }

    /// PUT de actualización: `If-Match` con ETag concreto. HTTP 412 = conflicto.
    pub fn put(&self, href: &Url, vcard: &str, etag: &str) -> CardDavResult<WriteResult> {
        let etag = require_concrete_etag(etag)?;
        validate_server_url(href)?;
        let if_match = quote_etag(&etag);
        self.write_object(
            Method::PUT,
            href.clone(),
            Some(vcard.to_string()),
            vec![
                ("If-Match".into(), if_match.clone()),
                ("Content-Type".into(), "text/vcard; charset=utf-8".into()),
            ],
            &if_match,
        )
    }

    /// PUT de alta: `If-None-Match: *` (falla si el href ya existe).
    pub fn create(&self, href: &Url, vcard: &str) -> CardDavResult<WriteResult> {
        validate_server_url(href)?;
        self.write_object(
            Method::PUT,
            href.clone(),
            Some(vcard.to_string()),
            vec![
                ("If-None-Match".into(), "*".into()),
                ("Content-Type".into(), "text/vcard; charset=utf-8".into()),
            ],
            "*",
        )
    }

    /// DELETE con `If-Match`. HTTP 412 = conflicto; nunca borra a ciegas.
    pub fn delete(&self, href: &Url, etag: &str) -> CardDavResult<WriteResult> {
        let etag = require_concrete_etag(etag)?;
        validate_server_url(href)?;
        let if_match = quote_etag(&etag);
        self.write_object(
            Method::DELETE,
            href.clone(),
            None,
            vec![("If-Match".into(), if_match.clone())],
            &if_match,
        )
    }

    /// CTag / sync-token / ETag de la colección (un PROPFIND).
    pub fn collection_sync_state(&self, addressbook: Option<&Url>) -> CardDavResult<SyncState> {
        let url = match addressbook.or(self.config.addressbook.as_ref()) {
            Some(u) => ensure_trailing_slash(u),
            None => {
                let mut books = self.list_addressbooks()?;
                match books.len() {
                    0 => return Err(CardDavError::NoAddressbook(self.config.url.to_string())),
                    1 => ensure_trailing_slash(&books.remove(0).url),
                    n => return Err(CardDavError::AmbiguousAddressbook { count: n }),
                }
            }
        };
        validate_server_url(&url)?;
        let xml = self.propfind(&url, "0", propfind_sync_state())?;
        let responses = parse_multistatus(&xml)?;
        let resp = responses.into_iter().next().unwrap_or_default();
        Ok(SyncState {
            addressbook: url,
            ctag: resp.ctag,
            sync_token: resp.sync_token,
            etag: resp.etag,
        })
    }

    fn write_object(
        &self,
        method: Method,
        href: Url,
        body: Option<String>,
        extra_headers: Vec<(String, String)>,
        etag_for_error: &str,
    ) -> CardDavResult<WriteResult> {
        let resp = self.send_outbound(Outbound {
            method,
            url: href.clone(),
            depth: None,
            body,
            allow_cross_host: false,
            send_auth: true,
            extra_headers,
        })?;
        let status = resp.status();
        if status == StatusCode::PRECONDITION_FAILED {
            return Err(CardDavError::PreconditionFailed {
                url: href.to_string(),
                etag: etag_for_error.to_string(),
            });
        }
        if status == StatusCode::UNAUTHORIZED {
            return Err(CardDavError::Unauthorized(href.to_string()));
        }
        if !(status.is_success() || status.as_u16() == 204 || status.as_u16() == 201) {
            return Err(status_error(&resp, &href));
        }
        let new_etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(unquote_header_etag);
        Ok(WriteResult {
            href,
            etag: new_etag,
            status: status.as_u16(),
        })
    }

    fn resolve_dav_root(&self) -> CardDavResult<Url> {
        let given = ensure_trailing_slash(&self.config.url);
        let origin = origin_with_slash(&given)?;

        if given.path() != "/" && self.probe_dav(&given) {
            return Ok(given);
        }

        if let Some(well_known) = self.follow_well_known(&origin)? {
            if self.probe_dav(&well_known) {
                return Ok(ensure_trailing_slash(&well_known));
            }
        }

        let nextcloud = origin
            .join("remote.php/dav/")
            .map_err(|e| CardDavError::InvalidUrl(e.to_string()))?;
        if self.probe_dav(&nextcloud) {
            return Ok(nextcloud);
        }

        if self.probe_dav(&origin) {
            return Ok(origin);
        }

        // Último recurso: la URL que dio el operador.
        Ok(given)
    }

    fn follow_well_known(&self, origin: &Url) -> CardDavResult<Option<Url>> {
        let well_known = origin
            .join(".well-known/carddav")
            .map_err(|e| CardDavError::InvalidUrl(e.to_string()))?;
        match self.send_follow(
            Method::GET,
            well_known.clone(),
            None,
            None,
            true, // well-known: se permite cambio de host (p. ej. iCloud).
            true,
        ) {
            Ok(resp) => {
                let final_url = resp.url().clone();
                let status = resp.status();
                if status == StatusCode::NOT_FOUND && final_url.path().contains(".well-known") {
                    return Ok(None);
                }
                if status == StatusCode::NOT_FOUND {
                    return Ok(None);
                }
                // Tras redirección, 401/403/200/207 indican que el DAV root existe.
                if final_url.path().contains(".well-known") {
                    Ok(Some(ensure_trailing_slash(origin)))
                } else {
                    Ok(Some(ensure_trailing_slash(&final_url)))
                }
            }
            Err(CardDavError::HttpStatus { status: 404, .. }) => Ok(None),
            Err(err) => {
                tracing::debug!("carddav: well-known falló: {err}");
                Ok(None)
            }
        }
    }

    fn probe_dav(&self, url: &Url) -> bool {
        self.current_user_principal(url).is_ok()
            || self.propfind(url, "0", propfind_collection()).is_ok()
    }

    fn current_user_principal(&self, url: &Url) -> CardDavResult<Url> {
        let xml = self.propfind(url, "0", propfind_current_user_principal())?;
        let responses = parse_multistatus(&xml)?;
        let href = responses
            .iter()
            .find_map(|r| r.current_user_principal.as_deref())
            .ok_or_else(|| CardDavError::Protocol("current-user-principal ausente".into()))?;
        resolve_href(url, href)
    }

    fn addressbook_home_set(&self, url: &Url) -> CardDavResult<Url> {
        let xml = self.propfind(url, "0", propfind_addressbook_home_set())?;
        let responses = parse_multistatus(&xml)?;
        let href = responses
            .iter()
            .find_map(|r| r.addressbook_home_set.as_deref())
            .ok_or_else(|| CardDavError::Protocol("addressbook-home-set ausente".into()))?;
        resolve_href(url, href)
    }

    fn list_addressbooks_at(&self, home: &Url) -> CardDavResult<Vec<Addressbook>> {
        let home = ensure_trailing_slash(home);
        let xml = self.propfind(&home, "1", propfind_collection())?;
        let responses = parse_multistatus(&xml)?;
        let mut books = Vec::new();
        for resp in responses {
            if !resp.is_addressbook {
                continue;
            }
            let url = resolve_href(&home, &resp.href)?;
            if same_collection(&url, &home) {
                // El home-set a veces se declara a sí mismo; solo si es addressbook.
            }
            books.push(Addressbook {
                url: ensure_trailing_slash(&url),
                displayname: resp.displayname,
                etag: resp.etag,
                ctag: resp.ctag,
                sync_token: resp.sync_token,
            });
        }
        Ok(books)
    }

    fn describe_addressbook(&self, url: &Url) -> CardDavResult<Addressbook> {
        let url = ensure_trailing_slash(url);
        let xml = self.propfind(&url, "0", propfind_collection())?;
        let responses = parse_multistatus(&xml)?;
        let resp = responses.into_iter().next().unwrap_or_default();
        Ok(Addressbook {
            url,
            displayname: resp.displayname,
            etag: resp.etag,
            ctag: resp.ctag,
            sync_token: resp.sync_token,
        })
    }

    fn download_cards(&self, book: &Url) -> CardDavResult<Vec<CardObject>> {
        let book = ensure_trailing_slash(book);
        let xml = self.propfind(&book, "1", propfind_collection())?;
        let responses = parse_multistatus(&xml)?;
        let mut cards = Vec::new();
        for resp in responses {
            if !looks_like_vcard(&resp) {
                continue;
            }
            let href = resolve_href(&book, &resp.href)?;
            if same_collection(&href, &book) {
                continue;
            }
            let vcard = self.get_text(&href)?;
            if vcard.trim().is_empty() {
                continue;
            }
            cards.push(CardObject {
                href,
                etag: resp.etag,
                vcard,
            });
        }
        Ok(cards)
    }

    fn propfind(&self, url: &Url, depth: &str, body: &str) -> CardDavResult<String> {
        let method = Method::from_bytes(b"PROPFIND")
            .map_err(|_| CardDavError::Protocol("no se pudo construir PROPFIND".into()))?;
        let resp = self.send_follow(
            method,
            url.clone(),
            Some(depth),
            Some(body.to_string()),
            false,
            true,
        )?;
        expect_dav_ok(&resp, url)?;
        let text = resp.text()?;
        Ok(text)
    }

    fn get_text(&self, url: &Url) -> CardDavResult<String> {
        let resp = self.send_follow(Method::GET, url.clone(), None, None, false, true)?;
        if !resp.status().is_success() {
            return Err(status_error(&resp, url));
        }
        Ok(resp.text()?)
    }

    fn send_follow(
        &self,
        method: Method,
        url: Url,
        depth: Option<&str>,
        body: Option<String>,
        allow_cross_host: bool,
        send_auth: bool,
    ) -> CardDavResult<Response> {
        self.send_outbound(Outbound {
            method,
            url,
            depth,
            body,
            allow_cross_host,
            send_auth,
            extra_headers: vec![],
        })
    }

    fn send_outbound(&self, mut req: Outbound<'_>) -> CardDavResult<Response> {
        let start_host = req.url.host_str().map(str::to_string);
        let mut hops = 0usize;
        loop {
            validate_server_url(&req.url)?;
            let cross_host = start_host
                .as_deref()
                .zip(req.url.host_str())
                .is_some_and(|(a, b)| !a.eq_ignore_ascii_case(b));
            if cross_host && !req.allow_cross_host {
                return Err(CardDavError::Protocol(format!(
                    "redirección fuera de origen bloqueada: {}",
                    req.url
                )));
            }
            let use_auth = req.send_auth && !cross_host;
            let resp = self.send_with_retry(
                &req.method,
                &req.url,
                req.depth,
                req.body.as_deref(),
                use_auth,
                &req.extra_headers,
            )?;
            if !resp.status().is_redirection() {
                return Ok(resp);
            }
            hops += 1;
            if hops > MAX_REDIRECTS {
                return Err(CardDavError::Protocol(format!(
                    "demasiadas redirecciones desde {}",
                    req.url
                )));
            }
            let Some(loc) = resp.headers().get(reqwest::header::LOCATION) else {
                return Err(CardDavError::Protocol(format!(
                    "redirección sin Location desde {}",
                    req.url
                )));
            };
            let loc = loc
                .to_str()
                .map_err(|_| CardDavError::Protocol("Location no ASCII".into()))?;
            req.url = resolve_redirect(&req.url, loc, req.allow_cross_host)?;
            tracing::debug!(url = %req.url, "carddav: siguiendo redirección");
        }
    }

    fn send_with_retry(
        &self,
        method: &Method,
        url: &Url,
        depth: Option<&str>,
        body: Option<&str>,
        send_auth: bool,
        extra_headers: &[(String, String)],
    ) -> CardDavResult<Response> {
        let mut last_retry_after = None;
        for attempt in 0..MAX_RETRIES {
            let resp = self.dispatch(method, url, depth, body, send_auth, extra_headers)?;
            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE
            {
                let wait = retry_after_delay(resp.headers()).unwrap_or_else(|| backoff(attempt));
                last_retry_after = Some(wait);
                tracing::debug!(%url, %status, ?wait, attempt, "carddav: backoff");
                drop(resp);
                if attempt + 1 == MAX_RETRIES {
                    break;
                }
                thread::sleep(wait);
                continue;
            }
            return Ok(resp);
        }
        Err(CardDavError::RetriesExhausted {
            attempts: MAX_RETRIES,
            retry_after: last_retry_after,
        })
    }

    fn dispatch(
        &self,
        method: &Method,
        url: &Url,
        depth: Option<&str>,
        body: Option<&str>,
        send_auth: bool,
        extra_headers: &[(String, String)],
    ) -> CardDavResult<Response> {
        tracing::debug!(%url, method = %method, depth, "carddav: request");
        let mut req: RequestBuilder = self.http.request(method.clone(), url.clone());
        if send_auth {
            req = req.basic_auth(&self.config.username, Some(&self.config.password));
        }
        if let Some(depth) = depth {
            req = req.header("Depth", depth);
        }
        let has_content_type = extra_headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"));
        for (k, v) in extra_headers {
            req = req.header(k.as_str(), v.as_str());
        }
        if body.is_some() && !has_content_type {
            req = req.header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/xml; charset=utf-8"),
            );
        }
        if let Some(body) = body {
            req = req.body(body.to_string());
        }
        Ok(req.send()?)
    }
}

fn expect_dav_ok(resp: &Response, url: &Url) -> CardDavResult<()> {
    let status = resp.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(CardDavError::Unauthorized(url.to_string()));
    }
    if status.as_u16() == 207 || status.is_success() {
        return Ok(());
    }
    Err(status_error(resp, url))
}

fn status_error(resp: &Response, url: &Url) -> CardDavError {
    if resp.status() == StatusCode::UNAUTHORIZED {
        return CardDavError::Unauthorized(url.to_string());
    }
    if resp.status() == StatusCode::PRECONDITION_FAILED {
        return CardDavError::PreconditionFailed {
            url: url.to_string(),
            etag: String::new(),
        };
    }
    CardDavError::HttpStatus {
        status: resp.status().as_u16(),
        url: url.to_string(),
        detail: resp
            .status()
            .canonical_reason()
            .unwrap_or("sin motivo")
            .to_string(),
    }
}

fn require_concrete_etag(etag: &str) -> CardDavResult<String> {
    let t = etag.trim();
    if t.is_empty() || t == "*" {
        return Err(CardDavError::MissingEtag);
    }
    Ok(unquote_header_etag(t))
}

fn quote_etag(etag: &str) -> String {
    let t = etag.trim();
    if t.starts_with('"') || t.starts_with("W/") || t.starts_with("w/") {
        t.to_string()
    } else {
        format!("\"{t}\"")
    }
}

fn unquote_header_etag(raw: &str) -> String {
    let t = raw.trim();
    let rest = if let Some(stripped) = t.strip_prefix("W/").or_else(|| t.strip_prefix("w/")) {
        stripped.trim()
    } else {
        t
    };
    rest.trim_matches('"').to_string()
}

fn retry_after_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let raw = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    let secs: u64 = raw.trim().parse().ok()?;
    Some(Duration::from_secs(secs.min(30)))
}

fn backoff(attempt: u32) -> Duration {
    let ms = 200u64.saturating_mul(1u64 << attempt.min(5));
    Duration::from_millis(ms.min(5_000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    #[test]
    fn retry_after_integer_seconds_capped() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("5"));
        assert_eq!(retry_after_delay(&headers), Some(Duration::from_secs(5)));

        headers.insert(RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(retry_after_delay(&headers), Some(Duration::from_secs(30)));

        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_static("Wed, 21 Oct 2015 07:28:00 GMT"),
        );
        assert_eq!(retry_after_delay(&headers), None);
    }

    #[test]
    fn backoff_grows_and_caps() {
        assert_eq!(backoff(0), Duration::from_millis(200));
        assert_eq!(backoff(1), Duration::from_millis(400));
        assert!(backoff(8) <= Duration::from_millis(5_000));
    }

    #[test]
    fn etag_helpers_reject_wildcard() {
        assert!(matches!(
            require_concrete_etag("*"),
            Err(CardDavError::MissingEtag)
        ));
        assert!(matches!(
            require_concrete_etag("  "),
            Err(CardDavError::MissingEtag)
        ));
        assert_eq!(require_concrete_etag("\"abc\"").unwrap(), "abc");
        assert_eq!(quote_etag("abc"), "\"abc\"");
        assert_eq!(quote_etag("\"abc\""), "\"abc\"");
    }

    #[test]
    fn sync_state_detects_token_change() {
        let book = Url::parse("http://127.0.0.1/book/").unwrap();
        let a = SyncState {
            addressbook: book.clone(),
            ctag: Some("1".into()),
            sync_token: None,
            etag: None,
        };
        let b = SyncState {
            addressbook: book,
            ctag: Some("2".into()),
            sync_token: None,
            etag: None,
        };
        assert!(b.changed_from(&a));
        assert!(!a.changed_from(&a));
    }
}
