use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use std::time::Instant;

#[test]
#[ignore = "Massive benchmark - run manually with --ignored"]
fn test_massive_population_performance() {
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
        for e in &mut world.entities {
            e.metabolism.energy = 500.0;
        }
        world.update(&mut env).unwrap();
        if i % 10 == 0 {
            println!(
                "Tick {}: {:?} (Pop: {})",
                i,
                tick_start.elapsed(),
                world.entities.len()
            );
        }
    }
    let duration = start.elapsed();
    println!("Total time for 100 ticks: {:?}", duration);
    println!("Average time per tick: {:?}", duration / 100);

    assert!(
        !world.entities.is_empty(),
        "Population should survive some ticks"
    );
}
