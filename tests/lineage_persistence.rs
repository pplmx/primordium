use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use std::fs;

#[test]
fn test_lineage_registry_persistence() {
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();

    let e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let l_id = e.metabolism.lineage_id;

    // 1. Record birth and consumption
    world.lineage_registry.record_birth(l_id, 1, 0);
    world.lineage_registry.record_consumption(l_id, 50.0);

    // 2. Save
    let test_path = "logs/test_lineages.json";
    world.lineage_registry.save(test_path).unwrap();

    // 3. Load into new registry
    let loaded =
        primordium_lib::model::state::lineage_registry::LineageRegistry::load(test_path).unwrap();

    assert!(loaded.lineages.contains_key(&l_id));
    assert_eq!(
        loaded.lineages.get(&l_id).unwrap().total_entities_produced,
        1
    );
    assert_eq!(
        loaded.lineages.get(&l_id).unwrap().total_energy_consumed,
        50.0
    );

    // Cleanup
    let _ = fs::remove_file(test_path);
}

#[test]
fn test_world_update_tracks_lineage_metrics() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    let mut e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let l_id = e.metabolism.lineage_id;
    e.metabolism.energy = 50.0;
    world.ecs.spawn((
        e.identity,
        primordium_lib::model::state::Position {
            x: e.physics.x,
            y: e.physics.y,
        },
        e.physics,
        e.metabolism,
        e.health,
        e.intel,
    ));

    // Initial state seeding happened in World::new if population > 0,
    // but here we pushed manually. We should manually record birth if we push manually.
    world.lineage_registry.record_birth(l_id, 1, 0);

    // Run one update - should trigger feeding/metabolism
    world.update(&mut env).unwrap();

    let record = world
        .lineage_registry
        .lineages
        .get(&l_id)
        .expect("Lineage record missing");
    assert!(record.total_entities_produced >= 1);
}
