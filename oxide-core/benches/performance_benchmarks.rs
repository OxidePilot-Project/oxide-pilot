use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

/// Benchmark: String operations
fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_operations");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let s = "x".repeat(size);
                black_box(s);
            });
        });
    }

    group.finish();
}

/// Benchmark: Vector operations
fn bench_vec_operations(c: &mut Criterion) {
    c.bench_function("vec_creation", |b| {
        b.iter(|| {
            let v: Vec<i32> = (0..1000).collect();
            black_box(v);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(100);
    targets = bench_string_operations, bench_vec_operations
}

criterion_main!(benches);
