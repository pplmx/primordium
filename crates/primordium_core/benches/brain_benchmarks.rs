use criterion::{black_box, criterion_group, criterion_main, Criterion};
use primordium_core::brain::{BrainLogic, GenotypeLogic};
use primordium_core::brain::{BRAIN_INPUTS, BRAIN_MEMORY};
use primordium_data::{Brain, Genotype};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Benchmark brain forward pass with typical inputs.
fn bench_brain_forward(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let brain = Brain::new_random_with_rng(&mut rng);
    let inputs = [0.5; BRAIN_INPUTS];
    let hidden = [0.0; BRAIN_MEMORY];

    c.bench_function("brain_forward", |b| {
        b.iter(|| {
            let result = brain.forward(black_box(inputs), hidden);
            black_box(result)
        })
    });
}

/// Benchmark brain forward pass with extreme values.
fn bench_brain_forward_extreme(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let brain = Brain::new_random_with_rng(&mut rng);
    let inputs = [1.0; BRAIN_INPUTS];
    let hidden = [1.0; BRAIN_MEMORY];

    c.bench_function("brain_forward_extreme", |b| {
        b.iter(|| {
            let result = brain.forward(black_box(inputs), hidden);
            black_box(result)
        })
    });
}

/// Benchmark brain creation.
fn bench_brain_creation(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    c.bench_function("brain_creation", |b| {
        b.iter(|| {
            let brain = Brain::new_random_with_rng(&mut rng);
            black_box(brain)
        })
    });
}

/// Benchmark genotype creation.
fn bench_genotype_creation(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    c.bench_function("genotype_creation", |b| {
        b.iter(|| {
            let genotype = Genotype::new_random_with_rng(&mut rng);
            black_box(genotype)
        })
    });
}

/// Benchmark brain crossover.
fn bench_brain_crossover(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let p1 = Brain::new_random_with_rng(&mut rng);
    let p2 = Brain::new_random_with_rng(&mut rng);

    c.bench_function("brain_crossover", |b| {
        b.iter(|| {
            let child = p1.crossover_with_rng(&p2, &mut rng);
            black_box(child)
        })
    });
}

/// Benchmark serialization to hex.
fn bench_brain_to_hex(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let brain = Brain::new_random_with_rng(&mut rng);

    c.bench_function("brain_to_hex", |b| {
        b.iter(|| {
            let hex = brain.to_hex();
            black_box(hex)
        })
    });
}

/// Benchmark deserialization from hex.
fn bench_brain_from_hex(c: &mut Criterion) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let brain = Brain::new_random_with_rng(&mut rng);
    let hex = brain.to_hex();

    c.bench_function("brain_from_hex", |b| {
        b.iter(|| {
            let restored = Brain::from_hex(&hex).unwrap();
            black_box(restored)
        })
    });
}

criterion_group!(
    benches,
    bench_brain_forward,
    bench_brain_forward_extreme,
    bench_brain_creation,
    bench_genotype_creation,
    bench_brain_crossover,
    bench_brain_to_hex,
    bench_brain_from_hex
);
criterion_main!(benches);
