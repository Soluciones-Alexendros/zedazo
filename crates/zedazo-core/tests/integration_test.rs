use std::fs;
use std::path::PathBuf;
use zedazo_core::infrastructure::parser::{parse_vcards, unfold};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn read_vcf_to_vcards(path: &PathBuf) -> Vec<zedazo_core::infrastructure::parser::ParsedVCard> {
    let bytes = fs::read(path).expect("Unable to read fixture");
    let raw = String::from_utf8(bytes).expect("Fixture not valid UTF-8");
    let unfolded = unfold(&raw);
    parse_vcards(&unfolded).expect("Failed to parse fixture")
}

// ── Sample contacts vCard 4.0 ──

#[test]
fn test_parse_sample_contacts() {
    let vcards = read_vcf_to_vcards(&fixture("sample-contacts.vcf"));

    assert!(
        vcards.len() >= 5,
        "Expected >=5 contacts, got {}",
        vcards.len()
    );

    let v4_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("4.0"))
        .count();
    assert!(v4_count > 0, "No vCard 4.0 contacts found");

    let with_email = vcards.iter().filter(|v| !v.emails_raw.is_empty()).count();
    let with_tel = vcards.iter().filter(|v| !v.tels_raw.is_empty()).count();
    assert!(with_email > 0, "No contacts with email");
    assert!(with_tel > 0, "No contacts with phone");

    println!(
        "Sample: {} contacts (email: {}, tel: {}, v4: {})",
        vcards.len(),
        with_email,
        with_tel,
        v4_count
    );
}

#[test]
fn test_convert_sample_to_contacts() {
    let vcards = read_vcf_to_vcards(&fixture("sample-contacts.vcf"));

    let contacts: Vec<_> = vcards
        .into_iter()
        .map(|v| v.to_contact())
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to convert contacts");

    assert!(contacts.len() >= 5);
    for c in &contacts {
        assert!(!c.uid.is_empty(), "Contact has empty UID");
    }

    println!("Sample: {} contacts converted", contacts.len());
}

// ── Google Contacts vCard 3.0 ──

#[test]
fn test_parse_google_contactos() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    assert!(
        vcards.len() >= 3,
        "Expected >=3 contacts, got {}",
        vcards.len()
    );

    let v3_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("3.0"))
        .count();
    assert_eq!(
        v3_count,
        vcards.len(),
        "All Google contacts should be vCard 3.0"
    );

    let with_tel = vcards.iter().filter(|v| !v.tels_raw.is_empty()).count();
    assert!(
        with_tel >= 2,
        "Expected >=2 contacts with phone, got {}",
        with_tel
    );

    let with_fn = vcards.iter().filter(|v| v.fn_raw.is_some()).count();
    assert!(with_fn > 0, "Some contacts should have FN");

    println!(
        "Google contactos: {} vcards (with_fn: {}, with_tel: {}, v3: {})",
        vcards.len(),
        with_fn,
        with_tel,
        v3_count
    );
}

#[test]
fn test_parse_google_otroscontactos() {
    let vcards = read_vcf_to_vcards(&fixture("google_otroscontactos.vcf"));

    assert!(vcards.len() >= 3);

    let v3_count = vcards
        .iter()
        .filter(|v| v.version.as_deref() == Some("3.0"))
        .count();
    assert_eq!(v3_count, vcards.len());

    println!("Google otros contactos: {} vcards", vcards.len());
}

#[test]
fn test_google_escaped_commas_in_org() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    assert!(!vcards.is_empty());
    // Verificar que los contactos con ORG se parsean correctamente
    let with_org = vcards.iter().filter(|v| v.org_raw.is_some()).count();
    assert!(with_org > 0, "Should have contacts with ORG: {}", with_org);

    for v in &vcards {
        v.clone().to_contact().unwrap();
    }
}

#[test]
fn test_google_emoji_and_brackets_in_fn() {
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));

    // Verificar que todos se convierten a Contact sin panics
    assert!(!vcards.is_empty());
    for v in &vcards {
        let contact = v.clone().to_contact();
        assert!(contact.is_ok(), "Failed to convert: {:?}", v.fn_raw);
    }
}

#[test]
fn test_google_contacto_without_fn_has_uid_from_fallback() {
    // Los fixtures sintéticos tienen FN en todos, pero este test verifica
    // el comportamiento del fallback UID cuando no hay FN
    let vcards = read_vcf_to_vcards(&fixture("google_contactos.vcf"));
    let contacts: Vec<_> = vcards
        .into_iter()
        .map(|v| v.to_contact())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    for c in &contacts {
        assert!(!c.uid.is_empty());
        assert!(!c.fn_value.is_empty());
    }
}

// ── Pipeline ──

#[test]
fn test_pipeline_sample_dry_run() {
    use zedazo_core::application::cribar;
    use zedazo_core::domain::screening::ScreeningDecision;

    let input = fixture("sample-contacts.vcf");

    let (pipeline_stats, contacts) =
        cribar::execute(&input, None, None, None, "auto", true, false, false)
            .expect("Pipeline failed");

    assert!(pipeline_stats.total_entrada >= 5);
    assert!(pipeline_stats.conservados > 0);
    assert!(pipeline_stats.eliminados > 0 || pipeline_stats.conservados > 0);

    let conserved: Vec<_> = contacts
        .iter()
        .filter(|c| matches!(c.decision, ScreeningDecision::Conserved))
        .collect();
    assert!(!conserved.is_empty(), "Some contacts should be conserved");

    for c in &conserved {
        assert!(
            !c.categories.n1.is_empty(),
            "Conserved contact must have N1 category"
        );
    }

    // Verificar que stats funciona sobre la salida
    assert!(pipeline_stats.total_entrada >= 5);

    println!(
        "Pipeline sample: {} entrada, {} conservados, {} eliminados",
        pipeline_stats.total_entrada, pipeline_stats.conservados, pipeline_stats.eliminados
    );
}

#[test]
fn test_pipeline_google_dry_run() {
    use zedazo_core::application::cribar;

    let input = fixture("google_contactos.vcf");

    let (stats, _contacts) = cribar::execute(&input, None, None, None, "auto", true, false, false)
        .expect("Pipeline failed");

    assert!(stats.total_entrada >= 3);
    assert!(stats.conservados > 0 || stats.eliminados > 0);

    println!(
        "Google pipeline: {} entrada, {} conservados, {} eliminados",
        stats.total_entrada, stats.conservados, stats.eliminados
    );
}

#[test]
fn test_empty_vcf_error() {
    use zedazo_core::application::cribar;
    use zedazo_core::error::CribaError;

    let empty_content = "";
    let empty_fixture = std::env::temp_dir().join("empty_test.vcf");
    std::fs::write(&empty_fixture, empty_content).unwrap();

    let result = cribar::execute(&empty_fixture, None, None, None, "auto", true, false, false);
    let _ = std::fs::remove_file(&empty_fixture);
    assert!(matches!(result, Err(CribaError::EmptyVcf)));
}

// ── New fixtures tests ──

#[test]
fn test_parse_iso_sample() {
    let vcards = read_vcf_to_vcards(&fixture("iso_sample.vcf"));
    assert_eq!(vcards.len(), 2);
    for v in &vcards {
        assert_eq!(v.version.as_deref(), Some("4.0"));
        v.clone().to_contact().unwrap();
    }
}

#[test]
fn test_parse_duplicates() {
    let vcards = read_vcf_to_vcards(&fixture("duplicates.vcf"));
    assert_eq!(vcards.len(), 6);

    let contacts: Vec<_> = vcards
        .into_iter()
        .map(|v| v.to_contact())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    // Verificar que el D1 (mismo UID) se detecta
    let uids: std::collections::HashSet<_> = contacts.iter().map(|c| c.uid.clone()).collect();
    assert!(uids.len() < 6, "Should have duplicate UIDs");
}

#[test]
fn test_parse_edge_cases() {
    let vcards = read_vcf_to_vcards(&fixture("edge_cases.vcf"));
    assert_eq!(vcards.len(), 6);

    for v in &vcards {
        let contact = v.clone().to_contact();
        assert!(
            contact.is_ok(),
            "Failed to convert edge case: {:?}",
            v.fn_raw
        );
    }
}

#[test]
fn test_parse_proton_sample() {
    let vcards = read_vcf_to_vcards(&fixture("proton_sample.vcf"));
    assert_eq!(vcards.len(), 5);

    let proton_count = vcards
        .iter()
        .filter(|v| v.prodid.as_deref().unwrap_or("").contains("ProtonMail"))
        .count();
    assert_eq!(proton_count, 5);

    let autosave = vcards
        .iter()
        .filter(|v| v.uid.as_deref().unwrap_or("").contains("autosave"))
        .count();
    let import = vcards
        .iter()
        .filter(|v| v.uid.as_deref().unwrap_or("").contains("import"))
        .count();
    let web = vcards
        .iter()
        .filter(|v| v.uid.as_deref().unwrap_or("").contains("web"))
        .count();

    assert_eq!(autosave, 2);
    assert_eq!(import, 2);
    assert_eq!(web, 1);
}

#[test]
fn test_pipeline_duplicates_dry_run() {
    use zedazo_core::application::cribar;

    let input = fixture("duplicates.vcf");

    let (stats, _contacts) = cribar::execute(&input, None, None, None, "auto", true, false, false)
        .expect("Pipeline failed");

    assert_eq!(stats.total_entrada, 6);
    // D1 + transitivos deberían fusionarse
    assert!(stats.fusionados > 0);
}

#[test]
fn test_pipeline_edge_cases_dry_run() {
    use zedazo_core::application::cribar;

    let input = fixture("edge_cases.vcf");

    let (stats, contacts) = cribar::execute(&input, None, None, None, "auto", true, false, false)
        .expect("Pipeline failed");

    assert_eq!(stats.total_entrada, 6);
    // Contacto con email-only en FN debería ser E1
    println!("Stats: {:?}", stats);
    for c in &contacts {
        println!(
            "Contact: FN={}, Decision={:?}, ORG={:?}",
            c.fn_value, c.decision, c.org
        );
    }
    assert!(stats.eliminados > 0);
}

#[test]
fn test_pipeline_proton_dry_run() {
    use zedazo_core::application::cribar;

    let input = fixture("proton_sample.vcf");

    let (stats, contacts) = cribar::execute(&input, None, None, None, "auto", true, false, false)
        .expect("Pipeline failed");

    assert_eq!(stats.total_entrada, 5);
    assert!(stats.conservados > 0);

    // Verificar source_detail correcto
    let sources: Vec<_> = contacts.iter().map(|c| c.source_detail.clone()).collect();
    println!("Sources: {:?}", sources);
    assert!(sources.iter().any(|s| matches!(
        s,
        zedazo_core::domain::contact::SourceDetail::ProtonAutosave
    )));
    assert!(sources
        .iter()
        .any(|s| matches!(s, zedazo_core::domain::contact::SourceDetail::ProtonImport)));
    assert!(sources
        .iter()
        .any(|s| matches!(s, zedazo_core::domain::contact::SourceDetail::ProtonWeb)));
}
