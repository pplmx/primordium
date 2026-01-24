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

    let mut world = World::new_at(1, config, log_dir).unwrap();

    let mut env = Environment::default();

    let l_id = world.entities[0].metabolism.lineage_id;

    // Debug: Check initial state
    let initial_pop = world
        .lineage_registry
        .lineages
        .get(&l_id)
        .map(|r| r.current_population)
        .unwrap_or(0);
    assert_eq!(initial_pop, 1, "Initial population should be 1");

    // 1. Force a "Legend" state
    world.entities[0].metabolism.offspring_count = 100;
    world.entities[0].metabolism.peak_energy = 500.0;

    // 2. Kill it through starvation
    world.entities[0].metabolism.energy = 0.0;

    // Run update
    world.update(&mut env).unwrap();

    // Debug: Check entity count
    assert_eq!(
        world.entities.len(),
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
