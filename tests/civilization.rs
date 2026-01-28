use primordium_data::AncestralTrait;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;
use uuid::Uuid;

#[test]
fn test_ancestral_trait_metabolism_buff() {
    let config = AppConfig::default();
    let world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    let l_id = Uuid::new_v4();
    let mut e = Entity::new(10.0, 10.0, 0);
    e.metabolism.lineage_id = l_id;
    e.intel
        .ancestral_traits
        .insert(AncestralTrait::HardenedMetabolism);

    let initial_energy = 100.0;
    e.metabolism.energy = initial_energy;

    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &world.terrain,
        snapshots: &[],
        entity_id_map: &std::collections::HashMap::new(),
        spatial_hash: &world.spatial_hash,
        pressure: &world.pressure,
        width: 100,
        height: 50,
    };

    let outputs = [0.0; 12];
    action_system(&mut e, outputs, &mut ctx);

    let drain_with_trait = initial_energy - e.metabolism.energy;

    let mut e_normal = Entity::new(10.0, 10.0, 0);
    e_normal.metabolism.energy = initial_energy;
    action_system(&mut e_normal, outputs, &mut ctx);

    let drain_normal = initial_energy - e_normal.metabolism.energy;

    assert!(
        drain_with_trait < drain_normal,
        "HardenedMetabolism should reduce energy drain"
    );
}

#[test]
fn test_global_event_radiation_surge() {
    let world = World::new(0, AppConfig::default()).unwrap();
    let env = Environment {
        radiation_timer: 500,
        ..Default::default()
    };

    assert!(env.is_radiation_storm());

    let mut genotype = primordium_lib::model::state::entity::Genotype::new_random();
    let original_dna = genotype.to_hex();

    use primordium_lib::model::systems::intel;
    let mut rng = rand::thread_rng();
    intel::mutate_genotype(
        &mut genotype,
        &world.config,
        100,
        true,
        None,
        &mut rng,
        None,
    );

    assert_ne!(genotype.to_hex(), original_dna);
}

#[test]
fn test_civilization_leveling_outposts() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let l_id = Uuid::new_v4();

    for i in 0..5 {
        let idx = world.terrain.index(i as u16, 0);
        world
            .terrain
            .set_cell_type(i as u16, 0, TerrainType::Outpost);
        world.terrain.cells[idx].owner_id = Some(l_id);
    }

    let outpost_counts =
        primordium_lib::model::systems::civilization::count_outposts_by_lineage(&world.terrain);
    assert_eq!(outpost_counts.get(&l_id), Some(&5));

    world.lineage_registry.record_birth(l_id, 0, 0);
    world
        .lineage_registry
        .check_goals(0, &[], 100, 50, &outpost_counts);

    let record = world.lineage_registry.lineages.get(&l_id).unwrap();
    assert_eq!(
        record.civilization_level, 1,
        "Lineage should reach Civilization Level 1"
    );
}
