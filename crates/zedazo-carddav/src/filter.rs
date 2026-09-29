//! Filtro client-side por taxonomía N1/N2 (ADR-0018).
//!
//! No usa `addressbook-query` RFC 6352. Empareja `CATEGORIES` del vCard
//! con códigos N1/N2 ya existentes. No reimplementa las reglas de dominio.

use crate::error::{CardDavError, CardDavResult};

/// N1 de la taxonomía vigente (`domain::rules` + defaults de clasificación).
pub const TAXONOMY_N1: &[&str] = &[
    "PROF", "INST", "SALUD", "FIN", "FORM", "TEC", "HOST", "TRAN", "INMO", "SERV", "ASOC", "PERS",
];

/// Normaliza un código de filtro (`prof-jud` → `PROF-JUD`) y valida la forma N1/N2.
pub fn parse_category_filter(raw: &str) -> CardDavResult<String> {
    let t = raw.trim().to_ascii_uppercase();
    if !is_taxonomy_code(&t) {
        return Err(CardDavError::UnknownCategory(raw.trim().to_string()));
    }
    Ok(t)
}

/// Código N1 (`PROF`) o N2 (`PROF-JUD`); opcional N3 (`JUD-JUZ`).
pub fn is_taxonomy_code(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    (1..=3).contains(&parts.len())
        && parts.iter().all(|p| {
            (2..=8).contains(&p.len())
                && p.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
}

/// Extrae `CATEGORIES` de un vCard (unfold RFC 6350; sin parseo de dominio).
pub fn extract_categories(vcard: &str) -> Vec<String> {
    let unfolded = unfold_vcard(vcard);
    let mut out = Vec::new();
    for line in unfolded.lines() {
        let trimmed = line.trim();
        let head = trimmed.to_ascii_uppercase();
        if !(head.starts_with("CATEGORIES:") || head.starts_with("CATEGORIES;")) {
            continue;
        }
        let Some((_, rest)) = trimmed.split_once(':') else {
            continue;
        };
        for part in rest.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                out.push(t.to_ascii_uppercase());
            }
        }
    }
    out
}

/// `want=PROF` coincide con `PROF` o `PROF-JUD` en el vCard.
/// `want=PROF-JUD` solo coincide con ese N2 (o un N3 debajo).
pub fn category_matches(have: &str, want: &str) -> bool {
    let h = have.trim().to_ascii_uppercase();
    let w = want.trim().to_ascii_uppercase();
    if h.is_empty() || w.is_empty() {
        return false;
    }
    h == w || h.starts_with(&format!("{w}-"))
}

/// El vCard encaja si alguna `CATEGORIES` coincide con algún filtro (OR).
pub fn vcard_matches_categories(vcard: &str, wanted: &[String]) -> bool {
    if wanted.is_empty() {
        return true;
    }
    let have = extract_categories(vcard);
    wanted
        .iter()
        .any(|w| have.iter().any(|h| category_matches(h, w)))
}

fn unfold_vcard(vcard: &str) -> String {
    let normalized = vcard.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::new();
    for line in normalized.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            out.push_str(line.trim_start());
        } else {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_normalizes_and_rejects_garbage() {
        assert_eq!(parse_category_filter("prof-jud").unwrap(), "PROF-JUD");
        assert_eq!(parse_category_filter(" FIN ").unwrap(), "FIN");
        assert!(matches!(
            parse_category_filter("not a category"),
            Err(CardDavError::UnknownCategory(_))
        ));
        assert!(matches!(
            parse_category_filter(""),
            Err(CardDavError::UnknownCategory(_))
        ));
    }

    #[test]
    fn extract_handles_fold_and_params() {
        let vcard = "BEGIN:VCARD\nCATEGORIES:PROF,\n PROF-JUD\nFN:Ada\nEND:VCARD\n";
        let cats = extract_categories(vcard);
        assert_eq!(cats, vec!["PROF", "PROF-JUD"]);

        let typed = "CATEGORIES;LANGUAGE=es:FIN,FIN-CRYPTO\n";
        assert_eq!(extract_categories(typed), vec!["FIN", "FIN-CRYPTO"]);
    }

    #[test]
    fn n1_filter_matches_n2_child_not_the_reverse() {
        assert!(category_matches("PROF-JUD", "PROF"));
        assert!(category_matches("PROF-JUD", "PROF-JUD"));
        assert!(!category_matches("PROF", "PROF-JUD"));
        assert!(!category_matches("FIN-CRYPTO", "PROF"));
    }

    #[test]
    fn vcard_or_filter() {
        let vcard = "BEGIN:VCARD\nCATEGORIES:FIN-CRYPTO\nEND:VCARD\n";
        assert!(vcard_matches_categories(
            vcard,
            &["PROF".into(), "FIN".into()]
        ));
        assert!(!vcard_matches_categories(vcard, &["PROF-JUD".into()]));
        assert!(vcard_matches_categories(vcard, &[]));
    }

    #[test]
    fn known_n1_are_taxonomy_codes() {
        for n1 in TAXONOMY_N1 {
            assert!(is_taxonomy_code(n1), "{n1}");
        }
    }
}
