use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::{ClimateState, Environment, Era, ResourceState};
use primordium_lib::model::history::PopulationStats;
use primordium_lib::model::systems::environment as environment_system;

#[tokio::test] async
fn test_hardware_coupling_logic() {
    let mut env = Environment {
        cpu_usage: 10.0,
        ..Environment::default()
    };

    // 1. Test CPU coupling
    assert_eq!(env.climate(), ClimateState::Temperate);

    env.cpu_usage = 95.0;
    assert_eq!(env.climate(), ClimateState::Scorching);

    // 2. Test RAM coupling
    env.ram_usage_percent = 10.0;
    assert_eq!(env.resource_state(), ResourceState::Abundant);

    env.ram_usage_percent = 90.0;
    assert_eq!(env.resource_state(), ResourceState::Famine);
}

#[tokio::test] async
fn test_era_transitions() {
    let mut env = Environment::default();
    let mut stats = PopulationStats::default();
    let config = AppConfig::default();

    // Start at Primordial
    assert_eq!(env.current_era, Era::Primordial);

    // Transition to DawnOfLife
    stats.avg_lifespan = 250.0;
    environment_system::update_era(&mut env, 6000, &stats, &config);
    assert_eq!(env.current_era, Era::DawnOfLife);

    // Transition to Flourishing
    stats.population = 300;
    stats.species_count = 5;
    stats.biodiversity_hotspots = 2; // Required for new Flourishing trigger
    environment_system::update_era(&mut env, 7000, &stats, &config);
    assert_eq!(env.current_era, Era::Flourishing);

    // Transition to DominanceWar (High Carbon Level)
    env.carbon_level = 900.0;
    environment_system::update_era(&mut env, 8000, &stats, &config);
    assert_eq!(env.current_era, Era::DominanceWar);

    // Transition to ApexEra
    stats.top_fitness = 9000.0; // Comfortably above 8000
    environment_system::update_era(&mut env, 9000, &stats, &config);
    assert_eq!(env.current_era, Era::ApexEra);
}
