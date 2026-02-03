use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use std::collections::HashSet;

#[tokio::test]
async fn test_high_density_interaction_safety() {
    let mut config = AppConfig::default();
    config.world.width = 50;
    config.world.height = 50;
    config.world.initial_population = 0;

    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    // Spawn 100 entities in a tiny 2x2 area to force massive overlap
    for _ in 0..100 {
        let mut e = lifecycle::create_entity(25.0, 25.0, 0);
        e.metabolism.energy = 100.0;
        e.metabolism.max_energy = 500.0;
        // High aggression to ensure they attack each other
        e.intel
            .genotype
            .brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 3, // Density
                to: 32,  // Aggro
                weight: 10.0,
                enabled: true,
                innovation: 12345,
            });
        world.spawn_entity(e);
    }

    // Run for multiple ticks and check for data corruption or panics
    for _ in 0..50 {
        world
            .update(&mut env)
            .expect("Update failed under high density");

        let entities = world.get_all_entities();
        let mut seen_ids = HashSet::new();

        for e in entities {
            // Check 1: No duplicate IDs (ECS corruption)
            assert!(
                seen_ids.insert(e.identity.id),
                "Duplicate entity ID detected!"
            );

            // Check 2: No negative energy (Interaction logic failure)
            assert!(
                e.metabolism.energy >= 0.0,
                "Negative energy detected: {}",
                e.metabolism.energy
            );

            // Check 3: Coordinates are finite (NaN check)
            assert!(e.position.x.is_finite(), "NaN X coordinate detected");
            assert!(e.position.y.is_finite(), "NaN Y coordinate detected");
        }
    }
}

/*
#[tokio::test]
async fn test_race_condition_predation_determinism() {
    // ...
}
*/
