use primordium_core::systems::{intel, social};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;

#[tokio::test]
async fn test_r_strategy_fast_reproduction() {
    let mut config = AppConfig::default();
    config.metabolism.maturity_age = 100;

    // 1. R-Strategist (Fast maturity, low investment)
    let mut r_parent = lifecycle::create_entity(10.0, 10.0, 0);
    std::sync::Arc::make_mut(&mut r_parent.intel.genotype).maturity_gene = 0.5; // Matures at tick 50
    std::sync::Arc::make_mut(&mut r_parent.intel.genotype).reproductive_investment = 0.2; // Gives 20% energy
    r_parent.metabolism.energy = 200.0;

    // Check maturity at tick 60
    assert!(lifecycle::is_mature_components(
        &r_parent.metabolism,
        &r_parent.intel,
        60,
        config.metabolism.maturity_age
    ));

    // Reproduce
    let mut rng = rand::thread_rng();
    let mut ctx = social::ReproductionContext {
        tick: 60,
        config: &config,
        population: 1,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child, _) = social::reproduce_asexual_parallel_components_decomposed(
        social::AsexualReproductionContext {
            pos: &r_parent.position,
            energy: r_parent.metabolism.energy,
            generation: r_parent.metabolism.generation,
            genotype: &r_parent.intel.genotype,
            specialization: r_parent.intel.specialization,
            ctx: &mut ctx,
        },
    );
    r_parent.metabolism.energy *= 1.0 - r_parent.intel.genotype.reproductive_investment as f64;

    // Child should have 20% of 200 = 40 energy
    assert!(child.metabolism.energy < 50.0);
    assert!(r_parent.metabolism.energy > 150.0);
}

#[tokio::test]
async fn test_k_strategy_slow_reproduction() {
    let mut config = AppConfig::default();
    config.metabolism.maturity_age = 100;

    let mut k_parent = lifecycle::create_entity(10.0, 10.0, 0);
    std::sync::Arc::make_mut(&mut k_parent.intel.genotype).maturity_gene = 2.0;
    std::sync::Arc::make_mut(&mut k_parent.intel.genotype).reproductive_investment = 0.8;
    k_parent.metabolism.energy = 400.0;

    // Check maturity at tick 150 - should NOT be mature
    assert!(!lifecycle::is_mature_components(
        &k_parent.metabolism,
        &k_parent.intel,
        150,
        config.metabolism.maturity_age
    ));

    // Check at 250 - should be mature
    assert!(lifecycle::is_mature_components(
        &k_parent.metabolism,
        &k_parent.intel,
        250,
        config.metabolism.maturity_age
    ));

    // Reproduce
    let mut rng = rand::thread_rng();
    let mut ctx = social::ReproductionContext {
        tick: 250,
        config: &config,
        population: 1,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child, _) = social::reproduce_asexual_parallel_components_decomposed(
        social::AsexualReproductionContext {
            pos: &k_parent.position,
            energy: k_parent.metabolism.energy,
            generation: k_parent.metabolism.generation,
            genotype: &k_parent.intel.genotype,
            specialization: k_parent.intel.specialization,
            ctx: &mut ctx,
        },
    );

    assert!(child.metabolism.energy > 250.0);
    assert!(child.metabolism.energy < 300.0);
}

#[tokio::test]
async fn test_maturity_body_size_coupling() {
    let config = AppConfig::default();
    let mut genotype = std::sync::Arc::new(
        primordium_lib::model::brain::create_genotype_random_with_rng(&mut rand::thread_rng()),
    );

    // Strategy R
    std::sync::Arc::make_mut(&mut genotype).maturity_gene = 0.5;
    let mut rng = rand::thread_rng();
    intel::mutate_genotype(
        &mut genotype,
        &intel::MutationParams {
            config: &config,
            population: 100,
            is_radiation_storm: false,
            specialization: None,
            ancestral_genotype: None,
            stress_factor: 0.0,
        },
        &mut rng,
    );
    let r_max = genotype.max_energy;

    // Strategy K
    std::sync::Arc::make_mut(&mut genotype).maturity_gene = 2.0;
    intel::mutate_genotype(
        &mut genotype,
        &intel::MutationParams {
            config: &config,
            population: 100,
            is_radiation_storm: false,
            specialization: None,
            ancestral_genotype: None,
            stress_factor: 0.0,
        },
        &mut rng,
    );
    let k_max = genotype.max_energy;

    assert!(
        k_max > r_max,
        "K-strategy should result in larger max energy capacity"
    );
}
