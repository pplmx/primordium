use primordium_lib::model::brain::Connection;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;

#[test]
fn test_larval_gating_logic() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // 1. Create a larva
    let mut larva = lifecycle::create_entity(5.0, 5.0, 0);
    larva.metabolism.has_metamorphosed = false;

    // Force brain outputs: Dig (index 9), Build (index 10), Bond (index 8) to 1.0
    // We need to manipulate the connections to hidden nodes to ensure these outputs are high.
    // Or simpler: manually set the decision_buffer in a mock loop, but we can't easily do that.
    // Instead, we'll manually set the outputs in a way that would trigger commands if not gated.

    // We'll test by setting the terrain to Wall and see if the larva can Dig it.
    world.terrain.set_cell_type(5, 5, TerrainType::Wall);

    // We'll give it enough energy
    larva.metabolism.energy = 100.0;

    // To ensure outputs are high, we'll manually modify the connections in the brain.
    // Connect all inputs to Dig/Build outputs with high weights.
    for i in 0..26 {
        larva.intel.genotype.brain.connections.push(Connection {
            from: i,
            to: 35,
            weight: 1.0,
            enabled: true,
            innovation: 10000 + i,
        });
        larva.intel.genotype.brain.connections.push(Connection {
            from: i,
            to: 36,
            weight: 1.0,
            enabled: true,
            innovation: 11000 + i,
        });
    }

    world.entities.push(larva);

    // Run one tick
    world.update(&mut env).unwrap();

    // Verify: Terrain at (5,5) should still be Wall because larva cannot Dig
    assert_eq!(world.terrain.get(5.0, 5.0).terrain_type, TerrainType::Wall);
}

#[test]
fn test_metamorphosis_transition_and_remodeling() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.metabolism.maturity_age = 10;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // 1. Create a larva close to maturity
    let mut larva = lifecycle::create_entity(5.0, 5.0, 0);
    larva.metabolism.has_metamorphosed = false;
    larva.metabolism.birth_tick = 0;
    larva.intel.genotype.maturity_gene = 1.0;

    larva.intel.genotype.brain.connections.retain(|c| c.to < 34);

    let initial_max_energy = larva.metabolism.max_energy;
    let initial_speed = larva.physics.max_speed;

    world.entities.push(larva);

    world.tick = 10;

    let events = world.update(&mut env).unwrap();

    let metamorphosed = events.iter().any(|e| {
        matches!(
            e,
            primordium_lib::model::history::LiveEvent::Metamorphosis { .. }
        )
    });
    assert!(metamorphosed, "Metamorphosis event should be triggered");

    let adult = &world.entities[0];
    assert!(adult.metabolism.has_metamorphosed);
    assert!(adult.metabolism.max_energy > initial_max_energy);
    assert!(adult.physics.max_speed > initial_speed);

    let has_dig_conn = adult
        .intel
        .genotype
        .brain
        .connections
        .iter()
        .any(|c| c.to == 35 && c.enabled);
    assert!(
        has_dig_conn,
        "Adult brain should have Dig connections after remodeling"
    );
}
