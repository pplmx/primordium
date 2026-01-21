use primordium_lib::model::environment::{ClimateState, Environment, Era, ResourceState};
use primordium_lib::model::history::PopulationStats;

#[test]
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

#[test]
fn test_era_transitions() {
    let mut env = Environment::default();
    let mut stats = PopulationStats::new();

    // Start at Primordial
    assert_eq!(env.current_era, Era::Primordial);

    // Transition to DawnOfLife
    stats.avg_lifespan = 250.0;
    env.update_era(6000, &stats);
    assert_eq!(env.current_era, Era::DawnOfLife);

    // Transition to Flourishing
    stats.population = 300;
    stats.species_count = 5;
    env.update_era(7000, &stats);
    assert_eq!(env.current_era, Era::Flourishing);

    // Transition to DominanceWar (High CPU stress)
    env.cpu_usage = 85.0;
    env.update_era(8000, &stats);
    assert_eq!(env.current_era, Era::DominanceWar);

    // Transition to ApexEra
    stats.top_fitness = 6000.0;
    env.update_era(9000, &stats);
    assert_eq!(env.current_era, Era::ApexEra);
}
