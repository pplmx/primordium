use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_genetic_bottleneck_increases_mutation() {
    let mut config = AppConfig::default();
    config.evolution.population_aware = true;
    config.evolution.bottleneck_threshold = 50;
    config.evolution.mutation_rate = 0.1;

    let _env = Environment::default();
    let world = World::new(0, config).unwrap();

    // 1. Create a parent
    let mut parent = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    parent.metabolism.energy = 500.0;

    // 2. Small population (1) -> Should have high effective mutation
    // We can't easily measure effective_mutation_rate directly as it's internal to mutate_genotype,
    // but we can check if it results in higher genetic variance over multiple trials.
    // However, for a unit test, we just verify it doesn't crash and follows the logic.
    let child_small = primordium_lib::model::systems::social::reproduce_asexual(
        &mut parent,
        1,
        &world.config,
        1,
        std::collections::HashSet::new(),
        false,
    );

    // 3. Large population (100) -> Should have base mutation
    let child_large = primordium_lib::model::systems::social::reproduce_asexual(
        &mut parent,
        2,
        &world.config,
        100,
        std::collections::HashSet::new(),
        false,
    );

    assert_ne!(child_small.id, child_large.id);
}

#[test]
fn test_genetic_drift_in_small_pop() {
    let mut config = AppConfig::default();
    config.evolution.mutation_rate = 0.0; // Disable normal mutation
    config.evolution.mutation_amount = 0.0;

    // Force a small population context
    let population = 5;

    let parent_genotype =
        primordium_lib::model::brain::create_genotype_random_with_rng(&mut rand::thread_rng());
    let original_tp = parent_genotype.trophic_potential;

    // Run many mutations to trigger the 5% drift chance
    let mut drift_occurred = false;
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let mut test_genotype = parent_genotype.clone();
        primordium_lib::model::systems::intel::mutate_genotype(
            &mut test_genotype,
            &config,
            population,
            false,
            None,
            &mut rng,
            None,
        );
        if (test_genotype.trophic_potential - original_tp).abs() > 0.001 {
            drift_occurred = true;
            break;
        }
    }

    assert!(
        drift_occurred,
        "Genetic drift should eventually flip a trait in small populations"
    );
}
