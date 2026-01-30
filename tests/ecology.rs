use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;

#[test]
fn test_terrain_fertility_cycle() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();
    let ix = 10;
    let iy = 10;
    let mut e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    e.metabolism.trophic_potential = 0.0;
    e.metabolism.energy = 100.0;
    world.spawn_entity(e);
    world.terrain.set_fertility(ix, iy, 0.5);
    world.ecs.spawn((
        primordium_lib::model::state::food::Food::new(ix, iy, 0.0),
        primordium_lib::model::state::Position {
            x: ix as f64,
            y: iy as f64,
        },
        primordium_lib::model::state::MetabolicNiche(0.0),
    ));
    world.food_dirty = true;
    world.config.ecosystem.soil_depletion_unit = 0.5;
    world.update(&mut env).expect("Update failed");
    let after_fertility = world.terrain.get_cell(ix, iy).fertility;
    assert!(after_fertility < 0.5);
}

#[test]
fn test_barren_transition() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let ix = 5;
    let iy = 5;
    world.terrain.set_cell_type(ix, iy, TerrainType::Plains);
    for _ in 0..10 {
        world.terrain.deplete(5.5, 5.5, 0.1);
    }
    world.terrain.update(0.0, 0, 42);
    let terrain_type = world.terrain.get_cell(ix, iy).terrain_type;
    assert!(terrain_type == TerrainType::Barren || terrain_type == TerrainType::Desert);
}

#[test]
fn test_trophic_diet_restrictions() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut env = Environment::default();

    {
        let mut world = World::new(0, config.clone()).expect("Failed to create world");
        let mut herbivore = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
        herbivore.metabolism.trophic_potential = 0.0;
        herbivore.metabolism.energy = 50.0;
        herbivore.intel.genotype.metabolic_niche = 0.5;
        world.spawn_entity(herbivore);
        world.ecs.spawn((
            primordium_lib::model::state::food::Food::new(10, 10, 0.5),
            primordium_lib::model::state::Position { x: 10.0, y: 10.0 },
            primordium_lib::model::state::MetabolicNiche(0.5),
        ));
        world.food_dirty = true;
        world.config.ecosystem.soil_depletion_unit = 0.5;
        world.update(&mut env).expect("Update failed");

        assert_eq!(world.get_food_count(), 0);
    }

    {
        let mut world = World::new(0, config).expect("Failed to create world");
        let mut carnivore = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
        carnivore.metabolism.trophic_potential = 1.0;
        carnivore.metabolism.energy = 50.0;
        world.spawn_entity(carnivore);
        world.ecs.spawn((
            primordium_lib::model::state::food::Food::new(10, 10, 0.0),
            primordium_lib::model::state::Position { x: 10.0, y: 10.0 },
            primordium_lib::model::state::MetabolicNiche(0.0),
        ));
        world.food_dirty = true;
        for _ in 0..10 {
            world.update(&mut env).expect("Update failed");
        }
        assert_eq!(world.get_food_count(), 1);
    }
}

#[test]
fn test_light_dependent_food_growth() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 1000;
    let mut day_food_count = 0;
    {
        let mut world = World::new(0, config.clone()).expect("Failed to create world");
        let mut env = Environment::default();
        for _ in 0..1000 {
            env.world_time = env.day_cycle_ticks / 4;
            world.update(&mut env).expect("Update failed");
            day_food_count += world.get_food_count();
            let mut food_handles = Vec::new();
            for (h, _) in world
                .ecs
                .query::<&primordium_lib::model::state::Food>()
                .iter()
            {
                food_handles.push(h);
            }
            for h in food_handles {
                let _ = world.ecs.despawn(h);
            }
        }
    }
    let mut night_food_count = 0;
    {
        let mut world = World::new(0, config).expect("Failed to create world");
        let mut env = Environment::default();
        for _ in 0..1000 {
            env.world_time = env.day_cycle_ticks / 2 + 100;
            world.update(&mut env).expect("Update failed");
            night_food_count += world.get_food_count();
            let mut food_handles = Vec::new();
            for (h, _) in world
                .ecs
                .query::<&primordium_lib::model::state::Food>()
                .iter()
            {
                food_handles.push(h);
            }
            for h in food_handles {
                let _ = world.ecs.despawn(h);
            }
        }
    }
    assert!(day_food_count > night_food_count);
}
