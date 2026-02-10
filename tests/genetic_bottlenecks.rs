use primordium_core::systems::{intel, social};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_genetic_bottleneck_increases_mutation() {
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
    let mut rng = rand::thread_rng();
    let mut ctx_small = social::ReproductionContext {
        tick: 1,
        config: &world.config,
        population: 1,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child_small, _) = social::reproduce_asexual_parallel_components_decomposed(
        social::AsexualReproductionContext {
            pos: &parent.position,
            energy: parent.metabolism.energy,
            generation: parent.metabolism.generation,
            genotype: &parent.intel.genotype,
            specialization: parent.intel.specialization,
            ctx: &mut ctx_small,
        },
    );

    // 3. Large population (100) -> Should have base mutation
    let mut rng = rand::thread_rng();
    let mut ctx_large = social::ReproductionContext {
        tick: 2,
        config: &world.config,
        population: 100,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child_large, _) = social::reproduce_asexual_parallel_components_decomposed(
        social::AsexualReproductionContext {
            pos: &parent.position,
            energy: parent.metabolism.energy,
            generation: parent.metabolism.generation,
            genotype: &parent.intel.genotype,
            specialization: parent.intel.specialization,
            ctx: &mut ctx_large,
        },
    );

    assert_ne!(child_small.identity.id, child_large.identity.id);
}

#[tokio::test]
async fn test_genetic_drift_in_small_pop() {
    let mut config = AppConfig::default();
    config.evolution.mutation_rate = 0.0; // Disable normal mutation
    config.evolution.mutation_amount = 0.0;

    // Force a small population context
    let population = 5;

    let parent_genotype = std::sync::Arc::new(
        primordium_lib::model::brain::create_genotype_random_with_rng(&mut rand::thread_rng()),
    );
    let original_tp = parent_genotype.trophic_potential;

    // Run many mutations to trigger the 5% drift chance
    let mut drift_occurred = false;
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let mut test_genotype = parent_genotype.clone();
        intel::mutate_genotype(
            &mut test_genotype,
            &intel::MutationParams {
                config: &config,
                population,
                is_radiation_storm: false,
                specialization: None,
                ancestral_genotype: None,
                stress_factor: 0.0,
            },
            &mut rng,
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
