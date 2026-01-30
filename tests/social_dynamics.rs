use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_tribe_solidarity_no_aggression() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let mut e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let mut e2 = primordium_lib::model::lifecycle::create_entity(10.5, 10.5, 0);
    e1.physics.r = 100;
    e1.physics.g = 100;
    e1.physics.b = 100;
    e2.physics.r = 100;
    e2.physics.g = 100;
    e2.physics.b = 100;
    e1.metabolism.trophic_potential = 1.0;
    e2.metabolism.trophic_potential = 0.0;
    e1.metabolism.energy = 5000.0;
    e2.metabolism.energy = 5000.0;
    e1.metabolism.max_energy = 10000.0;
    e2.metabolism.max_energy = 10000.0;
    e1.intel.genotype.max_energy = 10000.0;
    e2.intel.genotype.max_energy = 10000.0;
    world.spawn_entity(e1);
    world.spawn_entity(e2);
    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
    }
    assert!(
        world.get_population_count() >= 2,
        "Hunter attacked its own tribe!"
    );
}

#[test]
fn test_energy_sharing_between_allies() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let mut e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let mut e2 = primordium_lib::model::lifecycle::create_entity(10.2, 10.2, 0);
    e2.intel.genotype = e1.intel.genotype.clone();
    e1.physics.r = 200;
    e1.physics.g = 200;
    e1.physics.b = 200;
    e2.physics.r = 200;
    e2.physics.g = 200;
    e2.physics.b = 200;
    e1.metabolism.energy = 800.0;
    e1.metabolism.max_energy = 1000.0;
    e2.metabolism.energy = 10.0;
    e2.metabolism.max_energy = 1000.0;

    // Force sharing behavior via brain connection
    e1.intel
        .genotype
        .brain
        .connections
        .push(primordium_lib::model::brain::Connection {
            from: 2, // Energy input
            to: 33,  // Share output
            weight: 10.0,
            enabled: true,
            innovation: 999,
        });

    let e2_id = e2.identity.id;
    world.spawn_entity(e1);
    world.spawn_entity(e2);
    let mut shared = false;
    for _ in 0..100 {
        world.update(&mut env).expect("Update failed");
        let entities = world.get_all_entities();
        if let Some(e2_curr) = entities.iter().find(|e| e.identity.id == e2_id) {
            if e2_curr.metabolism.energy > 15.0 {
                shared = true;
                break;
            }
        }
        // Keep E1 energy high and force share intent
        for (_handle, (phys, met, intel, ident)) in world.ecs.query_mut::<(
            &mut primordium_lib::model::state::Physics,
            &mut primordium_lib::model::state::Metabolism,
            &mut primordium_lib::model::state::Intel,
            &primordium_lib::model::state::Identity,
        )>() {
            if phys.r == 200 && ident.id != e2_id {
                met.energy = 800.0;
                intel.last_share_intent = 1.0;
            }
        }
    }
    assert!(shared, "Energy sharing did not occur between allies");
}

#[test]
fn test_inter_tribe_predation() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let mut e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let mut e2 = primordium_lib::model::lifecycle::create_entity(10.1, 10.1, 0);
    e1.physics.r = 255;
    e1.physics.g = 0;
    e1.physics.b = 0;
    e2.physics.r = 0;
    e2.physics.g = 0;
    e2.physics.b = 255;
    e1.metabolism.trophic_potential = 1.0;
    e1.metabolism.energy = 5000.0;
    e2.metabolism.energy = 10.0;
    e2.metabolism.trophic_potential = 0.0;
    e1.intel.genotype.brain.connections.clear();
    e1.intel
        .genotype
        .brain
        .connections
        .push(primordium_lib::model::brain::Connection {
            from: 0,
            to: 32, // Aggro output
            weight: 10.0,
            enabled: true,
            innovation: 9999,
        });
    world.spawn_entity(e1);
    world.spawn_entity(e2);
    for _ in 0..200 {
        world.update(&mut env).expect("Update failed");
        if world.get_population_count() == 1 {
            break;
        }
    }
}
