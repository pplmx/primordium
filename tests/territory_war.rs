use primordium_data::{Specialization, TerrainType};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use uuid::Uuid;

#[tokio::test]
async fn test_coordinated_outpost_siege() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    // Lineage A: Outpost at (10, 10)
    let id_a = Uuid::new_v4();
    let idx_a = world.terrain.index(10, 10);
    world.terrain.set_cell_type(10, 10, TerrainType::Outpost);
    world.terrain.cells[idx_a].owner_id = Some(id_a);
    world.terrain.cells[idx_a].energy_store = 500.0;

    // Lineage B: Soldiers attacking at (11, 11)
    let id_b = Uuid::new_v4();
    for i in 0..5 {
        let mut soldier = lifecycle::create_entity(11.0 + (i as f64 * 0.1), 11.0, 0);
        soldier.metabolism.lineage_id = id_b;
        soldier.intel.specialization = Some(Specialization::Soldier);
        soldier.metabolism.energy = 200.0;
        // Force aggression against different lineage
        soldier
            .intel
            .genotype
            .brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 5, // Tribe density
                to: 32,  // Aggro
                weight: 10.0,
                enabled: true,
                innovation: 5555,
            });
        world.spawn_entity(soldier);
    }

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
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    let id_my = Uuid::new_v4();
    let id_enemy = Uuid::new_v4();

    // 1. Setup our outpost
    let idx = world.terrain.index(25, 25);
    world.terrain.set_cell_type(25, 25, TerrainType::Outpost);
    world.terrain.cells[idx].owner_id = Some(id_my);

    // 2. Spawn a guardian soldier
    let mut guardian = lifecycle::create_entity(24.5, 24.5, 0);
    guardian.metabolism.lineage_id = id_my;
    guardian.intel.specialization = Some(Specialization::Soldier);
    world.spawn_entity(guardian);

    // 3. Spawn an enemy nearby
    let mut enemy = lifecycle::create_entity(25.5, 25.5, 0);
    enemy.metabolism.lineage_id = id_enemy;
    enemy.metabolism.energy = 50.0; // Weak
    world.spawn_entity(enemy);

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
