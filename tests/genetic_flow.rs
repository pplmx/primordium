use primordium_lib::model::brain::{Brain, BrainLogic};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;

#[test]
fn test_dna_hex_roundtrip() {
    let brain = Brain::new_random();
    let hex = brain.to_hex();

    // Ensure hex is valid and not empty
    assert!(!hex.is_empty());

    let recovered_brain = Brain::from_hex(&hex).expect("Failed to recover brain from hex");

    // Test a few connections to ensure consistency
    assert_eq!(brain.connections.len(), recovered_brain.connections.len());
    for i in 0..5 {
        assert!(
            (brain.connections[i].weight - recovered_brain.connections[i].weight).abs() < 0.001
        );
    }
}

#[test]
fn test_entity_import_export() {
    let config = AppConfig::default();
    let mut world = World::new(1, config).expect("Failed to create world");

    let original_entity = world.get_all_entities()[0].clone();
    let dna = original_entity.intel.genotype.to_hex();

    // Manual import to world
    let fingerprint = world.config.fingerprint();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(100.0f32.to_be_bytes());
    hasher.update(5u32.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let _ = world.import_migrant(dna.clone(), 100.0, 5, &fingerprint, &checksum);

    let entities = world.get_all_entities();
    let imported = entities
        .iter()
        .find(|e| e.metabolism.generation == 5)
        .expect("Imported entity not found");

    assert_eq!(imported.metabolism.energy, 100.0);
    assert_eq!(imported.metabolism.generation, 5);
    // Genotypes should match
    assert_eq!(imported.intel.genotype.to_hex(), dna);
}

#[test]
fn test_genetic_surge() {
    let config = AppConfig::default();
    let mut world = World::new(10, config).expect("Failed to create world");

    let before_surge_dnas: Vec<String> = world
        .get_all_entities()
        .iter()
        .map(|e| e.intel.genotype.to_hex())
        .collect();

    // Simulate surge (same logic as in app/input.rs)
    use primordium_lib::model::systems::intel;
    let mut rng = rand::thread_rng();
    for (_handle, (intel, _met, _phys)) in world.ecs.query_mut::<(
        &mut primordium_lib::model::state::Intel,
        &mut primordium_lib::model::state::Metabolism,
        &mut primordium_lib::model::state::Physics,
    )>() {
        intel::mutate_genotype(
            &mut intel.genotype,
            &world.config,
            10,
            false,
            None,
            &mut rng,
            None,
            0.0,
        );
    }

    let after_surge_dnas: Vec<String> = world
        .get_all_entities()
        .iter()
        .map(|e| e.intel.genotype.to_hex())
        .collect();

    for i in 0..10 {
        assert_ne!(
            before_surge_dnas[i], after_surge_dnas[i],
            "Entity {} DNA did not change after surge",
            i
        );
    }
}
