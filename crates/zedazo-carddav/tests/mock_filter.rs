//! Filtro client-side por categorías N1/N2 (sin addressbook-query).

mod common;

use common::MockHttp;
use zedazo_carddav::config::CardDavConfig;
use zedazo_carddav::{parse_and_validate, CardDavClient};

const USER: &str = "ada";
const PASS: &str = "app-password";
const BOOK: &str = "/remote.php/dav/addressbooks/users/ada/contacts/";

const PROF_CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Ada Juzgado\r\nCATEGORIES:PROF,PROF-JUD\r\nUID:urn:uuid:00000000-0000-4000-a000-0000000000aa\r\nEND:VCARD\r\n";
const FIN_CARD: &str = "BEGIN:VCARD\r\nVERSION:4.0\r\nFN:Banco Example\r\nCATEGORIES:FIN,FIN-CRYPTO\r\nUID:urn:uuid:00000000-0000-4000-a000-0000000000bb\r\nEND:VCARD\r\n";

fn listing_xml() -> &'static str {
    r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/contacts/</d:href>
    <d:propstat>
      <d:prop>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/contacts/ada.vcf</d:href>
    <d:propstat>
      <d:prop>
        <d:getcontenttype>text/vcard</d:getcontenttype>
        <d:getetag>"e1"</d:getetag>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/remote.php/dav/addressbooks/users/ada/contacts/banco.vcf</d:href>
    <d:propstat>
      <d:prop>
        <d:getcontenttype>text/vcard</d:getcontenttype>
        <d:getetag>"e2"</d:getetag>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#
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

fn mount(server: &MockHttp) {
    server.mock(
        "PROPFIND",
        BOOK,
        207,
        &[("Content-Type", "application/xml")],
        listing_xml(),
    );
    server.mock(
        "GET",
        "/remote.php/dav/addressbooks/users/ada/contacts/ada.vcf",
        200,
        &[("Content-Type", "text/vcard")],
        PROF_CARD,
    );
    server.mock(
        "GET",
        "/remote.php/dav/addressbooks/users/ada/contacts/banco.vcf",
        200,
        &[("Content-Type", "text/vcard")],
        FIN_CARD,
    );
}

#[test]
fn pull_category_filter_keeps_n1_and_n2_matches() {
    let server = MockHttp::start();
    mount(&server);
    let client = client_for(&server);

    let all = client.pull(None).unwrap();
    assert_eq!(all.cards.len(), 2);

    let only_prof = client.pull_filtered(None, &["PROF-JUD".into()]).unwrap();
    assert_eq!(only_prof.cards.len(), 1);
    assert!(only_prof.concatenated_vcf().contains("Ada Juzgado"));
    assert!(!only_prof.concatenated_vcf().contains("Banco Example"));

    let n1_fin = client.pull_filtered(None, &["FIN".into()]).unwrap();
    assert_eq!(n1_fin.cards.len(), 1);
    assert!(n1_fin.concatenated_vcf().contains("FIN-CRYPTO"));
}
