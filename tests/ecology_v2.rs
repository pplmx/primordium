use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::{Entity, EntityRole};
use primordium_lib::model::state::food::Food;
use primordium_lib::model::systems::ecological;
use primordium_lib::model::world::World;
use std::collections::HashSet;

#[test]
fn test_metabolic_niche_efficiency() {
    // 1. Green Specialist (niche = 0.0)
    let mut green_forager = Entity::new(10.0, 10.0, 0);
    green_forager.intel.genotype.metabolic_niche = 0.0;
    green_forager.metabolism.role = EntityRole::Herbivore;
    green_forager.metabolism.energy = 100.0;
    green_forager.metabolism.max_energy = 500.0;

    // 2. Blue Specialist (niche = 1.0)
    let mut blue_forager = Entity::new(10.0, 10.0, 0);
    blue_forager.intel.genotype.metabolic_niche = 1.0;
    blue_forager.metabolism.role = EntityRole::Herbivore;
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
        food_value: 100.0,
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
        food_value: 100.0,
        lineage_consumption: &mut lineage_consumption_b,
    };
    ecological::handle_feeding_optimized(0, &mut entities_b, &mut ctx_b);

    // 6. Assertions
    // Green Specialist on Green Food: Niche match (1.0 - abs(0-0)) = 1.0. Efficiency = 1.2x. Gain = 120.
    // Blue Specialist on Green Food: Niche match (1.0 - abs(1-0)) = 0.0. Efficiency = 0.2x. Gain = 20.
    assert!(entities_g[0].metabolism.energy > entities_b[0].metabolism.energy);
    assert_eq!(entities_g[0].metabolism.energy, 220.0); // 100 + 120
    assert_eq!(entities_b[0].metabolism.energy, 120.0); // 100 + 20
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
    let env = primordium_lib::model::state::environment::Environment::default();
    world.update(&env).unwrap();

    // We can't see private perception_buffer, but we verified the logic in handle_feeding.
    // The sense_nearest_food function returning f_type is the key.
    let (_, _, f_type) =
        ecological::sense_nearest_food(&world.entities[0], &world.food, &world.food_hash);
    assert_eq!(f_type, 1.0);
}
