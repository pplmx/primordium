use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::world::World;

#[test]
fn test_mate_preference_rejection() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // 1. Selector: Prefers Carnivores (1.0)
    let mut selector = Entity::new(10.0, 10.0, 0); // born at 0
    selector.intel.genotype.mate_preference = 1.0;
    selector.metabolism.energy = 1000.0;

    // 2. Candidate: Pure Herbivore (0.0)
    let mut herbivore = Entity::new(10.5, 10.5, 0); // born at 0
    herbivore.metabolism.trophic_potential = 0.0;
    herbivore.metabolism.energy = 500.0;

    world.entities.push(selector);
    world.entities.push(herbivore);

    // 3. Update world at tick 200 (Mature)
    world.tick = 200;

    // Check if new babies were born
    // Initial was 2. If mating happened, would be 3.
    assert_eq!(
        world.entities.len(),
        2,
        "Selector should have rejected the mismatching partner"
    );
}

#[test]
fn test_mate_preference_acceptance() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    // 1. Selector: Prefers Carnivores (1.0)
    let mut selector = Entity::new(10.0, 10.0, 0);
    selector.intel.genotype.mate_preference = 1.0;
    selector.metabolism.energy = 1000.0;

    // 2. Candidate: Pure Carnivore (1.0)
    let mut carnivore = Entity::new(10.5, 10.5, 0);
    carnivore.metabolism.trophic_potential = 1.0;
    carnivore.metabolism.energy = 500.0;

    world.entities.push(selector);
    world.entities.push(carnivore);

    // 3. Update world at tick 200
    world.tick = 200;
    let mut env = primordium_lib::model::state::environment::Environment::default();
    world.update(&mut env).unwrap();

    assert!(
        world.entities.len() > 2,
        "Selector should have accepted the matching partner"
    );
}
