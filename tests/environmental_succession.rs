use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;

#[test]
fn test_succession_and_carbon_cycle() {
    let mut config = AppConfig::default();
    config.world.width = 20;
    config.world.height = 20;
    config.world.initial_population = 10;

    let mut world = World::new(10, config).unwrap();
    let mut env = Environment::default();

    let initial_carbon = env.carbon_level;

    // Run for some ticks
    for _ in 0..100 {
        world.update(&mut env).unwrap();
    }

    // Carbon should have changed due to emissions (10 entities)
    assert!(
        env.carbon_level != initial_carbon,
        "Carbon level should change"
    );

    // Force many entities to increase carbon
    for _ in 0..100 {
        world
            .entities
            .push(primordium_lib::model::state::entity::Entity::new(
                10.0, 10.0, world.tick,
            ));
    }

    let carbon_before_boom = env.carbon_level;
    for _ in 0..50 {
        world.update(&mut env).unwrap();
    }
    assert!(
        env.carbon_level > carbon_before_boom,
        "High population should increase carbon emissions"
    );

    // Check for Forest transition
    // Manually set a cell to high fertility and biomass
    world.terrain.set_cell_type(5, 5, TerrainType::Plains);
    for _ in 0..1000 {
        world.terrain.add_biomass(5.0, 5.0, 10.0);
        world.update(&mut env).unwrap();
        if world.terrain.get_cell(5, 5).terrain_type == TerrainType::Forest {
            return; // Success
        }
    }
}

#[test]
fn test_biodiversity_hotspots() {
    let mut config = AppConfig::default();
    config.world.width = 50;
    config.world.height = 50;

    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // Inject 10 entities of different lineages into a small area
    for _ in 0..10 {
        let mut e = primordium_lib::model::state::entity::Entity::new(5.0, 5.0, 0);
        e.metabolism.lineage_id = uuid::Uuid::new_v4();
        e.physics.vx = 0.0;
        e.physics.vy = 0.0;
        e.metabolism.energy = 1000.0;
        world.entities.push(e);
    }

    // Run for 61 ticks (updates every 60)
    for _ in 0..61 {
        world.update(&mut env).unwrap();
    }

    assert!(
        world.pop_stats.biodiversity_hotspots >= 1,
        "Should detect at least one biodiversity hotspot, got {}",
        world.pop_stats.biodiversity_hotspots
    );
}
