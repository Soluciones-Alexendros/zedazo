//! Parseo mínimo de `multistatus` WebDAV/CardDAV (RFC 4918 / 6352).

use roxmltree::{Document, Node};

use crate::error::{CardDavError, CardDavResult};

const CARDDAV: &str = "urn:ietf:params:xml:ns:carddav";

/// Una entrada `<response>` de PROPFIND.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DavResponse {
    pub href: String,
    pub status: Option<u16>,
    pub displayname: Option<String>,
    pub etag: Option<String>,
    pub content_type: Option<String>,
    pub is_collection: bool,
    pub is_addressbook: bool,
    pub current_user_principal: Option<String>,
    pub addressbook_home_set: Option<String>,
    /// Calendar-server `getctag` (polling de colección).
    pub ctag: Option<String>,
    /// RFC 6578 `DAV:sync-token`.
    pub sync_token: Option<String>,
}

/// Parsea un documento `multistatus`.
pub fn parse_multistatus(xml: &str) -> CardDavResult<Vec<DavResponse>> {
    let doc = Document::parse(xml.trim_start_matches('\u{feff}'))
        .map_err(|e| CardDavError::Protocol(e.to_string()))?;
    let root = doc.root_element();
    if !local_is(root, "multistatus") {
        return Err(CardDavError::Protocol(format!(
            "se esperaba multistatus DAV, se obtuvo <{}>",
            root.tag_name().name()
        )));
    }

    let mut out = Vec::new();
    for node in root.children().filter(|n| n.is_element()) {
        if local_is(node, "response") {
            out.push(parse_response(node));
        }
    }
    Ok(out)
}

fn parse_response(response: Node<'_, '_>) -> DavResponse {
    let mut parsed = DavResponse {
        href: direct_child_text(response, "href").unwrap_or_default(),
        ..DavResponse::default()
    };

    for propstat in children_named(response, "propstat") {
        let status = children_named(propstat, "status")
            .find_map(|n| n.text())
            .and_then(parse_http_status);
        if parsed.status.is_none() {
            parsed.status = status;
        }
        if status.is_some_and(|s| s >= 400) {
            continue;
        }
        for prop in children_named(propstat, "prop") {
            merge_prop(&mut parsed, prop);
        }
    }

    // Algunos servidores ponen propiedades directamente bajo response.
    for prop in children_named(response, "prop") {
        merge_prop(&mut parsed, prop);
    }

    parsed
}

fn merge_prop(parsed: &mut DavResponse, prop: Node<'_, '_>) {
    for child in prop.children().filter(|n| n.is_element()) {
        let local = child.tag_name().name();
        match local {
            "displayname" => {
                parsed.displayname = node_text(child);
            }
            "getetag" => {
                parsed.etag = node_text(child).map(unquote_etag);
            }
            "getcontenttype" => {
                parsed.content_type = node_text(child);
            }
            "resourcetype" => {
                parsed.is_collection = has_child_named(child, "collection");
                parsed.is_addressbook = has_child_named(child, "addressbook")
                    || child.children().any(|n| {
                        n.is_element()
                            && n.tag_name().name() == "addressbook"
                            && n.tag_name().namespace() == Some(CARDDAV)
                    });
            }
            "current-user-principal" => {
                parsed.current_user_principal = nested_href(child);
            }
            "addressbook-home-set" => {
                parsed.addressbook_home_set = nested_href(child);
            }
            "getctag" => {
                parsed.ctag = node_text(child).map(unquote_etag);
            }
            "sync-token" => {
                parsed.sync_token = node_text(child);
            }
            _ => {}
        }
    }
}

fn nested_href(node: Node<'_, '_>) -> Option<String> {
    node.descendants()
        .find(|n| local_is(*n, "href"))
        .and_then(node_text)
}

fn children_named<'a, 'i>(
    node: Node<'a, 'i>,
    local: &'a str,
) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children()
        .filter(move |n| n.is_element() && local_is(*n, local))
}

fn direct_child_text(node: Node<'_, '_>, local: &str) -> Option<String> {
    children_named(node, local).find_map(node_text)
}

fn has_child_named(node: Node<'_, '_>, local: &str) -> bool {
    node.children()
        .any(|n| n.is_element() && n.tag_name().name().eq_ignore_ascii_case(local))
}

fn local_is(node: Node<'_, '_>, local: &str) -> bool {
    node.is_element() && node.tag_name().name().eq_ignore_ascii_case(local)
}

fn node_text(node: Node<'_, '_>) -> Option<String> {
    let t = node.text().unwrap_or("").trim();
    if t.is_empty() {
        // Concatenar hijos de texto (href con espacios).
        let joined: String = node
            .children()
            .filter_map(|n| n.text())
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        if joined.is_empty() {
            None
        } else {
            Some(joined)
        }
    } else {
        Some(t.to_string())
    }
}

fn parse_http_status(line: &str) -> Option<u16> {
    // "HTTP/1.1 200 OK"
    line.split_whitespace().nth(1)?.parse().ok()
}

fn unquote_etag(raw: String) -> String {
    raw.trim().trim_matches('"').to_string()
}

/// Cuerpos PROPFIND canónicos.
pub fn propfind_current_user_principal() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#
}

pub fn propfind_addressbook_home_set() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <c:addressbook-home-set/>
  </d:prop>
</d:propfind>"#
}

pub fn propfind_collection() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <d:getetag/>
    <d:getcontenttype/>
    <c:addressbook-description/>
    <cs:getctag xmlns:cs="http://calendarserver.org/ns/"/>
    <d:sync-token/>
  </d:prop>
</d:propfind>"#
}

/// PROPFIND acotado a CTag / sync-token / ETag de la colección.
pub fn propfind_sync_state() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/">
  <d:prop>
    <d:getetag/>
    <cs:getctag/>
    <d:sync-token/>
  </d:prop>
</d:propfind>"#
}

/// Comprueba si un objeto listado parece vCard.
pub fn looks_like_vcard(resp: &DavResponse) -> bool {
    if resp.is_collection || resp.is_addressbook {
        return false;
    }
    if resp.href.is_empty() || resp.href.ends_with('/') {
        return false;
    }
    let ct = resp
        .content_type
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    if ct.contains("vcard") {
        return true;
    }
    let href_l = resp.href.to_ascii_lowercase();
    href_l.ends_with(".vcf") || href_l.ends_with(".vcard") || ct.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
        <d:current-user-principal>
          <d:href>/remote.php/dav/principals/users/ada/</d:href>
        </d:current-user-principal>
        <card:addressbook-home-set>
          <d:href>/remote.php/dav/addressbooks/users/ada/</d:href>
        </card:addressbook-home-set>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/contacts/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Contactos</d:displayname>
        <d:getetag>"abc123"</d:getetag>
        <cs:getctag xmlns:cs="http://calendarserver.org/ns/">ctag-9</cs:getctag>
        <d:sync-token>https://example.test/sync/9</d:sync-token>
        <d:resourcetype>
          <d:collection/>
          <card:addressbook/>
        </d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/contacts/ada.vcf</d:href>
    <d:propstat>
      <d:prop>
        <d:getcontenttype>text/vcard; charset=utf-8</d:getcontenttype>
        <d:getetag>"card-1"</d:getetag>
        <d:resourcetype/>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

    #[test]
    fn parse_principal_home_addressbook_and_card() {
        let all = parse_multistatus(SAMPLE).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(
            all[0].current_user_principal.as_deref(),
            Some("/remote.php/dav/principals/users/ada/")
        );
        assert_eq!(
            all[0].addressbook_home_set.as_deref(),
            Some("/remote.php/dav/addressbooks/users/ada/")
        );
        assert!(all[1].is_addressbook);
        assert_eq!(all[1].displayname.as_deref(), Some("Contactos"));
        assert_eq!(all[1].etag.as_deref(), Some("abc123"));
        assert_eq!(all[1].ctag.as_deref(), Some("ctag-9"));
        assert_eq!(
            all[1].sync_token.as_deref(),
            Some("https://example.test/sync/9")
        );
        assert!(looks_like_vcard(&all[2]));
        assert!(!looks_like_vcard(&all[1]));
    }

    #[test]
    fn rejects_non_multistatus() {
        let err = parse_multistatus("<html>no</html>").unwrap_err();
        assert!(matches!(err, CardDavError::Protocol(_)));
    }
}
