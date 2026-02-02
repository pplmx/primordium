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
    world.pop_stats.biomass_h = 10000.0;
    world.update(&mut env).unwrap();

    let f1 = world.terrain.get_cell(10, 10).fertility;
    assert!(f1 < f0, "Massive overgrazing should deplete fertility");
}

#[tokio::test]
async fn test_hunter_competition_impact() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let log_dir = "logs_test_stability";
    let _ = std::fs::remove_dir_all(log_dir);
    let mut world = World::new_at(0, config.clone(), log_dir).expect("Failed to create world");
    let mut env = Environment::default();

    let mut hunter = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    hunter.metabolism.trophic_potential = 1.0;
    hunter.metabolism.energy = 500.0;
    hunter.metabolism.max_energy = 1000.0;
    // Force aggression
    hunter
        .intel
        .genotype
        .brain
        .connections
        .push(primordium_lib::model::brain::Connection {
            from: 2,
            to: 32,
            weight: 10.0,
            enabled: true,
            innovation: 999,
        });

    let mut prey = primordium_lib::model::lifecycle::create_entity(10.5, 10.5, 0);
    prey.metabolism.energy = 500.0;
    prey.metabolism.max_energy = 1000.0;
    prey.metabolism.trophic_potential = 0.0;
    prey.intel.genotype.brain.connections.clear();
    prey.physics.r = 0;

    world.spawn_entity(hunter.clone());
    world.spawn_entity(prey.clone());
    assert_eq!(world.get_population_count(), 2);

    world.pop_stats.biomass_c = 0.0;
    world.update(&mut env).unwrap();
    let entities1 = world.get_all_entities();
    println!("Pop after update: {}", entities1.len());
    let hunter_after1 = entities1
        .iter()
        .find(|e| e.metabolism.trophic_potential > 0.9)
        .expect("Hunter missing");
    let energy1 = hunter_after1.metabolism.energy;

    let mut world2 = World::new_at(0, config, log_dir).expect("Failed to create world");
    world2.spawn_entity(hunter);
    world2.spawn_entity(prey);

    world2.pop_stats.biomass_c = 20000.0;
    world2.update(&mut env).unwrap();
    let entities2 = world2.get_all_entities();
    println!("Pop after update (world2): {}", entities2.len());
    let hunter_after2 = entities2
        .iter()
        .find(|e| e.metabolism.trophic_potential > 0.9)
        .expect("Hunter missing");
    let energy2 = hunter_after2.metabolism.energy;

    assert!(
        energy2 < energy1,
        "High competition should reduce energy gain from kill. Energy1: {}, Energy2: {}",
        energy1,
        energy2
    );
}
