//! Benchmarks S-03 fuera de ci-gate (`--ignored`): 40k < 5s; cancel 100k < 2s (margen x2).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use zedazo_core::domain::contact::{CategorySet, Contact, SourceDetail, Tel, TelType, TypedValue};
use zedazo_core::domain::identity::{deduplicate_with_ctrl, DedupCtrl};
use zedazo_core::domain::screening::ScreeningDecision;

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
        source_detail: SourceDetail::Unknown("perf".into()),
        decision: ScreeningDecision::Conserved,
        screening_rule: String::new(),
        merged_uids: vec![],
        adr_field: None,
    }
}

fn corpus(n: usize) -> Vec<Contact> {
    (0..n)
        .map(|i| {
            make_contact(
                &format!("u{i}"),
                &format!("Person {i}"),
                &format!("p{i}@example.invalid"),
                &format!("+3461{:07}", i % 50_000),
            )
        })
        .collect()
}

#[test]
#[ignore = "S-03 release-build: 40k dedup < 5s (margen x2 → 10s)"]
fn dedup_40k_under_budget() {
    let contacts = corpus(40_000);
    let start = Instant::now();
    let (_out, _) = deduplicate_with_ctrl(contacts, &DedupCtrl::default()).unwrap();
    let elapsed = start.elapsed();
    eprintln!("dedup_40k_elapsed_ms={}", elapsed.as_millis());
    assert!(
        elapsed < Duration::from_secs(10),
        "40k dedup tardó {elapsed:?} (presupuesto 5s, margen x2 = 10s)"
    );
}

#[test]
#[ignore = "S-03 release-build: cancelación 100k < 2s (margen x2 → 4s)"]
fn dedup_cancel_100k_under_budget() {
    let contacts = corpus(100_000);
    let flag = Arc::new(AtomicBool::new(false));
    let ctrl = DedupCtrl {
        cancelled: Some(Arc::clone(&flag)),
        deadline: None,
    };
    let flag2 = Arc::clone(&flag);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        flag2.store(true, Ordering::SeqCst);
    });
    let start = Instant::now();
    let res = deduplicate_with_ctrl(contacts, &ctrl);
    let elapsed = start.elapsed();
    eprintln!("dedup_cancel_100k_elapsed_ms={}", elapsed.as_millis());
    assert!(res.is_err(), "debe cancelar");
    assert!(
        elapsed < Duration::from_secs(4),
        "cancel 100k tardó {elapsed:?} (presupuesto 2s, margen x2 = 4s)"
    );
}
