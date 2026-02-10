use primordium_core::systems::social;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::history;
use primordium_lib::model::lifecycle;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_lineage_inheritance() {
    let config = AppConfig::default();
    let parent = lifecycle::create_entity(10.0, 10.0, 0);
    let original_lineage = parent.metabolism.lineage_id;

    // Asexual reproduction
    let mut rng = rand::thread_rng();
    let mut ctx = social::ReproductionContext {
        tick: 100,
        config: &config,
        population: 1,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child, _) = social::reproduce_asexual_parallel_components_decomposed(
        social::AsexualReproductionContext {
            pos: &parent.position,
            energy: parent.metabolism.energy,
            generation: parent.metabolism.generation,
            genotype: &parent.intel.genotype,
            specialization: parent.intel.specialization,
            ctx: &mut ctx,
        },
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

#[tokio::test]
async fn test_lineage_population_stats() {
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

    world.spawn_entity(e1);
    world.spawn_entity(e2);
    world.spawn_entity(e3);

    // Update stats
    let entities = world.get_all_entities();
    let food_count = world.get_food_count();
    history::update_population_stats(history::StatsContext {
        stats: &mut world.pop_stats,
        entities: &entities,
        food_count,
        top_fitness: 0.0,
        carbon_level: 0.0,
        mutation_scale: 0.1,
        terrain: &world.terrain,
    });

    assert_eq!(world.pop_stats.lineage_counts.get(&l1), Some(&2));
    assert_eq!(world.pop_stats.lineage_counts.get(&l2), Some(&1));
}

#[tokio::test]
async fn test_multiverse_lineage_preservation() {
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

    let entities = world.get_all_entities();
    assert_eq!(
        entities[0].metabolism.lineage_id, original_lineage,
        "Lineage must survive multiverse migration"
    );
}
