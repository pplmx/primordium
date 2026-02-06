use criterion::{black_box, criterion_group, criterion_main, Criterion};
use primordium_core::spatial_hash::SpatialHash;

fn bench_spatial_hash_build(c: &mut Criterion) {
    let positions: Vec<(f64, f64)> = (0..1000)
        .map(|i| {
            let x = (i % 100) as f64 * 10.0;
            let y = (i / 100) as f64 * 10.0;
            (x, y)
        })
        .collect();

    c.bench_function("spatial_hash_build_1000", |b| {
        b.iter(|| {
            let mut spatial = SpatialHash::new(10.0, 1000, 1000);
            spatial.build_parallel(&positions, 1000, 1000);
            black_box(spatial)
        })
    });
}

fn bench_spatial_hash_query(c: &mut Criterion) {
    let positions: Vec<(f64, f64)> = (0..1000)
        .map(|i| {
            let x = (i % 100) as f64 * 10.0;
            let y = (i / 100) as f64 * 10.0;
            (x, y)
        })
        .collect();

    let mut spatial = SpatialHash::new(10.0, 1000, 1000);
    spatial.build_parallel(&positions, 1000, 1000);

    c.bench_function("spatial_hash_query_50_radius", |b| {
        let mut results = Vec::new();
        b.iter(|| {
            results.clear();
            spatial.query_into(500.0, 500.0, 50.0, &mut results);
            black_box(results.len())
        })
    });
}

fn bench_spatial_hash_query_small(c: &mut Criterion) {
    let positions: Vec<(f64, f64)> = (0..1000)
        .map(|i| {
            let x = (i % 100) as f64 * 10.0;
            let y = (i / 100) as f64 * 10.0;
            (x, y)
        })
        .collect();

    let mut spatial = SpatialHash::new(10.0, 1000, 1000);
    spatial.build_parallel(&positions, 1000, 1000);

    c.bench_function("spatial_hash_query_10_radius", |b| {
        let mut results = Vec::new();
        b.iter(|| {
            results.clear();
            spatial.query_into(500.0, 500.0, 10.0, &mut results);
            black_box(results.len())
        })
    });
}

fn bench_spatial_hash_count_nearby(c: &mut Criterion) {
    let positions: Vec<(f64, f64)> = (0..1000)
        .map(|i| {
            let x = (i % 100) as f64 * 10.0;
            let y = (i / 100) as f64 * 10.0;
            (x, y)
        })
        .collect();

    let mut spatial = SpatialHash::new(10.0, 1000, 1000);
    spatial.build_parallel(&positions, 1000, 1000);

    c.bench_function("spatial_hash_count_nearby_50", |b| {
        b.iter(|| {
            let count = spatial.count_nearby(500.0, 500.0, 50.0);
            black_box(count)
        })
    });
}

criterion_group!(
    benches,
    bench_spatial_hash_build,
    bench_spatial_hash_query,
    bench_spatial_hash_query_small,
    bench_spatial_hash_count_nearby
);
criterion_main!(benches);
