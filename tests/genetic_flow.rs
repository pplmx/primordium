use primordium_lib::model::brain::Brain;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;

#[test]
fn test_dna_hex_roundtrip() {
    let brain = Brain::new_random();
    let hex = brain.to_hex();

    // Ensure hex is valid and not empty
    assert!(!hex.is_empty());

    let recovered_brain = Brain::from_hex(&hex).expect("Failed to recover brain from hex");

    // Test a few weights to ensure consistency
    assert_eq!(brain.weights_ih.len(), recovered_brain.weights_ih.len());
    for i in 0..5 {
        assert!((brain.weights_ih[i] - recovered_brain.weights_ih[i]).abs() < 0.001);
    }
}

#[test]
fn test_entity_import_export() {
    let config = AppConfig::default();
    let mut world = World::new(1, config).expect("Failed to create world");

    let original_entity = world.entities[0].clone();
    let dna = original_entity.intel.genotype.to_hex();

    // Manual import to world
    world.import_migrant(dna.clone(), 100.0, 5);

    let imported = world
        .entities
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
        .entities
        .iter()
        .map(|e| e.intel.genotype.to_hex())
        .collect();

    // Simulate surge (same logic as in app/input.rs)
    use primordium_lib::model::systems::intel;
    for entity in &mut world.entities {
        intel::mutate_genotype(&mut entity.intel.genotype, &world.config.evolution);
    }

    let after_surge_dnas: Vec<String> = world
        .entities
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
