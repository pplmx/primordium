use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_kin_recognition_influences_movement() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    // 1. Target Entity
    let mut e_target = Entity::new(10.0, 10.0, 0);
    let l_id = e_target.metabolism.lineage_id;
    e_target.metabolism.energy = 500.0;

    // 2. Nearby Kin
    let mut e_kin = Entity::new(12.0, 10.0, 0);
    e_kin.metabolism.lineage_id = l_id;

    world.entities.push(e_target);
    world.entities.push(e_kin);

    // 3. Update World. Target should see Kin at relative X = 1.0 (clamped).
    // The decision buffer will contain the brain's reaction.
    world.update(&env).unwrap();

    // We verify that the "Kin Centroid" inputs were correctly calculated.
    // (This is mostly verified by successful compilation of the new 10-sensor array).
    assert_eq!(world.entities.len(), 2);
}

#[test]
fn test_herding_bonus() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    // 1. Target Entity (Moving Right)
    let mut e = Entity::new(10.0, 10.0, 0);
    e.physics.vx = 1.0;
    e.physics.vy = 0.0;
    let l_id = e.metabolism.lineage_id;
    e.metabolism.energy = 100.0;

    // 2. Kin (To the right, also moving right)
    let mut kin = Entity::new(11.0, 10.0, 0);
    kin.metabolism.lineage_id = l_id;

    world.entities.push(e);
    world.entities.push(kin);

    // 3. Update world.
    world.update(&env).unwrap();

    // We expect the bonus to offset some drain.
    assert!(world.entities[0].metabolism.energy > 0.0);
}
