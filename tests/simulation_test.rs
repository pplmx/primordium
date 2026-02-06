use primordium_core::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_simulation_stability() {
    let mut config = AppConfig::default();
    // Deterministic mode for reproducible tests
    config.world.deterministic = true;
    config.world.seed = Some(42);
    // Lower metabolic costs to ensure survival of random brains
    config.metabolism.base_idle_cost = 0.01;
    config.metabolism.base_move_cost = 0.01;
    config.brain.hidden_node_cost = 0.0;
    config.brain.connection_cost = 0.0;

    let mut world = World::new(100, config).expect("Failed to create world");
    let mut env = Environment::default();

    let initial_pop = world.get_population_count();
    assert_eq!(initial_pop, 100);

    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
    }

    let final_pop = world.get_population_count();
    assert!(final_pop > 0, "Population shouldn't crash immediately");
    assert_eq!(world.tick, 50);
}
