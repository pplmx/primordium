use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_voluntary_bond_breaking() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let mut e2 = primordium_lib::model::lifecycle::create_entity(10.2, 10.2, 0);

    // Setup compatible genotypes for bonding
    e2.intel.genotype = e1.intel.genotype.clone();

    let e1_id = e1.identity.id;
    let e2_id = e2.identity.id;

    // Manually force bond
    e1.intel.bonded_to = Some(e2_id);
    e2.intel.bonded_to = Some(e1_id);

    // Configure brain to output low Bond signal (Index 30 < 0.2)
    // We add a connection from a constant input (Bias, Index 20/Hearing is usually 0 but we can use Energy)
    // Or we can just set the brain to produce negative output for node 30
    e1.intel.genotype.brain.connections.clear();
    e1.intel
        .genotype
        .brain
        .connections
        .push(primordium_lib::model::brain::Connection {
            from: 2, // Energy Input (high)
            to: 31,  // Hidden
            weight: -10.0,
            enabled: true,
            innovation: 1,
        });
    e1.intel
        .genotype
        .brain
        .connections
        .push(primordium_lib::model::brain::Connection {
            from: 31, // Hidden
            to: 30,   // Bond Output (Index 30)
            weight: 10.0,
            enabled: true,
            innovation: 2,
        });
    // With Energy ~1.0:
    // Hidden = tanh(1.0 * -10.0) = -1.0
    // Output = tanh(-1.0 * 10.0) = -1.0
    // -1.0 < 0.2 -> Should Break Bond

    world.spawn_entity(e1);
    world.spawn_entity(e2);

    // Run simulation
    world.update(&mut env).expect("Update failed");

    // Verify bond broken for E1
    let entities = world.get_all_entities();
    let e1_curr = entities.iter().find(|e| e.identity.id == e1_id).unwrap();
    assert!(
        e1_curr.intel.bonded_to.is_none(),
        "E1 should have broken the bond voluntarily"
    );

    // E2 should still be bonded (unless it also broke it, but we didn't wire its brain)
    // Actually E2's brain is empty/random, likely 0.0 output.
    // Tanh(0) = 0.0 < 0.2, so E2 might also break it!
    // That's fine, we just want to prove the mechanism works.
}
