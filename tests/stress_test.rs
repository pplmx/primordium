use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use std::time::Instant;

#[tokio::test] async
fn test_engine_stress_high_load() {
    let mut config = AppConfig::default();
    // High density configuration
    config.world.width = 150;
    config.world.height = 100;
    config.world.initial_population = 1500;
    config.world.max_food = 1000;

    let mut env = Environment::default();
    let mut world =
        World::new(config.world.initial_population, config).expect("Failed to create world");

    println!("Initial population: {}", world.get_population_count());
    assert!(world.get_population_count() >= 1000);

    let start = Instant::now();
    let ticks = 100;

    for i in 0..ticks {
        world
            .update(&mut env)
            .unwrap_or_else(|_| panic!("Engine crashed at tick {}", i));

        // Sanity check: Ensure at least some entities survive or life cycle continues
        if world.get_population_count() == 0 {
            println!("Extinction occurred at tick {}", i);
            break;
        }
    }

    let duration = start.elapsed();
    println!("Processed {} ticks with high load in {:?}", ticks, duration);
    println!("Final population: {}", world.get_population_count());

    // Success means it didn't crash and maintained performance
    assert!(
        duration.as_secs() < 10,
        "Performance too slow for 100 ticks under load"
    );
}
