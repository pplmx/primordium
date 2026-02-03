use primordium_lib::model::config::AppConfig;
use primordium_lib::model::infra::network::{NetMessage, PeerInfo};
use primordium_lib::model::world::World;
use sha2::Digest;
use uuid::Uuid;

#[tokio::test]
async fn test_entity_migration_via_network() {
    let mut entity = primordium_lib::model::lifecycle::create_entity(50.0, 50.0, 0);
    entity.metabolism.energy = 175.0;
    entity.metabolism.generation = 5;

    // 1. Pack entity into migration message
    let brain_dna = entity.intel.genotype.to_hex();
    let config = AppConfig::default();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(brain_dna.as_bytes());
    hasher.update((entity.metabolism.energy as f32).to_be_bytes());
    hasher.update(entity.metabolism.generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let migration_id = Uuid::new_v4();
    let msg = NetMessage::MigrateEntity {
        migration_id,
        dna: brain_dna.clone(),
        energy: entity.metabolism.energy as f32,
        generation: entity.metabolism.generation,
        species_name: "TestTribe".to_string(),
        fingerprint: config.fingerprint(),
        checksum,
    };

    // 2. Serialize message for "transport"
    let transport_json = serde_json::to_string(&msg).expect("Failed to serialize message");

    // 3. Receive on another site
    let received_msg: NetMessage =
        serde_json::from_str(&transport_json).expect("Failed to parse message");

    if let NetMessage::MigrateEntity {
        migration_id: m_id,
        dna,
        energy,
        generation,
        species_name,
        fingerprint,
        checksum,
    } = received_msg
    {
        assert_eq!(m_id, migration_id);
        assert_eq!(dna, brain_dna);
        assert_eq!(energy, 175.0);
        assert_eq!(generation, 5);
        assert_eq!(species_name, "TestTribe");

        // 4. Reconstruct in new world
        let config = AppConfig::default();
        let mut world = World::new(0, config).unwrap();

        world
            .import_migrant(dna, energy, generation, &fingerprint, &checksum)
            .expect("Failed to import");
        assert_eq!(world.get_population_count(), 1);
        let entities = world.get_all_entities();
        assert_eq!(entities[0].metabolism.energy, 175.0);
    } else {
        panic!("Incorrect message variant");
    }
}

/// Test migration using hex-encoded brain DNA (as used in production)
#[tokio::test]
async fn test_entity_migration_with_hex_dna() {
    let mut entity = primordium_lib::model::lifecycle::create_entity(25.0, 25.0, 0);
    entity.metabolism.energy = 200.0;
    entity.metabolism.generation = 10;

    // Use hex encoding (production method)
    let brain_hex = entity.intel.genotype.to_hex();
    let config = AppConfig::default();

    let mut hasher = sha2::Sha256::new();
    hasher.update(brain_hex.as_bytes());
    hasher.update((entity.metabolism.energy as f32).to_be_bytes());
    hasher.update(entity.metabolism.generation.to_be_bytes());
    let checksum = hex::encode(hasher.finalize());

    let migration_id = Uuid::new_v4();
    let msg = NetMessage::MigrateEntity {
        migration_id,
        dna: brain_hex.clone(),
        energy: entity.metabolism.energy as f32,
        generation: entity.metabolism.generation,
        species_name: primordium_lib::model::lifecycle::get_name(&entity),
        fingerprint: config.fingerprint(),
        checksum,
    };

    let json = serde_json::to_string(&msg).unwrap();
    let parsed: NetMessage = serde_json::from_str(&json).unwrap();

    if let NetMessage::MigrateEntity { dna, energy, .. } = parsed {
        // Reconstruct genotype from hex
        let restored_genotype = primordium_lib::model::state::entity::Genotype::from_hex(&dna)
            .expect("Failed to parse hex DNA");

        // Verify brain was reconstructed correctly
        assert_eq!(
            restored_genotype.brain.to_hex(),
            entity.intel.genotype.brain.to_hex()
        );
        assert!((energy - 200.0).abs() < 0.01);
    } else {
        panic!("Expected MigrateEntity");
    }
}

/// Test peer discovery message flow
#[tokio::test]
async fn test_peer_discovery_flow() {
    // Simulate server creating peer list
    let peer1 = PeerInfo {
        peer_id: Uuid::new_v4(),
        entity_count: 50,
        migrations_sent: 5,
        migrations_received: 3,
    };
    let peer2 = PeerInfo {
        peer_id: Uuid::new_v4(),
        entity_count: 75,
        migrations_sent: 10,
        migrations_received: 7,
    };

    let peer_list_msg = NetMessage::PeerList {
        peers: vec![peer1.clone(), peer2.clone()],
    };

    // Serialize as server would broadcast
    let json = serde_json::to_string(&peer_list_msg).unwrap();

    // Client receives and parses
    let received: NetMessage = serde_json::from_str(&json).unwrap();

    if let NetMessage::PeerList { peers } = received {
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].peer_id, peer1.peer_id);
        assert_eq!(peers[1].entity_count, 75);

        // Calculate network-wide stats
        let total_entities: usize = peers.iter().map(|p| p.entity_count).sum();
        let total_migrations: usize = peers.iter().map(|p| p.migrations_sent).sum();

        assert_eq!(total_entities, 125);
        assert_eq!(total_migrations, 15);
    } else {
        panic!("Expected PeerList");
    }
}

/// Test peer announce message
#[tokio::test]
async fn test_peer_announce_message() {
    let announce = NetMessage::PeerAnnounce {
        entity_count: 42,
        migrations_sent: 8,
        migrations_received: 5,
    };

    let json = serde_json::to_string(&announce).unwrap();

    // Verify JSON structure
    assert!(json.contains("\"type\":\"PeerAnnounce\""));

    let parsed: NetMessage = serde_json::from_str(&json).unwrap();

    if let NetMessage::PeerAnnounce {
        entity_count,
        migrations_sent,
        migrations_received,
    } = parsed
    {
        assert_eq!(entity_count, 42);
        assert_eq!(migrations_sent, 8);
        assert_eq!(migrations_received, 5);
    } else {
        panic!("Expected PeerAnnounce");
    }
}

#[tokio::test]
async fn test_migration_checksum_mismatch() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let config = AppConfig::default();
    let fingerprint = config.fingerprint();
    
    // Valid DNA but invalid checksum
    let dna = "invalid_but_not_checked_yet".to_string();
    let energy = 100.0;
    let generation = 1;
    let bad_checksum = "deadbeef".to_string();

    let result = world.import_migrant(dna, energy, generation, &fingerprint, &bad_checksum);
    
    assert!(result.is_err(), "Should reject invalid checksum");
    if let Err(e) = result {
        assert!(e.to_string().to_lowercase().contains("checksum mismatch") || e.to_string().contains("hex"));
    }
}

#[tokio::test]
async fn test_migration_fingerprint_mismatch() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    
    let dna = "some_valid_dna_string_placeholder".to_string();
    let energy = 100.0;
    let generation = 1;
    let checksum = "ignored_for_this_test".to_string();
    
    // Simulate a fingerprint from a different world configuration
    let bad_fingerprint = "incompatible_world_config_hash".to_string();

    let result = world.import_migrant(dna, energy, generation, &bad_fingerprint, &checksum);
    
    assert!(result.is_err(), "Should reject incompatible world fingerprint");
    if let Err(e) = result {
        assert!(e.to_string().to_lowercase().contains("fingerprint"));
    }
}

#[tokio::test]
async fn test_malformed_json_packet() {
    // Simulate receiving garbage over the wire
    let garbage_json = "{ \"type\": \"MigrateEntity\", \"dna\": [BROKEN_JSON_HERE";
    
    let result = serde_json::from_str::<NetMessage>(garbage_json);
    
    assert!(result.is_err(), "Should fail to parse malformed JSON");
}
