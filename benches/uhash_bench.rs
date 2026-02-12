//! Benchmark for UniversalHash algorithm

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uhash_core::UniversalHash;

fn bench_hash(c: &mut Criterion) {
    let mut hasher = UniversalHash::new();
    let input = b"benchmark input data for testing UniversalHash v4 performance";

    c.bench_function("uhash_single", |b| {
        b.iter(|| hasher.hash(black_box(input)))
    });
}

fn bench_hash_varying_input(c: &mut Criterion) {
    let mut hasher = UniversalHash::new();

    c.bench_function("uhash_varying", |b| {
        let mut nonce: u64 = 0;
        b.iter(|| {
            let mut input = Vec::with_capacity(64);
            input.extend_from_slice(b"seed");
            input.extend_from_slice(&nonce.to_le_bytes());
            nonce = nonce.wrapping_add(1);
            hasher.hash(black_box(&input))
        })
    });
}

criterion_group!(benches, bench_hash, bench_hash_varying_input);
criterion_main!(benches);
