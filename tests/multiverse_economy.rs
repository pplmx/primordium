use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use uuid::Uuid;

#[tokio::test]
async fn test_cross_universe_relief_extinction_safety() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");

    let l_id = Uuid::new_v4();
    // 1. Send relief to a lineage that DOES NOT exist in this world
    // This should fail silently or record for future members, but NOT panic
    world.apply_relief(l_id, 1000.0);

    assert_eq!(world.get_population_count(), 0);

    // 2. Spawn members of that lineage and apply again
    let mut e1 = lifecycle::create_entity(10.0, 10.0, 0);
    e1.metabolism.lineage_id = l_id;
    e1.metabolism.energy = 50.0;
    e1.metabolism.max_energy = 500.0;
    let h1 = world.spawn_entity(e1);

    world.apply_relief(l_id, 100.0);

    let met1 = world
        .ecs
        .get::<&primordium_lib::model::state::Metabolism>(h1)
        .unwrap();
    assert!(
        met1.energy > 50.0,
        "Energy relief should have reached the living members"
    );
}

#[tokio::test]
async fn test_biomass_trade_overflow_protection() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.max_food = 100;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // Force an incoming trade of 10,000 biomass (exceeding world limits)
    // The system should clamp this to a reasonable amount (e.g., 100)
    world.apply_trade(
        &mut env,
        primordium_lib::model::infra::network::TradeResource::Biomass,
        10000.0,
        true,
    );

    assert!(
        world.get_food_count() <= 100 + 100,
        "Biomass trade did not respect safety clamping limits"
    );

    // Test negative trade (outgoing) with 0 biomass
    world.apply_trade(
        &mut env,
        primordium_lib::model::infra::network::TradeResource::Biomass,
        5000.0,
        false,
    );
    assert_eq!(
        world.get_food_count(),
        0,
        "Outgoing trade should drain food but not underflow to massive count"
    );
}
