use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use std::time::{Duration, Instant};

#[tokio::test]
#[ignore = "Massive benchmark - run manually with --ignored"]
async fn test_massive_population_performance() {
    let log_dir = "logs_test_perf";
    let _ = std::fs::remove_dir_all(log_dir);
    let mut config = AppConfig::default();
    config.world.width = 200;
    config.world.height = 200;
    config.world.initial_population = 10000;

    println!("Initializing 10,000 entities...");
    let mut world = World::new_at(10000, config, log_dir).unwrap();

    let mut env = Environment::default();

    println!("Starting benchmark for 100 ticks...");
    let start = Instant::now();
    for i in 0..100 {
        let tick_start = Instant::now();
        // Give them energy so they don't die
        for (_handle, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 500.0;
        }
        world.update(&mut env).unwrap();
        if i % 10 == 0 {
            println!(
                "Tick {}: {:?} (Pop: {})",
                i,
                tick_start.elapsed(),
                world.get_population_count()
            );
        }
    }
    let duration = start.elapsed();
    println!("Total time for 100 ticks: {:?}", duration);
    println!("Average time per tick: {:?}", duration / 100);

    assert!(
        world.get_population_count() > 0,
        "Population should survive some ticks"
    );
}

#[tokio::test]
#[ignore = "Micro-benchmark - run manually with --ignored"]
async fn benchmark_spatial_hash_query() {
    use primordium_lib::model::spatial_hash::SpatialHash;

    let width = 200;
    let height = 200;
    let cell_size = 5.0;

    let mut positions = Vec::new();
    for i in 0..10000 {
        let x = (i % width) as f64 * 5.0;
        let y = (i / width) as f64 * 5.0;
        positions.push((x, y));
    }

    let mut sh = SpatialHash::new(cell_size, width as u16, height as u16);

    let build_start = Instant::now();
    sh.build_parallel(&positions, width as u16, height as u16);
    let build_time = build_start.elapsed();
    println!(
        "SpatialHash build time for 10,000 entities: {:?}",
        build_time
    );

    let iterations = 100000;
    let count_start = Instant::now();
    let mut total = 0;
    for i in 0..iterations {
        total += sh.count_nearby(
            positions[i % positions.len()].0,
            positions[i % positions.len()].1,
            20.0,
        );
    }
    let count_time = count_start.elapsed();
    let avg_count = count_time
        .checked_div(iterations as u32)
        .unwrap_or(Duration::ZERO);
    println!(
        "count_nearby x{}: {:?} (avg: {:?}, total counted: {})",
        iterations, count_time, avg_count, total
    );

    let query_start = Instant::now();
    let mut found = 0;
    for i in 0..iterations {
        sh.query_callback(
            positions[i % positions.len()].0,
            positions[i % positions.len()].1,
            20.0,
            |_| found += 1,
        );
    }
    let query_time = query_start.elapsed();
    let avg_query = query_time
        .checked_div(iterations as u32)
        .unwrap_or(Duration::ZERO);
    println!(
        "query_callback x{}: {:?} (avg: {:?}, total found: {})",
        iterations, query_time, avg_query, found
    );
}

#[tokio::test]
#[ignore = "Micro-benchmark - run manually with --ignored"]
async fn benchmark_brain_forward_pass() {
    use primordium_lib::model::brain::BrainLogic;
    use primordium_lib::model::GenotypeLogic;

    let iterations = 10000;

    let genotype = primordium_data::Genotype::new_random();

    let mut activations = primordium_data::Activations::default();
    let inputs: [f32; 29] = [0.1; 29];
    let last_hidden: [f32; 6] = [0.05; 6];

    let forward_start = Instant::now();
    for _ in 0..iterations {
        let _ = genotype
            .brain
            .forward_internal(inputs, last_hidden, &mut activations);
    }
    let forward_time = forward_start.elapsed();
    let avg_forward = forward_time
        .checked_div(iterations as u32)
        .unwrap_or(Duration::ZERO);
    println!(
        "Brain forward_internal x{}: {:?} (avg: {:?})",
        iterations, forward_time, avg_forward
    );
}

#[tokio::test]
#[ignore = "Micro-benchmark - run manually with --ignored"]
async fn benchmark_full_tick_scaling() {
    for pop_size in [100, 500, 1000, 2000] {
        let log_dir = format!("logs_test_bench_{}", pop_size);
        let _ = std::fs::remove_dir_all(&log_dir);

        let mut config = AppConfig::default();
        config.world.width = 200;
        config.world.height = 200;
        config.world.initial_population = pop_size;

        let mut world = World::new_at(pop_size, config, &log_dir).unwrap();
        let mut env = Environment::default();

        let iterations = 10;
        let benchmark_start = Instant::now();
        for _ in 0..iterations {
            world.update(&mut env).unwrap();
        }
        let benchmark_time = benchmark_start.elapsed();
        let avg_tick = benchmark_time
            .checked_div(iterations as u32)
            .unwrap_or(Duration::ZERO);
        println!(
            "Full tick x{} with {} entities: {:?} (avg per tick: {:?})",
            iterations, pop_size, benchmark_time, avg_tick
        );
    }
}
