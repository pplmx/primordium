use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
#[ignore] // Long-running stability test
async fn test_stability_long_haul() {
    let mut config = AppConfig::default();
    config.world.width = 150;
    config.world.height = 150;
    config.world.seed = Some(12345);
    config.world.deterministic = true;
    config.world.initial_population = 500;

    let mut world = World::new(500, config).unwrap();
    let mut env = Environment::default();

    println!("Starting long-haul stability test (2000 ticks, 500 entities)...");

    for i in 0..2000 {
        world
            .update(&mut env)
            .unwrap_or_else(|_| panic!("Simulation failed at tick {}", i));

        let pop = world.get_population_count();
        if pop == 0 {
            println!("Population extinct at tick {}, restarting...", i);
            // In a stability test, we might want to ensure it can run even if they die out
            // but let's just assert it doesn't crash.
        }

        // Numerical stability check
        let stats = &world.pop_stats;
        assert!(
            stats.avg_lifespan.is_finite(),
            "NaN detected in avg_lifespan at tick {}",
            i
        );
        assert!(
            stats.biomass_h.is_finite(),
            "NaN detected in biomass_h at tick {}",
            i
        );
        assert!(
            stats.biomass_c.is_finite(),
            "NaN detected in biomass_c at tick {}",
            i
        );
        assert!(
            env.carbon_level.is_finite(),
            "NaN detected in carbon_level at tick {}",
            i
        );

        if i % 100 == 0 {
            println!("Tick {}: Pop={}", i, pop);
        }
    }

    println!("Long-haul stability test completed successfully.");
}
