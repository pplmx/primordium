use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_simulation_lifecycle() {
    // 1. Setup
    let config = AppConfig::default();
    let initial_pop = 50;
    let mut world = World::new(initial_pop, config).expect("Failed to create world");
    let env = Environment::default();

    assert_eq!(world.entities.len(), initial_pop);

    // 2. Run for 100 ticks
    for _ in 0..100 {
        world.update(&env).expect("World update failed");
    }

    // 3. Verify
    // Population should change based on birth/death
    println!("Population after 100 ticks: {}", world.entities.len());

    // Hall of fame should be populated if there were any high performers
    // Or at least initialized
    assert!(world.hall_of_fame.top_living.len() <= 3);

    // Check if time progressed
    assert_eq!(world.tick, 100);
}

#[test]
fn test_reproduction_and_genetics() {
    let mut config = AppConfig::default();
    // High energy start to encourage reproduction
    config.world.initial_population = 10;
    config.metabolism.maturity_age = 10; // Rapid maturity for test

    let mut world = World::new(10, config).expect("Failed to create world");
    let env = Environment::default();

    // Force high energy on all entities to trigger reproduction
    for entity in &mut world.entities {
        entity.metabolism.energy = 200.0;
    }

    // Run ticks - some should reproduce
    let mut total_births = 0;
    for _ in 0..50 {
        let events = world.update(&env).expect("Update failed");
        for event in events {
            if matches!(
                event,
                primordium_lib::model::history::LiveEvent::Birth { .. }
            ) {
                total_births += 1;
            }
        }
    }

    assert!(
        total_births > 0,
        "No births occurred even with high energy and rapid maturity"
    );
    println!("Total births in 50 ticks: {}", total_births);
}
