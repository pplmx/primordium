use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::{Environment, Era};
use primordium_lib::model::history::PopulationStats;
use primordium_lib::model::systems::environment as environment_system;

#[tokio::test]
async fn test_era_transition_sequence() {
    let mut env = Environment::default();
    let mut stats = PopulationStats::default();

    // 1. Initial State
    let config = AppConfig::default();
    assert_eq!(env.current_era, Era::Primordial);
    let primordial_mult = env.metabolism_multiplier();

    // 2. Trigger DawnOfLife (Requires long lifespan)
    // Era transition logic: current_era == Primordial && tick > 5000 && lifespan > 200.0
    stats.avg_lifespan = 500.0; // Comfortably above 200.0
    environment_system::update_era(&mut env, 10000, &stats, &config); // Comfortably above 5000

    assert_eq!(
        env.current_era,
        Era::DawnOfLife,
        "Failed to transition to DawnOfLife"
    );

    // 3. Trigger Flourishing (Requires population and diversity)
    stats.population = 300;
    stats.species_count = 5;
    stats.biodiversity_hotspots = 2; // Required for new Flourishing trigger
    environment_system::update_era(&mut env, 15000, &stats, &config);

    assert_eq!(
        env.current_era,
        Era::Flourishing,
        "Failed to transition to Flourishing"
    );

    // Flourishing should have different growth/metabolism conditions
    let flourishing_mult = env.metabolism_multiplier();
    assert_ne!(
        primordial_mult, flourishing_mult,
        "Metabolism should change with eras"
    );
}

#[tokio::test]
async fn test_circadian_environmental_stress() {
    // Day time (Default is 0)
    let mut env = Environment {
        world_time: 0,
        ..Default::default()
    };
    let day_mult = env.metabolism_multiplier();

    // Night time
    env.world_time = env.day_cycle_ticks / 2 + 10;
    let night_mult = env.metabolism_multiplier();

    assert!(
        night_mult < day_mult,
        "Metabolism should be lower at night (resting)"
    );
}

#[tokio::test]
async fn test_hardware_resource_linkage() {
    // Simulate low stress
    let mut env = Environment {
        cpu_usage: 10.0,
        ram_usage_percent: 20.0,
        ..Default::default()
    };
    let low_mult = env.metabolism_multiplier();

    // Simulate extreme stress
    env.cpu_usage = 95.0;
    env.ram_usage_percent = 90.0;
    let high_mult = env.metabolism_multiplier();

    // Extreme heat from high CPU should accelerate metabolism (and energy drain)
    assert!(
        high_mult > low_mult,
        "High CPU stress should increase metabolic cost"
    );
}
