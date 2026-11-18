use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
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
        tels: vec![Tel {
            value: tel_val.into(),
            tel_type: TelType::Cell,
            normalized: true,
        }],
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

fn corpus(count: usize) -> Vec<Contact> {
    (0..count)
        .map(|i| {
            make_contact(
                &format!("u{i}"),
                &format!("Person {i}"),
                &format!("p{i}@example.invalid"),
                &format!("+3461{i:07}"),
            )
        })
        .collect()
}

fn bench_dedup(c: &mut Criterion) {
    c.bench_function("dedup_512", |b| {
        b.iter_batched(
            || corpus(512),
            |contacts| {
                deduplicate_with_ctrl(contacts, &DedupCtrl::default()).expect("dedup sintético")
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_dedup);
criterion_main!(benches);
