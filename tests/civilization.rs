use primordium_core::systems::{civilization, intel};
use primordium_data::TerrainType;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;
use uuid::Uuid;

#[tokio::test]
async fn test_global_event_radiation_surge() {
    let world = World::new(0, AppConfig::default()).unwrap();
    let env = Environment {
        radiation_timer: 500,
        ..Default::default()
    };

    assert!(env.is_radiation_storm());

    let mut genotype = std::sync::Arc::new(
        primordium_lib::model::brain::create_genotype_random_with_rng(&mut rand::thread_rng()),
    );
    let original_dna = genotype.to_hex();

    let mut rng = rand::thread_rng();
    intel::mutate_genotype(
        &mut genotype,
        &intel::MutationParams {
            config: &world.config,
            population: 100,
            is_radiation_storm: true,
            specialization: None,
            ancestral_genotype: None,
            stress_factor: 0.0,
        },
        &mut rng,
    );

    assert_ne!(genotype.to_hex(), original_dna);
}

#[tokio::test]
async fn test_civilization_leveling_outposts() {
    let mut world = World::new(0, AppConfig::default()).unwrap();
    let l_id = Uuid::new_v4();

    for i in 0..5 {
        let idx = world.terrain.index(i as u16, 0);
        let terrain = std::sync::Arc::make_mut(&mut world.terrain);
        terrain.set_cell_type(i as u16, 0, TerrainType::Outpost);
        terrain.cells[idx].owner_id = Some(l_id);
    }

    let outpost_counts = civilization::count_outposts_by_lineage(&world.terrain);
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
