//! Detección de duplicados con Union-Find (DSU).
//!
//! Implementa cierre transitivo: si A coincide con B por TEL
//! y A coincide con C por EMAIL, entonces A, B y C se fusionan.
//!
//! D3–D6 (S-03): índices `HashMap` con tope por cubo (no O(n²) global).

use crate::domain::contact::Contact;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Tope de índices por cubo en D3–D6 (comparaciones ≤ CAP*(CAP-1)/2 por cubo).
pub const DEDUP_BUCKET_CAP: usize = 64;
/// Cada cuántas comparaciones de par se consulta cancelación/plazo.
const CANCEL_EVERY: usize = 256;

/// Señal de interrupción cooperativa en dedup (sin depender de `application`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DedupCancelled;

impl std::fmt::Display for DedupCancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dedup cancelado o plazo agotado")
    }
}

impl std::error::Error for DedupCancelled {}

/// Control cooperativo de dedup (cancelación + plazo opcional).
#[derive(Debug, Clone, Default)]
pub struct DedupCtrl {
    /// Si es `true`, aborta (compartido con el token de proceso).
    pub cancelled: Option<Arc<AtomicBool>>,
    pub deadline: Option<Instant>,
}

impl DedupCtrl {
    fn check(&self) -> Result<(), DedupCancelled> {
        if let Some(flag) = &self.cancelled {
            if flag.load(Ordering::SeqCst) {
                return Err(DedupCancelled);
            }
        }
        if let Some(dl) = self.deadline {
            if Instant::now() >= dl {
                return Err(DedupCancelled);
            }
        }
        Ok(())
    }
}

struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent[rb] = ra;
        }
    }
}

/// Normaliza FN para comparación de duplicados (lowercase, sin tildes, sin espacios extra).
fn normalize_for_dedup(fn_val: &str) -> String {
    fn_val
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' => 'u',
            'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Deduplica contactos usando Union-Find para D1 y D2.
///
/// Retorna los contactos fusionados y el número de fusiones realizadas.
/// La materialización usa `Vec<Option<Contact>>` con índices descendentes
/// para evitar invalidaciones.
pub fn deduplicate(contacts: Vec<Contact>) -> (Vec<Contact>, usize) {
    match deduplicate_with_ctrl(contacts, &DedupCtrl::default()) {
        Ok(v) => v,
        Err(_) => unreachable!("DedupCtrl::default no cancela"),
    }
}

/// Como [`deduplicate`], con cancelación/plazo (S-03).
pub fn deduplicate_with_ctrl(
    contacts: Vec<Contact>,
    ctrl: &DedupCtrl,
) -> Result<(Vec<Contact>, usize), DedupCancelled> {
    let n = contacts.len();
    let mut dsu = Dsu::new(n);
    ctrl.check()?;

    // ── D1: mismo UID ──
    let mut uid_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, c) in contacts.iter().enumerate() {
        if i % CANCEL_EVERY == 0 {
            ctrl.check()?;
        }
        let uid = c.uid.clone();
        if let Some(&prev) = uid_to_idx.get(&uid) {
            dsu.union(prev, i);
        } else {
            uid_to_idx.insert(uid, i);
        }
    }

    // ── D2: mismo FN + TEL  o  mismo FN + EMAIL ──
    let mut fn_tel_to_idx: HashMap<(String, String), usize> = HashMap::new();
    let mut fn_email_to_idx: HashMap<(String, String), usize> = HashMap::new();

    for (i, c) in contacts.iter().enumerate() {
        if i % CANCEL_EVERY == 0 {
            ctrl.check()?;
        }
        let fn_norm = normalize_for_dedup(&c.fn_value);

        for tel in &c.tels {
            if tel.normalized {
                let key = (fn_norm.clone(), tel.value.clone());
                if let Some(&prev) = fn_tel_to_idx.get(&key) {
                    dsu.union(prev, i);
                } else {
                    fn_tel_to_idx.insert(key, i);
                }
            }
        }

        for email in &c.emails {
            let key = (fn_norm.clone(), email.value.to_lowercase());
            if let Some(&prev) = fn_email_to_idx.get(&key) {
                dsu.union(prev, i);
            } else {
                fn_email_to_idx.insert(key, i);
            }
        }
    }

    // ── Materializar fusión: agrupar por raíz ──
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); n];
    for i in 0..n {
        groups[dsu.find(i)].push(i);
    }

    let mut merged: Vec<Option<Contact>> = contacts.into_iter().map(Some).collect();
    let mut fusion_count = 0;

    for indices in &groups {
        if indices.len() <= 1 {
            continue;
        }
        // Ordenar descendentemente: fusionar del último al primero
        let mut sorted = indices.clone();
        sorted.sort_unstable_by(|a, b| b.cmp(a));

        let Some(base_idx) = sorted.pop() else {
            continue;
        };
        for &other_idx in &sorted {
            let other = merged[other_idx].take().expect("contacto ya absorbido");
            if let Some(ref mut base) = merged[base_idx] {
                merge_contacts(base, other);
                fusion_count += 1;
            }
        }
    }

    let mut result: Vec<Contact> = merged.into_iter().flatten().collect();

    // ── D3-D6: candidatos a duplicado (notas, no fusión automática) ──
    let mut _cmp = 0u64;
    detect_candidates_indexed(&mut result, ctrl, &mut _cmp)?;

    Ok((result, fusion_count))
}

/// Clasifica un par con la misma prioridad que el algoritmo histórico.
fn rule_for_pair(a: &Contact, b: &Contact) -> Option<&'static str> {
    let fn_a = normalize_for_dedup(&a.fn_value);
    let fn_b = normalize_for_dedup(&b.fn_value);
    let org_a = a.org.as_deref().unwrap_or("").to_lowercase();
    let org_b = b.org.as_deref().unwrap_or("").to_lowercase();

    let same_fn = fn_a == fn_b;
    let same_org = !org_a.is_empty() && org_a == org_b;
    let shared_tel = a
        .tels
        .iter()
        .any(|ta| b.tels.iter().any(|tb| ta.value == tb.value));
    let shared_email = a.emails.iter().any(|ea| {
        b.emails
            .iter()
            .any(|eb| ea.value.to_lowercase() == eb.value.to_lowercase())
    });

    if same_fn && same_org {
        Some("D3")
    } else if shared_tel && !same_fn {
        Some("D4")
    } else if shared_email && !same_fn {
        Some("D5")
    } else if (shared_tel || shared_email) && !same_fn {
        Some("D6")
    } else {
        None
    }
}

fn pair_key(i: usize, j: usize) -> (usize, usize) {
    if i < j {
        (i, j)
    } else {
        (j, i)
    }
}

/// D3–D6 indexados (S-03): claves precalculadas + tope por cubo.
fn detect_candidates_indexed(
    contacts: &mut [Contact],
    ctrl: &DedupCtrl,
    comparisons: &mut u64,
) -> Result<(), DedupCancelled> {
    let n = contacts.len();
    if n < 2 {
        return Ok(());
    }

    let fns: Vec<String> = contacts
        .iter()
        .map(|c| normalize_for_dedup(&c.fn_value))
        .collect();
    let orgs: Vec<String> = contacts
        .iter()
        .map(|c| c.org.as_deref().unwrap_or("").to_lowercase())
        .collect();

    let mut by_fn: HashMap<String, Vec<usize>> = HashMap::new();
    let mut by_tel: HashMap<String, Vec<usize>> = HashMap::new();
    let mut by_email: HashMap<String, Vec<usize>> = HashMap::new();

    for (i, c) in contacts.iter().enumerate() {
        by_fn.entry(fns[i].clone()).or_default().push(i);
        for tel in &c.tels {
            by_tel.entry(tel.value.clone()).or_default().push(i);
        }
        for email in &c.emails {
            by_email
                .entry(email.value.to_lowercase())
                .or_default()
                .push(i);
        }
    }
    let _ = orgs;

    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut pending: Vec<(usize, usize, &'static str)> = Vec::new();

    let buckets = by_fn
        .into_values()
        .chain(by_tel.into_values())
        .chain(by_email.into_values());

    for idxs in buckets {
        if idxs.len() < 2 {
            continue;
        }
        let len = idxs.len().min(DEDUP_BUCKET_CAP);
        for a in 0..len {
            for b in (a + 1)..len {
                *comparisons += 1;
                if *comparisons % CANCEL_EVERY as u64 == 0 {
                    ctrl.check()?;
                }
                let i = idxs[a];
                let j = idxs[b];
                let key = pair_key(i, j);
                if !seen.insert(key) {
                    continue;
                }
                if let Some(rule) = rule_for_pair(&contacts[i], &contacts[j]) {
                    pending.push((i, j, rule));
                }
            }
        }
    }

    for (i, j, rule) in pending {
        let proposal_a = format!(
            "{} candidate: uid={} fn={}",
            rule, contacts[j].uid, contacts[j].fn_value
        );
        let proposal_b = format!(
            "{} candidate: uid={} fn={}",
            rule, contacts[i].uid, contacts[i].fn_value
        );
        append_note(&mut contacts[i], &proposal_a);
        append_note(&mut contacts[j], &proposal_b);
    }
    Ok(())
}

/// Algoritmo O(n²) histórico (solo tests de paridad).
#[cfg(test)]
fn detect_candidates_naive(contacts: &mut [Contact]) {
    let n = contacts.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if let Some(r) = rule_for_pair(&contacts[i], &contacts[j]) {
                let proposal_a = format!(
                    "{} candidate: uid={} fn={}",
                    r, contacts[j].uid, contacts[j].fn_value
                );
                let proposal_b = format!(
                    "{} candidate: uid={} fn={}",
                    r, contacts[i].uid, contacts[i].fn_value
                );
                append_note(&mut contacts[i], &proposal_a);
                append_note(&mut contacts[j], &proposal_b);
            }
        }
    }
}

fn append_note(contact: &mut Contact, text: &str) {
    match contact.note {
        Some(ref mut note) => {
            note.push_str(" | ");
            note.push_str(text);
        }
        None => contact.note = Some(text.into()),
    }
}

fn format_address_key(addr: &crate::domain::contact::Address) -> String {
    format!(
        "{};{};{};{};{};{};{};{}",
        addr.po_box,
        addr.extended,
        addr.street,
        addr.locality,
        addr.region,
        addr.postal_code,
        addr.country,
        addr.types.join(",")
    )
}

fn merge_contacts(base: &mut Contact, other: Contact) {
    base.merged_uids.push(other.uid);

    // FN: el más largo en tokens
    if other.fn_value.split_whitespace().count() > base.fn_value.split_whitespace().count() {
        base.fn_value = other.fn_value;
    }
    // structured_name: el que lo tenga
    if base.structured_name.is_none() && other.structured_name.is_some() {
        base.structured_name = other.structured_name;
    }
    // ORG: el más específico (más tokens)
    if let Some(ref other_org) = other.org {
        if base
            .org
            .as_ref()
            .map(|o| other_org.split_whitespace().count() > o.split_whitespace().count())
            .unwrap_or(true)
        {
            base.org = Some(other_org.clone());
        }
    }
    // TELs: unión dedup
    let mut seen_tels: std::collections::HashSet<String> =
        base.tels.iter().map(|t| t.value.clone()).collect();
    for tel in other.tels {
        if seen_tels.insert(tel.value.clone()) {
            base.tels.push(tel);
        }
    }
    // EMAILs: unión dedup
    let mut seen_emails: std::collections::HashSet<String> =
        base.emails.iter().map(|e| e.value.to_lowercase()).collect();
    for email in other.emails {
        if seen_emails.insert(email.value.to_lowercase()) {
            base.emails.push(email);
        }
    }
    // ADRs: unión dedup por valor completo
    let mut seen_adrs: std::collections::HashSet<String> =
        base.addresses.iter().map(format_address_key).collect();
    for addr in other.addresses {
        if seen_adrs.insert(format_address_key(&addr)) {
            base.addresses.push(addr);
        }
    }
    // TITLE, ROLE: unión
    if base.title.is_none() {
        base.title = other.title;
    }
    if base.role.is_none() {
        base.role = other.role;
    }
    // NOTE: concatenar
    if let Some(ref other_note) = other.note {
        match base.note {
            Some(ref mut bnote) => {
                bnote.push_str(" | ");
                bnote.push_str(other_note);
            }
            None => base.note = Some(other_note.clone()),
        }
    }
    // CATEGORIES: unión
    for n1 in other.categories.n1 {
        base.categories.n1.insert(n1);
    }
    for n2 in other.categories.n2 {
        base.categories.n2.insert(n2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contact::{CategorySet, SourceDetail, Tel, TelType, TypedValue};
    use crate::domain::screening::ScreeningDecision;

    fn make_contact(uid: &str, fn_val: &str, email: &str, tel_val: &str) -> Contact {
        Contact {
            uid: uid.into(),
            fn_value: fn_val.into(),
            emails: vec![TypedValue {
                value: email.into(),
                types: vec![],
                pref: 1,
            }],
            tels: if tel_val.is_empty() {
                vec![]
            } else {
                vec![Tel {
                    value: tel_val.into(),
                    tel_type: TelType::Cell,
                    normalized: true,
                }]
            },
            structured_name: None,
            org: None,
            org_fullname: None,
            org_legal_form: None,
            title: None,
            role: None,
            note: None,
            addresses: vec![],
            categories: CategorySet::default(),
            source_detail: SourceDetail::Unknown("test".into()),
            decision: ScreeningDecision::Conserved,
            screening_rule: String::new(),
            merged_uids: vec![],
            adr_field: None,
        }
    }

    #[test]
    fn test_d1_same_uid() {
        let a = make_contact("u1", "Alice", "a@x.com", "");
        let b = make_contact("u1", "Alice", "a2@x.com", "");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
        assert_eq!(result[0].merged_uids.len(), 1);
    }

    #[test]
    fn test_d2_same_fn_tel() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "a2@x.com", "+34600000001");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_d2_same_fn_email() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "a@x.com", "+34600000002");
        let (result, count) = deduplicate(vec![a, b]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 1);
        // emails duplicados no se duplican
        assert_eq!(result[0].emails.len(), 1);
    }

    #[test]
    fn test_d2_transitive() {
        // A ↔ B por TEL, A ↔ C por EMAIL → los tres se fusionan
        let a = make_contact("u1", "Alice", "shared@x.com", "+34600000001");
        let b = make_contact("u2", "Alice", "b@x.com", "+34600000001");
        let c = make_contact("u3", "Alice", "shared@x.com", "+34600000003");
        let (result, count) = deduplicate(vec![a, b, c]);
        assert_eq!(result.len(), 1);
        assert_eq!(count, 2);
        assert_eq!(result[0].merged_uids.len(), 2);
    }

    #[test]
    fn test_no_duplicates() {
        let a = make_contact("u1", "Alice", "a@x.com", "+34600000001");
        let b = make_contact("u2", "Bob", "b@x.com", "+34600000002");
        let c = make_contact("u3", "Carol", "c@x.com", "+34600000003");
        let (result, count) = deduplicate(vec![a, b, c]);
        assert_eq!(result.len(), 3);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_deduplicate_empty_input_no_panic() {
        // Regresión C-02: clusters vacíos / entrada vacía no deben panic.
        let (result, count) = deduplicate(vec![]);
        assert!(result.is_empty());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_normalize_for_dedup_tildes() {
        assert_eq!(normalize_for_dedup("María José"), "maria jose");
        assert_eq!(normalize_for_dedup("GARCÍA  López"), "garcia lopez");
    }

    fn notes_fingerprint(contacts: &[Contact]) -> HashSet<(String, String)> {
        contacts
            .iter()
            .filter_map(|c| c.note.as_ref().map(|n| (c.uid.clone(), n.clone())))
            .collect()
    }

    #[test]
    fn indexed_matches_naive_on_2k_planted() {
        // Corpus sintético 2k con duplicados plantados (example.invalid).
        let mut contacts = Vec::with_capacity(2000);
        for i in 0..1990 {
            contacts.push(make_contact(
                &format!("u{i}"),
                &format!("Person {i}"),
                &format!("p{i}@example.invalid"),
                &format!("+3461{:07}", i),
            ));
        }
        // Plantas D3 (mismo FN+ORG), D4 (TEL compartido), D5 (EMAIL compartido).
        let mut d3a = make_contact("d3a", "Same Name", "a@example.invalid", "+34620000001");
        d3a.org = Some("Acme Org".into());
        let mut d3b = make_contact("d3b", "Same Name", "b@example.invalid", "+34620000002");
        d3b.org = Some("Acme Org".into());
        let d4a = make_contact("d4a", "Alpha One", "a1@example.invalid", "+34629999999");
        let d4b = make_contact("d4b", "Beta Two", "b2@example.invalid", "+34629999999");
        let d5a = make_contact(
            "d5a",
            "Gamma Three",
            "shared@example.invalid",
            "+34621111111",
        );
        let d5b = make_contact(
            "d5b",
            "Delta Four",
            "shared@example.invalid",
            "+34622222222",
        );
        contacts.extend([d3a, d3b, d4a, d4b, d5a, d5b]);

        let mut naive = contacts.clone();
        detect_candidates_naive(&mut naive);

        let mut indexed = contacts;
        let mut cmp = 0u64;
        detect_candidates_indexed(&mut indexed, &DedupCtrl::default(), &mut cmp).unwrap();

        assert_eq!(
            notes_fingerprint(&naive),
            notes_fingerprint(&indexed),
            "pares D3–D6 deben coincidir con el algoritmo histórico"
        );
        // Con cubos acotados, las comparaciones no crecen como O(n²)≈2e6.
        let n = indexed.len() as u64;
        assert!(
            cmp < n * n / 4,
            "comparaciones {cmp} demasiado altas para n={n}"
        );
    }

    #[test]
    fn bucket_cap_bounds_comparisons() {
        // Un TEL compartido por 500 contactos → sin tope sería C(500,2); con CAP=64, ≤ C(64,2).
        let shared = "+34628888888";
        let contacts: Vec<_> = (0..500)
            .map(|i| {
                make_contact(
                    &format!("big{i}"),
                    &format!("Name {i}"),
                    &format!("n{i}@example.invalid"),
                    shared,
                )
            })
            .collect();
        let mut indexed = contacts;
        let mut cmp = 0u64;
        detect_candidates_indexed(&mut indexed, &DedupCtrl::default(), &mut cmp).unwrap();
        let max_per_bucket = (DEDUP_BUCKET_CAP * (DEDUP_BUCKET_CAP - 1) / 2) as u64;
        // Tres índices (fn, tel, email): cota holgada 3 * C(CAP,2) + margen.
        assert!(
            cmp <= max_per_bucket * 4,
            "cmp={cmp} supera cota por cubo {max_per_bucket}"
        );
    }

    #[test]
    fn dedup_respects_cancel_flag() {
        let flag = Arc::new(AtomicBool::new(true));
        let ctrl = DedupCtrl {
            cancelled: Some(flag),
            deadline: None,
        };
        let contacts: Vec<_> = (0..100)
            .map(|i| {
                make_contact(
                    &format!("c{i}"),
                    "Same",
                    &format!("c{i}@example.invalid"),
                    "+34620000000",
                )
            })
            .collect();
        let err = deduplicate_with_ctrl(contacts, &ctrl).unwrap_err();
        assert_eq!(err, DedupCancelled);
    }
}
