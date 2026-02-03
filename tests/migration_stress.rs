use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;
use primordium_lib::model::BrainLogic;

#[tokio::test]
async fn test_migration_burst_stability() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).expect("Failed to create world");

    let fingerprint = config.fingerprint();
    let l_id = uuid::Uuid::new_v4();

    // Create a valid genotype via API
    let brain = primordium_lib::model::brain::Brain::new_random();
    let genotype = primordium_lib::model::state::entity::Genotype {
        brain,
        sensing_range: 5.0,
        max_speed: 1.0,
        max_energy: 100.0,
        lineage_id: l_id,
        metabolic_niche: 0.5,
        trophic_potential: 0.0,
        reproductive_investment: 0.5,
        maturity_gene: 1.0,
        mate_preference: 0.5,
        pairing_bias: 0.5,
        specialization_bias: [0.33, 0.33, 0.34],
        regulatory_rules: Vec::new(),
    };
    let dna_template = genotype.to_hex();

    // Simulate 200 migrants arriving simultaneously
    for i in 0..200 {
        let energy = 100.0f32;
        let gen = i as u32;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(dna_template.as_bytes());
        hasher.update(energy.to_be_bytes());
        hasher.update(gen.to_be_bytes());
        let checksum = hex::encode(hasher.finalize());

        world
            .import_migrant(dna_template.clone(), energy, gen, &fingerprint, &checksum)
            .expect("Migration failed during surge");
    }

    assert_eq!(
        world.get_population_count(),
        200,
        "Surge migration failed to register all migrants"
    );
}

#[tokio::test]
async fn test_semantic_corruption_nan_dna() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).expect("Failed to create world");

    let fingerprint = config.fingerprint();

    // DNA with NaN weight (Semantic corruption)
    let corrupted_dna = hex::encode("{\"nodes\":[{\"id\":0,\"node_type\":\"Input\"},{\"id\":29,\"node_type\":\"Output\"}],\"connections\":[{\"from\":0,\"to\":29,\"weight\":NaN,\"enabled\":true,\"innovation\":1}]}");

    let result = world.import_migrant(corrupted_dna, 100.0, 1, &fingerprint, "checksum");

    // Should return error or ignore, NOT panic
    assert!(
        result.is_err() || world.get_population_count() == 0,
        "Corrupted DNA with NaN was accepted!"
    );
}

#[tokio::test]
async fn test_semantic_corruption_out_of_bounds_nodes() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).expect("Failed to create world");

    let fingerprint = config.fingerprint();

    // DNA with out of bounds node ID (47 is max)
    let corrupted_dna = hex::encode("{\"nodes\":[{\"id\":0,\"node_type\":\"Input\"},{\"id\":9999,\"node_type\":\"Output\"}],\"connections\":[{\"from\":0,\"to\":9999,\"weight\":1.0,\"enabled\":true,\"innovation\":1}]}");

    let result = world.import_migrant(corrupted_dna, 100.0, 1, &fingerprint, "checksum");

    assert!(
        result.is_err() || world.get_population_count() == 0,
        "Corrupted DNA with OOB node was accepted!"
    );
}
