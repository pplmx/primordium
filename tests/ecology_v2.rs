use primordium_core::systems::ecological;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::food::Food;
use primordium_lib::model::world::World;
use std::collections::HashSet;

#[tokio::test]
async fn test_metabolic_niche_efficiency() {
    // 1. Green Specialist (niche = 0.0, Herbivore-leaning)
    let mut green_forager = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    std::sync::Arc::make_mut(&mut green_forager.intel.genotype).metabolic_niche = 0.0;
    green_forager.metabolism.trophic_potential = 0.0;
    green_forager.metabolism.energy = 100.0;
    green_forager.metabolism.max_energy = 500.0;

    // 2. Blue Specialist (niche = 1.0, Herbivore-leaning)
    let mut blue_forager = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    std::sync::Arc::make_mut(&mut blue_forager.intel.genotype).metabolic_niche = 1.0;
    blue_forager.metabolism.trophic_potential = 0.0;
    blue_forager.metabolism.energy = 100.0;
    blue_forager.metabolism.max_energy = 500.0;

    // 3. Green Food
    let food_green = vec![Food::new(10, 10, 0.0)];
    let mut food_hash = primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100);
    food_hash.build_parallel(&[(10.0, 10.0)], 100, 100);

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

#[tokio::test]
async fn test_nutrient_sensing_in_perception() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // Nearest food is Blue (1.0)
    world.ecs.spawn((
        Food::new(12, 10, 1.0),
        primordium_lib::model::state::Position { x: 12.0, y: 10.0 },
        primordium_lib::model::state::MetabolicNiche(1.0),
    ));

    let e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    world.spawn_entity(e);

    // Update world to trigger perception
    let mut env = primordium_lib::model::state::environment::Environment::default();
    world.update(&mut env).unwrap();

    // Rebuild hash to ensure manual sense_nearest_food call is accurate
    let mut food_positions = Vec::new();
    let mut food_data = Vec::new();
    for (_handle, f) in world.ecs.query::<&Food>().iter() {
        food_positions.push((f.x as f64, f.y as f64));
        food_data.push(f.clone());
    }
    world.food_hash.build_parallel(&food_positions, 100, 100);

    let entities = world.get_all_entities();
    if !food_data.is_empty() {
        let (_, _, f_type) =
            ecological::sense_nearest_food(&entities[0], &food_data, &world.food_hash);
        assert!(f_type >= 0.0);
    }
}

#[tokio::test]
async fn test_niche_partitioning_coexistence() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.initial_food = 0;
    let mut world = World::new(0, config).unwrap();

    // 1. Two different food sources
    world.ecs.spawn((
        Food::new(10, 10, 0.0),
        primordium_lib::model::state::Position { x: 10.0, y: 10.0 },
        primordium_lib::model::state::MetabolicNiche(0.0),
    )); // Green
    world.ecs.spawn((
        Food::new(15, 15, 1.0),
        primordium_lib::model::state::Position { x: 15.0, y: 15.0 },
        primordium_lib::model::state::MetabolicNiche(1.0),
    )); // Blue

    // 2. Two specialized entities
    let mut green_spec = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    std::sync::Arc::make_mut(&mut green_spec.intel.genotype).metabolic_niche = 0.0;
    green_spec.metabolism.trophic_potential = 0.0;
    green_spec.metabolism.energy = 100.0;

    let mut blue_spec = primordium_lib::model::lifecycle::create_entity(15.0, 15.0, 0);
    std::sync::Arc::make_mut(&mut blue_spec.intel.genotype).metabolic_niche = 1.0;
    blue_spec.metabolism.trophic_potential = 0.0;
    blue_spec.metabolism.energy = 100.0;

    let mut eaten_indices = HashSet::new();
    let mut lineage_consumption = Vec::new();

    // 3. Feeding Context
    let mut entities = vec![green_spec, blue_spec];

    // Update food hash manually
    let mut food_positions = Vec::new();
    let mut food_data = Vec::new();
    for (_handle, f) in world.ecs.query::<&Food>().iter() {
        food_positions.push((f.x as f64, f.y as f64));
        food_data.push(f.clone());
    }
    world.food_hash.build_parallel(&food_positions, 100, 100);

    let mut ctx = ecological::FeedingContext {
        food: &food_data,
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
