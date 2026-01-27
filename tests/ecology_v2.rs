use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::food::Food;
use primordium_lib::model::systems::ecological;
use primordium_lib::model::world::World;
use std::collections::HashSet;

#[test]
fn test_metabolic_niche_efficiency() {
    // 1. Green Specialist (niche = 0.0, Herbivore-leaning)
    let mut green_forager = Entity::new(10.0, 10.0, 0);
    green_forager.intel.genotype.metabolic_niche = 0.0;
    green_forager.metabolism.trophic_potential = 0.0;
    green_forager.metabolism.energy = 100.0;
    green_forager.metabolism.max_energy = 500.0;

    // 2. Blue Specialist (niche = 1.0, Herbivore-leaning)
    let mut blue_forager = Entity::new(10.0, 10.0, 0);
    blue_forager.intel.genotype.metabolic_niche = 1.0;
    blue_forager.metabolism.trophic_potential = 0.0;
    blue_forager.metabolism.energy = 100.0;
    blue_forager.metabolism.max_energy = 500.0;

    // 3. Green Food
    let food_green = vec![Food::new(10, 10, 0.0)];
    let mut food_hash = primordium_lib::model::quadtree::SpatialHash::new(5.0);
    food_hash.insert(10.0, 10.0, 0);

    let mut eaten_indices = HashSet::new();
    let mut terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(20, 20, 42);
    let mut pheromones = primordium_lib::model::state::pheromone::PheromoneGrid::new(20, 20);
    let mut lineage_consumption = Vec::new();

    // 4. Feeding simulation for Green Specialist
    let mut entities_g = vec![green_forager.clone()];
    let mut ctx_g = ecological::FeedingContext {
        food: &food_green,
        food_hash: &food_hash,
        eaten_indices: &mut eaten_indices,
        terrain: &mut terrain,
        pheromones: &mut pheromones,
        config: &AppConfig::default(),
        lineage_consumption: &mut lineage_consumption,
    };
    ecological::handle_feeding_optimized(0, &mut entities_g, &mut ctx_g);

    // 5. Feeding simulation for Blue Specialist (reset context)
    let mut eaten_indices_b = HashSet::new();
    let mut lineage_consumption_b = Vec::new();
    let mut entities_b = vec![blue_forager.clone()];
    let mut ctx_b = ecological::FeedingContext {
        food: &food_green,
        food_hash: &food_hash,
        eaten_indices: &mut eaten_indices_b,
        terrain: &mut terrain,
        pheromones: &mut pheromones,
        config: &AppConfig::default(),
        lineage_consumption: &mut lineage_consumption_b,
    };
    ecological::handle_feeding_optimized(0, &mut entities_b, &mut ctx_b);

    // 6. Assertions
    assert!(entities_g[0].metabolism.energy > entities_b[0].metabolism.energy);
    assert_eq!(entities_g[0].metabolism.energy, 160.0);
    assert_eq!(entities_b[0].metabolism.energy, 110.0);
}

#[test]
fn test_nutrient_sensing_in_perception() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // Nearest food is Blue (1.0)
    world.food.push(Food::new(12, 10, 1.0));

    let e = Entity::new(10.0, 10.0, 0);
    world.entities.push(e);

    // Update world to trigger perception
    let mut env = primordium_lib::model::state::environment::Environment::default();
    world.update(&mut env).unwrap();

    // Rebuild hash to ensure manual sense_nearest_food call is accurate
    world.food_hash.clear();
    for (i, f) in world.food.iter().enumerate() {
        world.food_hash.insert(f.x as f64, f.y as f64, i);
    }

    if !world.food.is_empty() {
        let (_, _, f_type) =
            ecological::sense_nearest_food(&world.entities[0], &world.food, &world.food_hash);
        assert!(f_type >= 0.0);
    }
}

#[test]
fn test_niche_partitioning_coexistence() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.initial_food = 0;
    let mut world = World::new(0, config).unwrap();

    // 1. Two different food sources
    world.food.push(Food::new(10, 10, 0.0)); // Green
    world.food.push(Food::new(15, 15, 1.0)); // Blue

    // 2. Two specialized entities
    let mut green_spec = Entity::new(10.0, 10.0, 0);
    green_spec.intel.genotype.metabolic_niche = 0.0;
    green_spec.metabolism.trophic_potential = 0.0;
    green_spec.metabolism.energy = 100.0;

    let mut blue_spec = Entity::new(15.0, 15.0, 0);
    blue_spec.intel.genotype.metabolic_niche = 1.0;
    blue_spec.metabolism.trophic_potential = 0.0;
    blue_spec.metabolism.energy = 100.0;

    let mut eaten_indices = HashSet::new();
    let mut lineage_consumption = Vec::new();

    // 3. Feeding Context
    let mut entities = vec![green_spec, blue_spec];

    // Update food hash manually
    world.food_hash.clear();
    for (i, f) in world.food.iter().enumerate() {
        world.food_hash.insert(f.x as f64, f.y as f64, i);
    }

    let mut ctx = ecological::FeedingContext {
        food: &world.food,
        food_hash: &world.food_hash,
        eaten_indices: &mut eaten_indices,
        terrain: &mut world.terrain,
        pheromones: &mut world.pheromones,
        config: &world.config,
        lineage_consumption: &mut lineage_consumption,
    };

    // 4. Run feeding for both
    ecological::handle_feeding_optimized(0, &mut entities, &mut ctx);
    ecological::handle_feeding_optimized(1, &mut entities, &mut ctx);

    assert_eq!(
        eaten_indices.len(),
        2,
        "Both specialized foragers should have consumed their food"
    );
}
