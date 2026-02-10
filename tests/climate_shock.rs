use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_climate_thermal_shock_stability() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    for _ in 0..10 {
        world.spawn_entity(lifecycle::create_entity(25.0, 25.0, 0));
    }

    // Oscillate between extreme CPU usages to trigger Era shifts and heat disasters
    for i in 0..100 {
        env.cpu_usage = if i % 20 < 10 { 95.0 } else { 5.0 };
        world.update(&mut env).unwrap();

        let stats = &world.pop_stats;
        assert!(
            stats.biomass_c.is_finite(),
            "Carbon biomass became non-finite during thermal shock!"
        );
    }
}

#[tokio::test]
async fn test_neural_topology_catatonic_metabolism() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut catatonic = lifecycle::create_entity(10.0, 10.0, 0);
    // Clear all connections - brain can't do anything
    {
        let brain = &mut std::sync::Arc::make_mut(&mut catatonic.intel.genotype).brain;
        brain.connections.clear();
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }
    catatonic.metabolism.energy = 100.0;
    let _h = world.spawn_entity(catatonic);

    for _ in 0..100 {
        world.update(&mut env).unwrap();
        if world.get_population_count() == 0 {
            break;
        }
    }

    // It should eventually die of metabolic cost despite doing nothing
    assert_eq!(
        world.get_population_count(),
        0,
        "Catatonic entity lived forever!"
    );
}

#[tokio::test]
async fn test_neural_topology_recursive_loop_bloat() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut recursive = lifecycle::create_entity(10.0, 10.0, 0);
    recursive.metabolism.energy = 50.0;

    // Create 100 internal recursive connections (hidden-to-hidden) to maximize "Brain Bloat" cost
    {
        let brain = &mut std::sync::Arc::make_mut(&mut recursive.intel.genotype).brain;
        for i in 0..100 {
            brain
                .connections
                .push(primordium_lib::model::brain::Connection {
                    from: 41 + (i % 6),
                    to: 41 + ((i + 1) % 6),
                    weight: 1.0,
                    enabled: true,
                    innovation: 10000 + i,
                });
        }
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    world.spawn_entity(recursive);

    // Should die very quickly due to connection maintenance costs
    for _ in 0..200 {
        world.update(&mut env).unwrap();
        if world.get_population_count() == 0 {
            break;
        }
    }

    assert_eq!(
        world.get_population_count(),
        0,
        "Recursive brain entity did not die from connection bloat cost!"
    );
}
