use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_fossilization_and_snapshots() {
    let log_dir = "logs_test_archeology_isolated";
    let _ = std::fs::remove_dir_all(log_dir);
    let _ = std::fs::create_dir_all(log_dir);

    let mut config = AppConfig::default();
    config.world.width = 20;
    config.world.height = 20;
    config.world.initial_population = 1;
    config.world.initial_food = 0;
    config.world.max_food = 0;

    let mut world = World::new_at(0, config, log_dir).unwrap();

    let mut env = Environment::default();

    let mut e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let l_id = e.metabolism.lineage_id;

    // 1. Force a "Legend" state
    e.metabolism.offspring_count = 100;
    e.metabolism.peak_energy = 500.0;
    // 2. Kill it through starvation
    e.metabolism.energy = 0.0;

    world.spawn_entity(e);

    world.lineage_registry.record_birth(l_id, 1, 0);

    // Run update
    world.update(&mut env).unwrap();

    // Debug: Check entity count
    assert_eq!(
        world.get_population_count(),
        0,
        "Entity should be dead after starvation"
    );

    assert!(
        world
            .lineage_registry
            .lineages
            .get(&l_id)
            .unwrap()
            .is_extinct,
        "Lineage should be extinct after population death"
    );

    // 3. Fast forward to tick 1001 (where snapshot and fossilization happen every 1000)
    while world.tick < 1001 {
        world.update(&mut env).unwrap();
    }

    // 4. Check if fossil exists
    assert!(
        world
            .fossil_registry
            .fossils
            .iter()
            .any(|f| f.lineage_id == l_id),
        "Fossil should be created for extinct legendary lineage"
    );

    // 5. Check if Snapshot event was emitted
    let snapshots = world.logger.get_snapshots().unwrap();
    assert!(
        !snapshots.is_empty(),
        "Snapshots should be recorded in the log file"
    );
}
