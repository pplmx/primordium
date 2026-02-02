use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::systems::civilization;
use primordium_lib::model::terrain::{OutpostSpecialization, TerrainType};
use primordium_lib::model::world::World;
use uuid::Uuid;

#[tokio::test] async
fn test_contested_ownership() {
    let mut config = AppConfig::default();
    config.world.width = 50;
    config.world.height = 50;
    let mut world = World::new(0, config).unwrap();

    let lineage_a = Uuid::new_v4();
    let lineage_b = Uuid::new_v4();

    let ox = 25;
    let oy = 25;
    let idx = world.terrain.index(ox, oy);
    world.terrain.set_cell_type(ox, oy, TerrainType::Outpost);
    world.terrain.cells[idx].owner_id = Some(lineage_a);
    world.terrain.cells[idx].energy_store = 100.0;

    let mut weak_defender = lifecycle::create_entity(25.0, 25.0, 0);
    weak_defender.metabolism.lineage_id = lineage_a;
    weak_defender.metabolism.energy = 30.0;
    world.ecs.spawn((
        weak_defender.identity,
        weak_defender.position,
        weak_defender.velocity,
        weak_defender.appearance,
        weak_defender.physics,
        weak_defender.metabolism,
        weak_defender.health,
        weak_defender.intel,
    ));

    for _ in 0..3 {
        let mut strong_invader = lifecycle::create_entity(26.0, 26.0, 0);
        strong_invader.metabolism.lineage_id = lineage_b;
        strong_invader.metabolism.energy = 100.0;
        world.ecs.spawn((
            strong_invader.identity,
            strong_invader.position,
            strong_invader.velocity,
            strong_invader.appearance,
            strong_invader.physics,
            strong_invader.metabolism,
            strong_invader.health,
            strong_invader.intel,
        ));
    }

    world.prepare_spatial_hash();
    world.capture_entity_snapshots();

    civilization::resolve_contested_ownership(
        &mut world.terrain,
        world.width,
        world.height,
        &world.spatial_hash,
        &world.entity_snapshots,
        &world.lineage_registry,
    );

    assert_eq!(
        world.terrain.cells[idx].owner_id,
        Some(lineage_b),
        "Ownership should transfer to lineage_b"
    );
    assert!(
        world.terrain.cells[idx].energy_store < 100.0,
        "Energy store should be reduced during transition"
    );
    assert_eq!(
        world.terrain.cells[idx].outpost_spec,
        OutpostSpecialization::Standard,
        "Specialization should reset to Standard"
    );
}

#[tokio::test] async
fn test_outpost_upgrades() {
    let mut config = AppConfig::default();
    config.world.width = 50;
    config.world.height = 50;
    let mut world = World::new(0, config).unwrap();

    let l_id = Uuid::new_v4();
    world.lineage_registry.record_birth(l_id, 0, 0);

    if let Some(record) = world.lineage_registry.lineages.get_mut(&l_id) {
        record.civilization_level = 2;
    }

    let ox = 10;
    let oy = 10;
    let idx = world.terrain.index(ox, oy);
    world.terrain.set_cell_type(ox, oy, TerrainType::Outpost);
    world.terrain.cells[idx].owner_id = Some(l_id);
    world.terrain.cells[idx].energy_store = 500.0;

    for _ in 0..3 {
        let mut healthy_entity = lifecycle::create_entity(11.0, 11.0, 0);
        healthy_entity.metabolism.lineage_id = l_id;
        healthy_entity.metabolism.energy = 80.0;
        world.ecs.spawn((
            healthy_entity.identity,
            healthy_entity.position,
            healthy_entity.velocity,
            healthy_entity.appearance,
            healthy_entity.physics,
            healthy_entity.metabolism,
            healthy_entity.health,
            healthy_entity.intel,
        ));
    }

    world.prepare_spatial_hash();
    world.capture_entity_snapshots();

    civilization::resolve_outpost_upgrades(
        &mut world.terrain,
        world.width,
        world.height,
        &world.spatial_hash,
        &world.entity_snapshots,
        &world.lineage_registry,
    );

    assert_eq!(
        world.terrain.cells[idx].outpost_spec,
        OutpostSpecialization::Silo,
        "Should upgrade to Silo for healthy tribe"
    );

    world.terrain.cells[idx].outpost_spec = OutpostSpecialization::Standard;
    world.terrain.cells[idx].energy_store = 500.0;

    world.ecs = hecs::World::new();
    for _ in 0..3 {
        let mut weak_entity = lifecycle::create_entity(11.0, 11.0, 0);
        weak_entity.metabolism.lineage_id = l_id;
        weak_entity.metabolism.energy = 20.0;
        world.ecs.spawn((
            weak_entity.identity,
            weak_entity.position,
            weak_entity.velocity,
            weak_entity.appearance,
            weak_entity.physics,
            weak_entity.metabolism,
            weak_entity.health,
            weak_entity.intel,
        ));
    }

    world.prepare_spatial_hash();
    world.capture_entity_snapshots();

    civilization::resolve_outpost_upgrades(
        &mut world.terrain,
        world.width,
        world.height,
        &world.spatial_hash,
        &world.entity_snapshots,
        &world.lineage_registry,
    );

    assert_eq!(
        world.terrain.cells[idx].outpost_spec,
        OutpostSpecialization::Nursery,
        "Should upgrade to Nursery for weak tribe"
    );
}
