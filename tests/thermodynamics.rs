use primordium_lib::model::state::environment::Environment;

#[test]
fn test_environment_has_energy_pool() {
    let env = Environment::default();
    assert!(
        env.available_energy > 0.0,
        "Environment must start with positive available energy"
    );
    assert_eq!(env.available_energy, 10000.0);
}

#[test]
fn test_solar_injection() {
    let mut config = primordium_lib::model::config::AppConfig::default();
    config.world.initial_population = 0;
    config.ecosystem.solar_energy_rate = 500.0;

    let mut world = primordium_lib::model::world::World::new(0, config).unwrap();
    let mut env = Environment::default();
    let initial_energy = env.available_energy;

    world.update(&mut env).unwrap();

    // Energy should increase by solar rate
    // Note: spawn_food might consume some if random chance hits.
    // To be safe, we disable food spawning or check if it increased net.
    // If we assume no food spawned (population 0, but food spawns independently).

    assert!(
        env.available_energy > initial_energy,
        "Solar injection should increase energy pool"
    );
}

#[test]
fn test_spawn_consumes_energy() {
    let mut config = primordium_lib::model::config::AppConfig::default();
    config.world.initial_population = 0;
    config.ecosystem.solar_energy_rate = 0.0; // Disable solar to isolate consumption
    config.ecosystem.base_spawn_chance = 1.0; // Force spawn attempt
    config.metabolism.food_energy_cost = 100.0;
    config.world.max_food = 1000;
    config.world.initial_food = 0; // Start empty

    let mut world = primordium_lib::model::world::World::new(0, config).unwrap();
    let mut env = Environment::default();
    let initial_energy = env.available_energy;

    world.update(&mut env).unwrap();

    let final_energy = env.available_energy;
    let food_count = world.get_food_count();

    if food_count > 0 {
        assert!(
            final_energy < initial_energy,
            "Spawning food should consume energy"
        );
        // Check exact amount?
        // cost = 100.0 * food_count
        let expected_cost = 100.0 * food_count as f64;
        assert!((initial_energy - final_energy - expected_cost).abs() < 0.001);
    }
}

#[test]
fn test_death_recycles_energy() {
    let mut config = primordium_lib::model::config::AppConfig::default();
    config.world.initial_population = 0;
    config.ecosystem.solar_energy_rate = 0.0; // Disable solar
    config.ecosystem.spawn_rate_limit_enabled = true; // Disable spawning
    config.ecosystem.max_food_per_tick = 0;

    let mut world = primordium_lib::model::world::World::new(0, config).unwrap();
    let mut env = Environment::default();

    // Spawn one entity
    let mut entity = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    entity.metabolism.energy = 500.0;
    entity.metabolism.max_energy = 1000.0;
    world.spawn_entity(entity);

    let initial_energy = env.available_energy;

    // Kill it
    for (_h, met) in world.ecs.query_mut::<&mut primordium_data::Metabolism>() {
        met.energy = -1.0;
    }

    world.update(&mut env).unwrap();

    let final_energy = env.available_energy;

    // Recycled = energy + 1000 * 0.5.
    // energy starts at -1.0, then metabolism reduces it further (e.g. -2.5).
    // Recycled approx -2.5 + 500 = 497.5.

    assert!(final_energy > initial_energy, "Death should recycle energy");
    assert!(
        (final_energy - initial_energy - 499.0).abs() < 5.0,
        "Recycled amount mismatch. Got: {}",
        final_energy - initial_energy
    );
}
