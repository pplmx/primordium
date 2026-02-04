use primordium_data::Specialization;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;
use uuid::Uuid;

#[tokio::test]
async fn test_collective_memory_reinforcement() {
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    let mut _env = Environment::default();

    let l_id = Uuid::new_v4();
    let mut e1 = lifecycle::create_entity(10.0, 10.0, 0);
    e1.metabolism.lineage_id = l_id;
    e1.intel.genotype.lineage_id = l_id;
    world.spawn_entity(e1);

    world.lineage_registry.record_birth(l_id, 0, 0);

    world
        .lineage_registry
        .boost_memory_value(&l_id, "goal", 1.0);

    assert!(world.lineage_registry.get_memory_value(&l_id, "goal") > 0.0);

    world.lineage_registry.decay_memory(0.5);
    assert_eq!(world.lineage_registry.get_memory_value(&l_id, "goal"), 0.5);
}

#[tokio::test]
async fn test_engineer_biological_irrigation_pressure() {
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    world.terrain.set_cell_type(10, 10, TerrainType::River);
    world.terrain.set_cell_type(11, 10, TerrainType::Plains);

    let mut eng = lifecycle::create_entity(11.0, 10.0, 0);
    eng.velocity.vx = 0.0;
    eng.velocity.vy = 0.0;
    eng.physics.max_speed = 0.0;
    eng.intel.genotype.max_speed = 0.0;
    eng.intel.specialization = Some(Specialization::Engineer);
    eng.metabolism.has_metamorphosed = true;
    world.spawn_entity(eng);

    world.update(&mut env).unwrap();

    let (d, _b) = world.pressure.sense(11.0, 10.0, 1.5);
    assert!(d > 0.0, "Engineer should deposit Dig pressure near river");
}

#[tokio::test]
async fn test_outpost_construction() {
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    let mut alpha = lifecycle::create_entity(10.0, 10.0, 0);
    alpha.intel.rank = 0.9;
    alpha.metabolism.energy = 500.0;
    alpha.metabolism.has_metamorphosed = true;
    let l_id = Uuid::new_v4();
    alpha.metabolism.lineage_id = l_id;

    let handle = world.spawn_entity(alpha);

    use primordium_core::systems::interaction;
    use primordium_lib::model::state::interaction::InteractionCommand;

    let mut lineage_cons = Vec::new();
    let mut rng = rand::thread_rng();

    let mut ctx = interaction::InteractionContext {
        terrain: &mut world.terrain,
        env: &mut env,
        pop_stats: &mut world.pop_stats,
        lineage_registry: &mut world.lineage_registry,
        fossil_registry: &mut world.fossil_registry,
        config: &world.config,
        tick: 0,
        width: world.width,
        height: world.height,
        social_grid: &mut world.social_grid,
        lineage_consumption: &mut lineage_cons,
        food_handles: &[],
        spatial_hash: &world.spatial_hash,
        rng: &mut rng,
        food_count: &world.food_count,
        world_seed: 0,
    };

    let cmd = InteractionCommand::Build {
        x: 10.0,
        y: 10.0,
        attacker_idx: 0,
        is_nest: false,
        is_outpost: true,
        outpost_spec: Some(primordium_data::OutpostSpecialization::Standard),
    };

    interaction::process_interaction_commands_ecs(&mut world.ecs, &[handle], vec![cmd], &mut ctx);

    let cell = world.terrain.get(10.0, 10.0);
    assert_eq!(cell.terrain_type, TerrainType::Outpost);
    assert_eq!(cell.owner_id, Some(l_id));
}

#[tokio::test]
async fn test_outpost_energy_capacitor() {
    let config = AppConfig::default();
    let mut world = World::new(0, config).unwrap();
    let mut _env = Environment::default();

    let l_id = Uuid::new_v4();
    let mut donor = lifecycle::create_entity(10.0, 10.0, 0);
    donor.metabolism.lineage_id = l_id;
    donor.metabolism.energy = 450.0;
    donor.metabolism.max_energy = 500.0;
    world.spawn_entity(donor);

    let idx = world.terrain.index(10, 10);
    world.terrain.set_cell_type(10, 10, TerrainType::Outpost);
    world.terrain.cells[idx].owner_id = Some(l_id);

    world.update(&mut _env).unwrap();

    let idx = world.terrain.index(10, 10);
    assert!(
        world.terrain.cells[idx].energy_store > 0.0,
        "Outpost should collect energy"
    );

    let mut needy = lifecycle::create_entity(11.0, 11.0, 0);
    needy.metabolism.lineage_id = l_id;
    needy.metabolism.energy = 20.0;
    needy.metabolism.max_energy = 500.0;
    world.spawn_entity(needy);

    world.update(&mut _env).unwrap();

    let entities = world.get_all_entities();
    let needy_entity = entities
        .iter()
        .find(|e| e.metabolism.energy > 20.0 && e.metabolism.energy < 400.0);
    assert!(
        needy_entity.is_some(),
        "Needy entity should have received energy"
    );
}
