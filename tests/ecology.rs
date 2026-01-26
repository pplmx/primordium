use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainType;
use primordium_lib::model::world::World;

#[test]
fn test_terrain_fertility_cycle() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 0; // Disable auto-spawn

    let mut world = World::new(1, config).expect("Failed to create world");
    let mut env = Environment::default();
    env.world_time = env.day_cycle_ticks / 4; // Midday to ensure high metabolism and light

    // Set entity position and stop movement
    world.entities[0].physics.x = 10.5;
    world.entities[0].physics.y = 10.5;
    world.entities[0].physics.vx = 0.0;
    world.entities[0].physics.vy = 0.0;
    world.entities[0].metabolism.trophic_potential = 0.0; // Herbivore leaning
    world.entities[0].metabolism.energy = 100.0;

    // Place food at same cell
    let ix = 10;
    let iy = 10;
    world.food.clear();
    world
        .food
        .push(primordium_lib::model::state::food::Food::new(ix, iy, 0.0));

    // 1. Check initial fertility
    world.terrain.set_fertility(ix, iy, 0.5);
    let initial_fertility = world.terrain.get_cell(ix, iy).fertility;

    let mut _food_eaten = false;
    let mut depleted_fertility = initial_fertility;
    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
        if world.food.is_empty() {
            _food_eaten = true;
            depleted_fertility = world.terrain.get_cell(ix, iy).fertility;
            break;
        }
    }

    assert!(
        depleted_fertility < initial_fertility,
        "Fertility should decrease after eating. Initial: {}, After: {}",
        initial_fertility,
        depleted_fertility
    );

    // 3. Test recovery over many ticks
    for _ in 0..100 {
        world.terrain.update(0.0);
    }
    let recovered_fertility = world.terrain.get_cell(ix, iy).fertility;
    assert!(
        recovered_fertility > depleted_fertility,
        "Fertility should recover over time"
    );
}

#[test]
fn test_barren_transition() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut world = World::new(0, config).expect("Failed to create world");

    let x = 5.5;
    let y = 5.5;
    let ix = 5;
    let iy = 5;

    // Ensure it's Plains first
    world.terrain.set_cell_type(ix, iy, TerrainType::Plains);

    // Manually deplete to nearly zero
    for _ in 0..10 {
        world.terrain.deplete(x, y, 0.1);
    }

    world.terrain.update(0.0);
    let terrain_type = world.terrain.get_cell(ix, iy).terrain_type;
    assert!(
        terrain_type == TerrainType::Barren || terrain_type == TerrainType::Desert,
        "Should turn barren or desert at low fertility, got {:?}",
        terrain_type
    );
}

#[test]
fn test_trophic_diet_restrictions() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    // 1. Herbivore specialist (trophic = 0.0) SHOULD eat plants
    let mut herbivore = primordium_lib::model::state::entity::Entity::new(10.5, 10.5, 0);
    herbivore.metabolism.trophic_potential = 0.0;
    herbivore.metabolism.energy = 50.0;
    // Align niche for maximum efficiency
    herbivore.intel.genotype.metabolic_niche = 0.0;
    world.entities.push(herbivore);
    world
        .food
        .push(primordium_lib::model::state::food::Food::new(10, 10, 0.0));

    world.update(&mut env).expect("Update failed");
    assert_eq!(
        world.food.len(),
        0,
        "Herbivore specialist should have eaten the plant"
    );

    // 2. Carnivore specialist (trophic = 1.0) should NOT eat plants
    world.entities.clear();
    world.food.clear();
    let mut carnivore = primordium_lib::model::state::entity::Entity::new(10.5, 10.5, 0);
    carnivore.physics.vx = 0.0;
    carnivore.physics.vy = 0.0;
    carnivore.metabolism.trophic_potential = 1.0;
    carnivore.metabolism.energy = 50.0;
    world.entities.push(carnivore);
    world
        .food
        .push(primordium_lib::model::state::food::Food::new(10, 10, 0.0));

    // Run ticks. Trophic efficiency for carnivore on plants is 1.0 - 1.0 = 0.0.
    for _ in 0..10 {
        world.update(&mut env).expect("Update failed");
    }
    assert_eq!(
        world.food.len(),
        1,
        "Carnivore specialist should not have eaten the plant"
    );
}

#[test]
fn test_light_dependent_food_growth() {
    let mut config = AppConfig::default();
    config.world.initial_food = 0;
    config.world.max_food = 100;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    // 1. Day time growth (Midday)
    env.world_time = env.day_cycle_ticks / 4;
    let mut day_food_count = 0;
    for _ in 0..1000 {
        world.update(&mut env).expect("Update failed");
        day_food_count += world.food.len();
        world.food.clear();
    }

    // 2. Night time growth
    env.world_time = env.day_cycle_ticks / 2 + 100; // Night
    let mut night_food_count = 0;
    for _ in 0..1000 {
        world.update(&mut env).expect("Update failed");
        night_food_count += world.food.len();
        world.food.clear();
    }

    assert!(
        day_food_count >= night_food_count,
        "More food should grow during day. Day: {}, Night: {}",
        day_food_count,
        night_food_count
    );
}
