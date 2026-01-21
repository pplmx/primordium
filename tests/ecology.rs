use primordium_lib::model::config::AppConfig;
use primordium_lib::model::entity::EntityRole;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::terrain::TerrainType;
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
    world.entities[0].x = 10.5;
    world.entities[0].y = 10.5;
    world.entities[0].vx = 0.0;
    world.entities[0].vy = 0.0;
    world.entities[0].role = EntityRole::Herbivore;
    world.entities[0].energy = 100.0;

    // Place food at same cell
    let ix = 10;
    let iy = 10;
    world.food.clear();
    world
        .food
        .push(primordium_lib::model::food::Food::new(ix, iy));

    // 1. Check initial fertility
    let initial_fertility = world.terrain.get_cell(ix, iy).fertility;

    // 2. Run update until food is eaten
    let mut food_eaten = false;
    for _ in 0..50 {
        world.update(&env).expect("Update failed");
        if world.food.is_empty() {
            food_eaten = true;
            break;
        }
    }

    assert!(food_eaten, "Food should have been eaten by herbivore");

    let depleted_fertility = world.terrain.get_cell(ix, iy).fertility;
    println!(
        "Initial: {}, Depleted: {}, Food count: {}",
        initial_fertility,
        depleted_fertility,
        world.food.len()
    );
    assert!(
        depleted_fertility < initial_fertility,
        "Fertility should decrease after eating. Initial: {}, After: {}",
        initial_fertility,
        depleted_fertility
    );

    // 3. Test recovery over many ticks
    for _ in 0..100 {
        world.terrain.update();
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

    // Manually deplete to nearly zero
    for _ in 0..10 {
        world.terrain.deplete(x, y, 0.1);
    }

    world.terrain.update();
    assert_eq!(
        world.terrain.get_cell(ix, iy).terrain_type,
        TerrainType::Barren,
        "Should turn barren at low fertility"
    );
    assert!(
        world.terrain.movement_modifier(x, y) < 1.0,
        "Barren should slow movement"
    );

    // Test recovery from barren
    for _ in 0..600 {
        world.terrain.update();
    }
    assert_ne!(
        world.terrain.get_cell(ix, iy).terrain_type,
        TerrainType::Barren,
        "Should recover from barren state"
    );
}

#[test]
fn test_trophic_diet_restrictions() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.initial_food = 0;
    config.world.max_food = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let env = Environment::default();

    // 1. Carnivore should NOT eat plants
    let mut carnivore = primordium_lib::model::entity::Entity::new(10.5, 10.5, 0);
    carnivore.vx = 0.0;
    carnivore.vy = 0.0;
    carnivore.role = EntityRole::Carnivore;
    carnivore.energy = 50.0;
    world.entities.push(carnivore);
    world
        .food
        .push(primordium_lib::model::food::Food::new(10, 10));

    world.update(&env).expect("Update failed");
    assert_eq!(
        world.food.len(),
        1,
        "Carnivore should not have eaten the plant"
    );

    // 2. Herbivore SHOULD eat plants
    world.entities.clear();
    let mut herbivore = primordium_lib::model::entity::Entity::new(10.5, 10.5, 0);
    herbivore.vx = 0.0;
    herbivore.vy = 0.0;
    herbivore.role = EntityRole::Herbivore;
    herbivore.energy = 50.0;
    world.entities.push(herbivore);

    world.update(&env).expect("Update failed");
    assert_eq!(world.food.len(), 0, "Herbivore should have eaten the plant");
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
    for _ in 0..200 {
        world.update(&env).expect("Update failed");
        day_food_count += world.food.len();
        world.food.clear();
    }

    // 2. Night time growth
    env.world_time = env.day_cycle_ticks / 2 + 100; // Night
    let mut night_food_count = 0;
    for _ in 0..200 {
        world.update(&env).expect("Update failed");
        night_food_count += world.food.len();
        world.food.clear();
    }

    println!(
        "Day food: {}, Night food: {}",
        day_food_count, night_food_count
    );
    assert!(
        day_food_count >= night_food_count,
        "More food should grow during day (or equal if very low spawn). Day: {}, Night: {}",
        day_food_count,
        night_food_count
    );
}
