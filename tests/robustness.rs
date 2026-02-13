use primordium_lib::model::config::AppConfig;
use primordium_lib::model::history;
use primordium_lib::model::lifecycle;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_corrupted_dna_handling() {
    use primordium_lib::model::state::entity::Genotype;
    // 1. Completely invalid hex
    let result = Genotype::from_hex("not_hex_at_all");
    assert!(result.is_err());

    // 2. Valid hex but invalid JSON content
    let invalid_json_hex = hex::encode("{\"weights_ih\": [1.0], \"invalid\": true}");
    let result2 = Genotype::from_hex(&invalid_json_hex);
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_lineage_registry_cleanup_on_extinction() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // Create an entity with a specific lineage
    let e = lifecycle::create_entity(10.0, 10.0, 0);
    let l_id = e.metabolism.lineage_id;
    world.spawn_entity(e);

    // Update stats - lineage should be there
    let entities = world.get_all_entities();
    let food_count = world.get_food_count();
    history::update_population_stats(history::StatsContext {
        stats: std::sync::Arc::make_mut(&mut world.pop_stats),
        entities: &entities,
        food_count,
        top_fitness: 0.0,
        carbon_level: 0.0,
        mutation_scale: 0.1,
        terrain: &world.terrain,
        tick: world.tick,
    });
    assert_eq!(world.pop_stats.lineage_counts.get(&l_id), Some(&1));

    // Kill the population
    world.ecs.clear();
    let entities = world.get_all_entities();
    let food_count = world.get_food_count();
    history::update_population_stats(history::StatsContext {
        stats: std::sync::Arc::make_mut(&mut world.pop_stats),
        entities: &entities,
        food_count,
        top_fitness: 0.0,
        carbon_level: 0.0,
        mutation_scale: 0.1,
        terrain: &world.terrain,
        tick: world.tick,
    });

    // Registry should show 0 or be empty for that ID
    assert_eq!(world.pop_stats.lineage_counts.get(&l_id), None);
    assert_eq!(world.pop_stats.population, 0);
}

#[tokio::test]
async fn test_multiverse_version_compatibility_resilience() {
    // Simulate a migration from a peer with missing fields (e.g. older version)
    // We should ensure import_migrant doesn't panic
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();

    let partial_dna_hex = hex::encode("{\"brain\": {\"weights_ih\": []}}"); // Incomplete genotype
    let fingerprint = world.config.fingerprint();

    // This should log an error or fail silently but NOT panic
    let _ = world.import_migrant(partial_dna_hex, 100.0, 1, &fingerprint, "dummy_checksum");

    // If it didn't panic, it's successful for this robustness test
    assert!(world.get_population_count() <= 1);
}
