//! Write opt-in: PUT/DELETE + If-Match; HTTP 412 no sobrescribe.

mod common;

use common::MockHttp;
use zedazo_carddav::config::CardDavConfig;
use zedazo_carddav::{parse_and_validate, CardDavClient, CardDavError};

const USER: &str = "ada";
const PASS: &str = "app-password";
const CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada Example\r\nUID:urn:uuid:00000000-0000-4000-a000-000000000001\r\nEND:VCARD\r\n";
const HREF: &str = "/remote.php/dav/addressbooks/users/ada/contacts/ada.vcf";

fn client_for(server: &MockHttp) -> CardDavClient {
    let url = parse_and_validate(&server.base_url()).expect("loopback");
    CardDavClient::new(CardDavConfig {
        url,
        username: USER.into(),
        password: PASS.into(),
        addressbook: None,
    })
    .unwrap()
}

fn object_url(server: &MockHttp) -> reqwest::Url {
    parse_and_validate(&format!("{}{HREF}", server.base_url())).unwrap()
}

#[test]
fn put_if_match_success_sends_quoted_etag() {
    let server = MockHttp::start();
    server.mock_if_match("PUT", HREF, "etag-1", 204, &[("ETag", "\"etag-2\"")], b"");

    let client = client_for(&server);
    let href = object_url(&server);
    let written = client.put(&href, CARD, "etag-1").unwrap();
    assert_eq!(written.status, 204);
    assert_eq!(written.etag.as_deref(), Some("etag-2"));
    let sent = server.header_of_last("If-Match").expect("If-Match");
    assert_eq!(sent.trim_matches('"'), "etag-1");
}

#[test]
fn put_http_412_is_conflict_not_overwrite() {
    let server = MockHttp::start();
    server.mock_if_match("PUT", HREF, "etag-current", 204, &[], b"");

    let client = client_for(&server);
    let href = object_url(&server);
    let err = client.put(&href, CARD, "etag-stale").unwrap_err();
    assert!(
        matches!(err, CardDavError::PreconditionFailed { .. }),
        "esperado 412 tipado, obtuvo {err:?}"
    );
}

#[test]
fn delete_http_412_is_conflict() {
    let server = MockHttp::start();
    server.mock_if_match("DELETE", HREF, "etag-current", 204, &[], b"");

    let client = client_for(&server);
    let href = object_url(&server);
    let err = client.delete(&href, "viejo").unwrap_err();
    assert!(matches!(err, CardDavError::PreconditionFailed { .. }));
}

#[test]
fn delete_if_match_success() {
    let server = MockHttp::start();
    server.mock_if_match("DELETE", HREF, "etag-1", 204, &[], b"");
    let client = client_for(&server);
    let href = object_url(&server);
    let written = client.delete(&href, "etag-1").unwrap();
    assert_eq!(written.status, 204);
}

#[test]
fn put_rejects_wildcard_etag_without_http() {
    let server = MockHttp::start();
    let client = client_for(&server);
    let href = object_url(&server);
    let err = client.put(&href, CARD, "*").unwrap_err();
    assert!(matches!(err, CardDavError::MissingEtag));
    assert!(server.last_request().is_none());
}

#[test]
fn create_sends_if_none_match_star() {
    let server = MockHttp::start();
    server.mock("PUT", HREF, 201, &[("ETag", "\"new-1\"")], b"");
    let client = client_for(&server);
    let href = object_url(&server);
    let written = client.create(&href, CARD).unwrap();
    assert_eq!(written.status, 201);
    assert_eq!(server.header_of_last("If-None-Match").as_deref(), Some("*"));
}
