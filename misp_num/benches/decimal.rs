use criterion::{Criterion, criterion_group, criterion_main};
use misp_num::decimal::Decimal;
use std::hint::black_box;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("basic add", |b| {
        b.iter(|| Decimal::from(black_box(12345)) + Decimal::from(black_box(12345)));
    });

    c.bench_function("basic sub", |b| {
        b.iter(|| Decimal::from(black_box(12345)) - Decimal::from(black_box(12345)));
    });

    c.bench_function("basic mult", |b| {
        b.iter(|| Decimal::from(black_box(12345)) * Decimal::from(black_box(12345)));
    });

    c.bench_function("basic div", |b| {
        b.iter(|| Decimal::from(black_box(12345)) / Decimal::from(black_box(12345)));
    });

    c.bench_function("perfect sqrt", |b| {
        b.iter(|| Decimal::from(black_box(16777216)).sqrt());
    });

    c.bench_function("pi sqrt", |b| {
        b.iter(|| Decimal::PI.sqrt());
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
