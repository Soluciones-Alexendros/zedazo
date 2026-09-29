//! Watch: polling de CTag / sync-token; un request en vuelo.

mod common;

use common::MockHttp;
use zedazo_carddav::config::CardDavConfig;
use zedazo_carddav::{parse_and_validate, CardDavClient};

const USER: &str = "ada";
const PASS: &str = "app-password";
const BOOK: &str = "/remote.php/dav/addressbooks/users/ada/contacts/";

fn sync_xml(ctag: &str, token: &str) -> String {
    format!(
        r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/">
  <d:response>
    <d:href>{BOOK}</d:href>
    <d:propstat>
      <d:prop>
        <d:getetag>"col-etag"</d:getetag>
        <cs:getctag>{ctag}</cs:getctag>
        <d:sync-token>{token}</d:sync-token>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#
    )
}

fn client_for(server: &MockHttp) -> CardDavClient {
    let url = parse_and_validate(&server.base_url()).expect("loopback");
    let addressbook = parse_and_validate(&format!("{}{BOOK}", server.base_url())).ok();
    CardDavClient::new(CardDavConfig {
        url,
        username: USER.into(),
        password: PASS.into(),
        addressbook,
    })
    .unwrap()
}

#[test]
fn watch_detects_sync_token_and_ctag_change() {
    let server = MockHttp::start();
    server.mock_once(
        "PROPFIND",
        BOOK,
        207,
        &[("Content-Type", "application/xml")],
        sync_xml("ctag-1", "https://example.test/sync/1"),
    );
    server.mock(
        "PROPFIND",
        BOOK,
        207,
        &[("Content-Type", "application/xml")],
        sync_xml("ctag-2", "https://example.test/sync/2"),
    );

    let client = client_for(&server);
    let first = client.collection_sync_state(None).unwrap();
    assert_eq!(first.ctag.as_deref(), Some("ctag-1"));
    assert_eq!(
        first.sync_token.as_deref(),
        Some("https://example.test/sync/1")
    );

    let second = client.collection_sync_state(None).unwrap();
    assert_eq!(second.ctag.as_deref(), Some("ctag-2"));
    assert_eq!(
        second.sync_token.as_deref(),
        Some("https://example.test/sync/2")
    );
    assert!(second.changed_from(&first));
    assert_eq!(second.fingerprint(), "https://example.test/sync/2");
}
