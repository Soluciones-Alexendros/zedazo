use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use zedazo_core::infrastructure::parser::parse_vcards;

fn synthetic_vcf(count: usize) -> String {
    (0..count)
        .map(|i| {
            format!(
                "BEGIN:VCARD\nVERSION:4.0\nFN:Person {i}\nEMAIL:p{i}@example.invalid\nTEL;TYPE=CELL:+3460000{i:04}\nEND:VCARD\n"
            )
        })
        .collect()
}

fn bench_parse(c: &mut Criterion) {
    let vcf = synthetic_vcf(100);
    c.bench_function("parse_vcards_100", |b| {
        b.iter(|| parse_vcards(black_box(&vcf)).expect("vcf sintético válido"));
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
