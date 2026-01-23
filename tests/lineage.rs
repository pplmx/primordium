use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::world::World;

#[test]
fn test_lineage_inheritance() {
    let config = AppConfig::default();
    let mut parent = Entity::new(10.0, 10.0, 0);
    let original_lineage = parent.metabolism.lineage_id;

    // Asexual reproduction
    let child = primordium_lib::model::systems::social::reproduce_asexual(
        &mut parent,
        100,
        &config.evolution,
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
    let e1 = Entity::new(10.0, 10.0, 0);
    let l1 = e1.metabolism.lineage_id;

    let mut e2 = Entity::new(20.0, 20.0, 0);
    e2.metabolism.lineage_id = l1; // Same lineage as e1

    let e3 = Entity::new(30.0, 30.0, 0);
    let l2 = e3.metabolism.lineage_id;
    assert_ne!(l1, l2);

    world.entities.extend(vec![e1, e2, e3]);

    // Update stats
    world
        .pop_stats
        .update_snapshot(&world.entities, world.food.len(), 0.0);

    assert_eq!(world.pop_stats.lineage_counts.get(&l1), Some(&2));
    assert_eq!(world.pop_stats.lineage_counts.get(&l2), Some(&1));
}

#[test]
fn test_multiverse_lineage_preservation() {
    let entity = Entity::new(5.0, 5.0, 0);
    let original_lineage = entity.metabolism.lineage_id;

    // Export to HexDNA (now unified Genotype)
    let dna = entity.intel.genotype.to_hex();

    // Import into a new world
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    world.import_migrant(dna, 100.0, 1);

    assert_eq!(
        world.entities[0].metabolism.lineage_id, original_lineage,
        "Lineage must survive multiverse migration"
    );
}
