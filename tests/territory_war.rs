mod common;
use common::{EntityBuilder, TestBehavior, WorldBuilder};
use primordium_data::Specialization;
use primordium_data::TerrainType;
use uuid::Uuid;

#[tokio::test]
async fn test_coordinated_outpost_siege() {
    let id_a = Uuid::new_v4();
    let id_b = Uuid::new_v4();

    let mut world_builder = WorldBuilder::new().with_outpost(10, 10, id_a);

    // Lineage B: Soldiers attacking at (11, 11)
    for i in 0..5 {
        let soldier = EntityBuilder::new()
            .at(11.0 + (i as f64 * 0.1), 11.0)
            .lineage(id_b)
            .specialization(Specialization::Soldier)
            .energy(200.0)
            .with_behavior(TestBehavior::SiegeSoldier)
            .build();
        world_builder = world_builder.with_entity(soldier);
    }

    let (mut world, mut env) = world_builder.build();

    // Run for multiple ticks
    for _ in 0..20 {
        world.update(&mut env).unwrap();
    }

    // Check if the outpost was damaged or destroyed
    let cell = world.terrain.get(10.0, 10.0);
    assert!(
        cell.energy_store < 500.0 || cell.terrain_type != TerrainType::Outpost,
        "Outpost should have been damaged by enemy soldiers"
    );
}

#[tokio::test]
async fn test_soldier_guardian_prioritization() {
    let id_my = Uuid::new_v4();
    let id_enemy = Uuid::new_v4();

    let guardian = EntityBuilder::new()
        .at(24.5, 24.5)
        .lineage(id_my)
        .specialization(Specialization::Soldier)
        .build();

    let enemy = EntityBuilder::new()
        .at(25.5, 25.5)
        .lineage(id_enemy)
        .energy(50.0) // Weak
        .build();

    let (mut world, mut env) = WorldBuilder::new()
        .with_outpost(25, 25, id_my)
        .with_entity(guardian)
        .with_entity(enemy)
        .build();

    // 4. Run update
    world.update(&mut env).unwrap();

    // 5. Verify the enemy was attacked (energy loss or death)
    let entities = world.get_all_entities();
    let enemy_rem = entities
        .iter()
        .find(|e| e.metabolism.lineage_id == id_enemy);

    if let Some(e) = enemy_rem {
        assert!(
            e.metabolism.energy < 50.0,
            "Guardian soldier should have attacked the enemy intruder"
        );
    } else {
        // Enemy killed, which is also a pass
    }
}
