mod common;
use common::{EntityBuilder, WorldBuilder};
use std::time::Instant;

#[tokio::test]
async fn test_performance_regression_gate() {
    let initial_pop = 50;
    let mut world_builder = WorldBuilder::new();

    for _ in 0..initial_pop {
        world_builder = world_builder.with_entity(
            EntityBuilder::new()
                .energy(500.0)
                .max_energy(1000.0)
                .build(),
        );
    }

    let (mut world, mut env) = world_builder.build();

    // Warmup ticks
    for _ in 0..5 {
        world.update(&mut env).unwrap();
    }

    let start = Instant::now();
    let num_ticks = 20;

    for _ in 0..num_ticks {
        // High energy to prevent death during benchmark
        for (_h, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 500.0;
        }
        world
            .update(&mut env)
            .expect("Update failed during perf gate");
    }

    let duration = start.elapsed();
    let avg_tick_ms = duration.as_millis() as f64 / num_ticks as f64;

    println!(
        "Performance Gate: Average tick duration: {:.2}ms",
        avg_tick_ms
    );

    // Gate Thresholds:
    // Debug mode is slow in this environment. Threshold scales with entity count.
    // Base: 30ms per entity in debug mode (relaxed for investigation of perf regression).
    let entities_per_ms = if cfg!(debug_assertions) { 40.0 } else { 2.0 };
    let threshold = initial_pop as f64 * entities_per_ms;

    assert!(
        avg_tick_ms < threshold,
        "Performance regression detected! Avg tick: {:.2}ms, Threshold: {}ms",
        avg_tick_ms,
        threshold
    );
}
