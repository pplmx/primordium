mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_data::TerrainType;

#[tokio::test]
async fn test_terrain_fertility_cycle() {
    let ix = 10;
    let iy = 10;

    let e = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(100.0)
        .trophic(0.0)
        .build();

    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(e)
        .with_fertility(0.5)
        .with_food(10.0, 10.0, 0.0)
        .with_config(|c| c.ecosystem.soil_depletion_unit = 0.5)
        .build();

    world.update(&mut env).expect("Update failed");
    let after_fertility = world.terrain.get_cell(ix, iy).fertility;
    assert!(after_fertility < 0.5);
}

#[tokio::test]
async fn test_barren_transition() {
    // This test involves manual depletion loops which WorldBuilder doesn't abstract (yet),
    // but we can use it for initial setup.
    let ix = 5;
    let iy = 5;

    let (mut world, _env) = WorldBuilder::new()
        .with_terrain(ix, iy, TerrainType::Plains)
        .build();

    for _ in 0..10 {
        world.terrain.deplete(5.5, 5.5, 0.1);
    }
    world.terrain.update(0.0, 0, 42);
    let terrain_type = world.terrain.get_cell(ix, iy).terrain_type;
    assert!(terrain_type == TerrainType::Barren || terrain_type == TerrainType::Desert);
}

#[tokio::test]
async fn test_trophic_diet_restrictions() {
    // Case 1: Herbivore (eats)
    {
        let herbivore = EntityBuilder::new()
            .at(10.0, 10.0)
            .energy(50.0)
            .trophic(0.0)
            .niche(0.5)
            .build();

        let (mut world, mut env) = WorldBuilder::new()
            .with_entity(herbivore)
            .with_food(10.0, 10.0, 0.5) // Matching niche
            .with_config(|c| c.ecosystem.soil_depletion_unit = 0.5)
            .build();

        world.update(&mut env).expect("Update failed");
        assert_eq!(world.get_food_count(), 0);
    }

    // Case 2: Carnivore (ignores plants)
    {
        let carnivore = EntityBuilder::new()
            .at(10.0, 10.0)
            .energy(50.0)
            .trophic(1.0)
            .build();

        let (mut world, mut env) = WorldBuilder::new()
            .with_entity(carnivore)
            .with_food(10.0, 10.0, 0.0) // Plant food
            .build();

        for _ in 0..10 {
            world.update(&mut env).expect("Update failed");
        }
        assert_eq!(world.get_food_count(), 1);
    }
}

#[tokio::test]
async fn test_light_dependent_food_growth() {
    let mut day_food_count = 0;
    {
        let (mut world, mut env) = WorldBuilder::new()
            .with_config(|c| c.world.max_food = 1000)
            .build();

        for _ in 0..1000 {
            env.world_time = env.day_cycle_ticks / 4; // Noon
            world.update(&mut env).expect("Update failed");
            day_food_count += world.get_food_count();

            // Clear food to measure spawn rate per tick
            // Direct ECS manipulation is still needed here as this is a specific measurement pattern
            let mut food_handles = Vec::new();
            for (h, _) in world
                .ecs
                .query::<&primordium_lib::model::food::Food>()
                .iter()
            {
                food_handles.push(h);
            }
            for h in food_handles {
                let _ = world.ecs.despawn(h);
                world
                    .food_count
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    let mut night_food_count = 0;
    {
        let (mut world, mut env) = WorldBuilder::new()
            .with_config(|c| c.world.max_food = 1000)
            .build();

        for _ in 0..1000 {
            env.world_time = env.day_cycle_ticks / 2 + 100; // Night
            world.update(&mut env).expect("Update failed");
            night_food_count += world.get_food_count();

            let mut food_handles = Vec::new();
            for (h, _) in world
                .ecs
                .query::<&primordium_lib::model::food::Food>()
                .iter()
            {
                food_handles.push(h);
            }
            for h in food_handles {
                let _ = world.ecs.despawn(h);
                world
                    .food_count
                    .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }
    assert!(day_food_count > night_food_count);
}
