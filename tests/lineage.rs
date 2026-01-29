use primordium_lib::model::config::AppConfig;
use primordium_lib::model::history;
use primordium_lib::model::lifecycle;
use primordium_lib::model::world::World;

#[test]
fn test_lineage_inheritance() {
    let config = AppConfig::default();
    let mut parent = lifecycle::create_entity(10.0, 10.0, 0);
    let original_lineage = parent.metabolism.lineage_id;

    // Asexual reproduction
    let child = primordium_lib::model::systems::social::reproduce_asexual(
        &mut parent,
        100,
        &config,
        1,
        std::collections::HashSet::new(),
        false,
    );

    assert_eq!(
        child.metabolism.lineage_id, original_lineage,
        "Child must inherit lineage from parent"
    );
    assert_eq!(
        child.intel.genotype.lineage_id, original_lineage,
        "Genotype must preserve lineage"
    );
}

#[test]
fn test_lineage_population_stats() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // Create 3 entities from 2 different lineages
    let e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let l1 = e1.metabolism.lineage_id;

    let mut e2 = primordium_lib::model::lifecycle::create_entity(20.0, 20.0, 0);
    e2.metabolism.lineage_id = l1; // Same lineage as e1

    let e3 = primordium_lib::model::lifecycle::create_entity(30.0, 30.0, 0);
    let l2 = e3.metabolism.lineage_id;
    assert_ne!(l1, l2);

    world.entities.extend(vec![e1, e2, e3]);

    // Update stats
    history::update_population_stats(
        &mut world.pop_stats,
        &world.entities,
        world.food.len(),
        0.0,
        0.0,
        0.1,
        &world.terrain,
    );

    assert_eq!(world.pop_stats.lineage_counts.get(&l1), Some(&2));
    assert_eq!(world.pop_stats.lineage_counts.get(&l2), Some(&1));
}

#[test]
fn test_multiverse_lineage_preservation() {
    let entity = primordium_lib::model::lifecycle::create_entity(5.0, 5.0, 0);
    let original_lineage = entity.metabolism.lineage_id;

    // Export to HexDNA (now unified Genotype)
    let dna = entity.intel.genotype.to_hex();

    // Import into a new world
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    let fingerprint = world.config.fingerprint();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(100.0f32.to_be_bytes());
    hasher.update(1u32.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let _ = world.import_migrant(dna, 100.0, 1, &fingerprint, &checksum);

    assert_eq!(
        world.entities[0].metabolism.lineage_id, original_lineage,
        "Lineage must survive multiverse migration"
    );
}
