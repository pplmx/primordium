use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_fossilization_and_snapshots() {
    let mut config = AppConfig::default();
    config.world.width = 20;
    config.world.height = 20;
    config.world.initial_population = 1;

    let mut world = World::new(1, config).unwrap();
    let mut env = Environment::default();

    let l_id = world.entities[0].metabolism.lineage_id;

    // 1. Force a "Legend" state
    world.entities[0].metabolism.offspring_count = 100;
    world.entities[0].metabolism.peak_energy = 500.0;

    // 2. Kill it through starvation (setting energy to 0)
    world.entities[0].metabolism.energy = 0.0;

    // Update world - this should trigger record_death and archive_if_legend
    world.update(&mut env).unwrap();

    // Ensure it's extinct in registry
    assert!(
        world
            .lineage_registry
            .lineages
            .get(&l_id)
            .unwrap()
            .is_extinct
    );

    // 3. Fast forward to tick 1000 (where snapshot and fossilization happen)
    while world.tick < 1000 {
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
