use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_migration_empty_dna() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy = 100.0;
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let result = world.import_migrant("".to_string(), energy, generation, &fingerprint, &checksum);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("cannot be empty"));
    }
}

#[tokio::test]
async fn test_migration_whitespace_dna() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy = 100.0;
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let result = world.import_migrant(
        "   \t\n   ".to_string(),
        energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("cannot be empty"));
    }
}

#[tokio::test]
async fn test_migration_non_hex_dna() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy = 100.0;
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let result = world.import_migrant(
        "GHIJ1234".to_string(),
        energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("non-hex") || e.to_string().contains("hex"));
    }
}

#[tokio::test]
async fn test_migration_odd_length_dna() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy = 100.0;
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let result = world.import_migrant(
        "ABC".to_string(),
        energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("odd number"));
    }
}

#[tokio::test]
async fn test_migration_dna_with_whitespace_prefix_suffix() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy: f32 = 200.0;
    let generation: u32 = 5;

    let entity = primordium_lib::model::lifecycle::create_entity(50.0, 50.0, 0);
    let dna = entity.intel.genotype.to_hex();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(energy.to_be_bytes());
    hasher.update(generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let dna_with_whitespace = format!("  {}  ", dna);
    let result = world.import_migrant(
        dna_with_whitespace,
        energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_ok());
    assert_eq!(world.get_population_count(), 1);
}

#[tokio::test]
async fn test_migration_nan_energy() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let nan_energy: f32 = f32::NAN;
    let result = world.import_migrant(
        "ABCD12".to_string(),
        nan_energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("finite")
                || e.to_string().to_lowercase().contains("nan")
        );
    }
}

#[tokio::test]
async fn test_migration_infinity_energy() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let inf_energy: f32 = f32::INFINITY;
    let result = world.import_migrant(
        "ABCD12".to_string(),
        inf_energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("finite")
                || e.to_string().to_lowercase().contains("infinity")
        );
    }
}

#[tokio::test]
async fn test_migration_negative_infinity_energy() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let generation = 1;
    let checksum = "deadbeef".to_string();

    let neg_inf_energy: f32 = f32::NEG_INFINITY;
    let result = world.import_migrant(
        "ABCD12".to_string(),
        neg_inf_energy,
        generation,
        &fingerprint,
        &checksum,
    );

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(
            e.to_string().to_lowercase().contains("finite")
                || e.to_string().to_lowercase().contains("infinity")
        );
    }
}

#[tokio::test]
async fn test_migration_valid_negative_energy() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let generation: u32 = 1;

    let entity = primordium_lib::model::lifecycle::create_entity(50.0, 50.0, 0);
    let dna = entity.intel.genotype.to_hex();

    let energy = -50.0_f32;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(energy.to_be_bytes());
    hasher.update(generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let result = world.import_migrant(dna, energy, generation, &fingerprint, &checksum);

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_migration_valid_zero_energy() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let generation: u32 = 1;

    let entity = primordium_lib::model::lifecycle::create_entity(50.0, 50.0, 0);
    let dna = entity.intel.genotype.to_hex();

    let energy = 0.0_f32;
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(energy.to_be_bytes());
    hasher.update(generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let result = world.import_migrant(dna, energy, generation, &fingerprint, &checksum);

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_migration_very_large_generation() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let fingerprint = AppConfig::default().fingerprint();
    let energy: f32 = 100.0;

    let entity = primordium_lib::model::lifecycle::create_entity(50.0, 50.0, 0);
    let dna = entity.intel.genotype.to_hex();

    let generation = u32::MAX;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(dna.as_bytes());
    hasher.update(energy.to_be_bytes());
    hasher.update(generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let result = world.import_migrant(dna, energy, generation, &fingerprint, &checksum);

    assert!(result.is_ok());
}
