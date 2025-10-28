use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

/// Benchmark: Simple operations
fn bench_simple_operations(c: &mut Criterion) {
    c.bench_function("vec_creation", |b| {
        b.iter(|| {
            let v = vec![1, 2, 3, 4, 5];
            black_box(v);
        });
    });
}

/// Benchmark: String concatenation
fn bench_string_concat(c: &mut Criterion) {
    c.bench_function("string_concat", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..100 {
                s.push_str(&i.to_string());
            }
            black_box(s);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = bench_simple_operations, bench_string_concat
}

criterion_main!(benches);
