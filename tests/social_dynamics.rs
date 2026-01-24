use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_tribe_solidarity_no_aggression() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let mut e1 = Entity::new(10.0, 10.0, 0);
    let mut e2 = Entity::new(10.5, 10.5, 0);
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
    world.entities.push(e1);
    world.entities.push(e2);
    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
    }
    assert_eq!(world.entities.len(), 2, "Hunter attacked its own tribe!");
}

#[test]
fn test_energy_sharing_between_allies() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let mut e1 = Entity::new(10.0, 10.0, 0);
    let mut e2 = Entity::new(10.2, 10.2, 0);
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
    let e2_id = e2.id;
    world.entities.push(e1);
    world.entities.push(e2);
    let mut shared = false;
    for _ in 0..100 {
        world.update(&mut env).expect("Update failed");
        if let Some(e2_curr) = world.entities.iter().find(|e| e.id == e2_id) {
            if e2_curr.metabolism.energy > 15.0 {
                shared = true;
                break;
            }
        }
        // Keep E1 energy high and force share intent
        if let Some(e1_curr) = world
            .entities
            .iter_mut()
            .find(|e| e.physics.r == 200 && e.id != e2_id)
        {
            e1_curr.metabolism.energy = 800.0;
            e1_curr.intel.last_share_intent = 1.0;
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
    let mut e1 = Entity::new(10.0, 10.0, 0);
    let mut e2 = Entity::new(10.1, 10.1, 0);
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
            to: 25,
            weight: 10.0,
            enabled: true,
            innovation: 9999,
        });
    world.entities.push(e1);
    world.entities.push(e2);
    for _ in 0..200 {
        world.update(&mut env).expect("Update failed");
        if world.entities.len() == 1 {
            break;
        }
    }
}
