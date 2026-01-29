use primordium_data::GeneType;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;

#[test]
fn test_genetic_edit_application() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    let mut entity = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let id = entity.id;
    entity.intel.genotype.sensing_range = 10.0;
    entity.intel.genotype.max_speed = 1.0;
    entity.intel.genotype.trophic_potential = 0.0;

    world.entities.push(entity);

    // 1. Edit Sensing
    world.apply_genetic_edit(id, GeneType::Sensing, 5.0);
    let e = world.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.intel.genotype.sensing_range, 15.0);
    assert_eq!(e.physics.sensing_range, 15.0);

    // 2. Edit Speed
    world.apply_genetic_edit(id, GeneType::Speed, 0.5);
    let e = world.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(e.intel.genotype.max_speed, 1.5);
    assert_eq!(e.physics.max_speed, 1.5);

    // 3. Edit Trophic
    world.apply_genetic_edit(id, GeneType::Trophic, 0.2);
    let e = world.entities.iter().find(|e| e.id == id).unwrap();
    assert!((e.intel.genotype.trophic_potential - 0.2).abs() < 0.01);
    assert!((e.metabolism.trophic_potential - 0.2).abs() < 0.01);
}
