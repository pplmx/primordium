use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_overgrazing_feedback_loop() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    let f0 = world.terrain.get_cell(10, 10).fertility;
    std::sync::Arc::make_mut(&mut world.pop_stats).biomass_h = 10000.0;
    world.update(&mut env).unwrap();

    let f1 = world.terrain.get_cell(10, 10).fertility;
    assert!(f1 < f0, "Massive overgrazing should deplete fertility");
}

#[tokio::test]
async fn test_hunter_competition_impact() {
    // Clean up log directories from previous test runs
    let _ = std::fs::remove_dir_all("logs_test_stability");
    let _ = std::fs::remove_dir_all("logs_test_stability1");
    let _ = std::fs::remove_dir_all("logs_test_stability2");

    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.deterministic = true;
    let log_dir1 = "logs_test_stability1";
    let _ = std::fs::remove_dir_all(log_dir1);
    let mut world = World::new_at(0, config.clone(), log_dir1).expect("Failed to create world");
    let mut env = Environment::default();

    let mut hunter = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    hunter.metabolism.trophic_potential = 1.0;
    hunter.metabolism.energy = 5000.0;
    hunter.metabolism.max_energy = 10000.0;
    hunter.physics.sensing_range = 20.0;
    std::sync::Arc::make_mut(&mut hunter.intel.genotype).sensing_range = 20.0;
    {
        let brain = &mut std::sync::Arc::make_mut(&mut hunter.intel.genotype).brain;
        brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 2,
                to: 32,
                weight: 10.0,
                enabled: true,
                innovation: 999,
            });
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    let mut prey = primordium_lib::model::lifecycle::create_entity(10.5, 10.5, 0);
    prey.metabolism.energy = 500.0;
    prey.metabolism.max_energy = 1000.0;
    prey.metabolism.trophic_potential = 0.0;
    prey.physics.max_speed = 0.0;
    std::sync::Arc::make_mut(&mut prey.intel.genotype).max_speed = 0.0;
    {
        let brain = &mut std::sync::Arc::make_mut(&mut prey.intel.genotype).brain;
        brain.connections.clear();
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }
    prey.physics.r = 0;

    // Low competition scenario: 1 hunter + 1 prey
    world.spawn_entity(hunter.clone());
    world.spawn_entity(prey.clone());
    assert_eq!(world.get_population_count(), 2);

    for _ in 0..100 {
        world.update(&mut env).unwrap();
        if world.get_population_count() == 1 {
            break;
        }
    }

    let entities1 = world.get_all_entities();
    let hunter_after1 = entities1
        .iter()
        .find(|e| e.metabolism.trophic_potential > 0.9)
        .expect("Hunter missing");
    let energy1 = hunter_after1.metabolism.energy;

    // High competition scenario: 40 hunters + 1 prey (spread out to avoid hunters killing each other)
    let log_dir2 = "logs_test_stability2";
    let _ = std::fs::remove_dir_all(log_dir2);
    let mut world2 = World::new_at(0, config, log_dir2).expect("Failed to create world");
    world2.spawn_entity(prey.clone());

    for i in 0..40 {
        let mut competitor = primordium_lib::model::lifecycle::create_entity(
            20.0 + (i % 20) as f64 * 2.0,
            20.0 + (i / 20) as f64 * 2.0,
            0,
        );
        competitor.metabolism.trophic_potential = 1.0;
        competitor.metabolism.energy = 5000.0;
        competitor.metabolism.max_energy = 10000.0;
        competitor.physics.sensing_range = 20.0;
        std::sync::Arc::make_mut(&mut competitor.intel.genotype).sensing_range = 20.0;
        {
            let brain = &mut std::sync::Arc::make_mut(&mut competitor.intel.genotype).brain;
            brain
                .connections
                .push(primordium_lib::model::brain::Connection {
                    from: 2,
                    to: 32,
                    weight: 10.0,
                    enabled: true,
                    innovation: 999,
                });
            use primordium_lib::model::brain::BrainLogic;
            brain.initialize_node_idx_map();
        }
        world2.spawn_entity(competitor);
    }
    assert_eq!(world2.get_population_count(), 41);

    for _ in 0..100 {
        world2.update(&mut env).unwrap();
        if world2.get_population_count() == 40 {
            break;
        }
    }

    let entities2 = world2.get_all_entities();
    let hunter_after2 = entities2
        .iter()
        .find(|e| e.metabolism.trophic_potential > 0.9)
        .expect("Hunter missing");
    let energy2 = hunter_after2.metabolism.energy;

    assert!(
        energy2 < energy1,
        "High competition should reduce energy gain from kill. Energy1 (1 hunter): {}, Energy2 (40 hunters): {}",
        energy1,
        energy2
    );
}
