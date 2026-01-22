use primordium_lib::model::config::AppConfig;
use primordium_lib::model::infra::network::NetMessage;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::world::World;

#[test]
fn test_entity_migration_via_network() {
    let mut entity = Entity::new(50.0, 50.0, 0);
    entity.metabolism.energy = 175.0;
    entity.metabolism.generation = 5;

    // 1. Pack entity into migration message
    let brain_dna = serde_json::to_string(&entity.intel.brain).unwrap();
    let msg = NetMessage::MigrateEntity {
        dna: brain_dna.clone(),
        energy: entity.metabolism.energy as f32,
        generation: entity.metabolism.generation,
        species_name: "TestTribe".to_string(),
    };

    // 2. Serialize message for "transport"
    let transport_json = serde_json::to_string(&msg).expect("Failed to serialize message");

    // 3. Receive on another site
    let received_msg: NetMessage =
        serde_json::from_str(&transport_json).expect("Failed to parse message");

    if let NetMessage::MigrateEntity {
        dna,
        energy,
        generation,
        species_name,
    } = received_msg
    {
        assert_eq!(dna, brain_dna);
        assert_eq!(energy, 175.0);
        assert_eq!(generation, 5);
        assert_eq!(species_name, "TestTribe");

        // 4. Reconstruct in new world
        let config = AppConfig::default();
        let mut world = World::new(0, config).unwrap();

        let mut new_entity = Entity::new(0.0, 0.0, 100);
        new_entity.intel.brain = serde_json::from_str(&dna).unwrap();
        new_entity.metabolism.energy = energy as f64;
        new_entity.metabolism.generation = generation;

        world.entities.push(new_entity);
        assert_eq!(world.entities.len(), 1);
        assert_eq!(world.entities[0].metabolism.energy, 175.0);
    } else {
        panic!("Incorrect message variant");
    }
}
