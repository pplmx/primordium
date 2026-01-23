use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;

#[test]
fn test_r_strategy_fast_reproduction() {
    let mut config = AppConfig::default();
    config.metabolism.maturity_age = 100;

    // 1. R-Strategist (Fast maturity, low investment)
    let mut r_parent = Entity::new(10.0, 10.0, 0);
    r_parent.intel.genotype.maturity_gene = 0.5; // Matures at tick 50
    r_parent.intel.genotype.reproductive_investment = 0.2; // Gives 20% energy
    r_parent.metabolism.energy = 200.0;

    // Check maturity at tick 60
    assert!(r_parent.is_mature(60, config.metabolism.maturity_age));

    // Reproduce
    let child = primordium_lib::model::systems::social::reproduce_asexual(
        &mut r_parent,
        60,
        &config.evolution,
    );

    // Child should have 20% of 200 = 40 energy
    assert!(child.metabolism.energy < 50.0);
    assert!(r_parent.metabolism.energy > 150.0);
}

#[test]
fn test_k_strategy_slow_reproduction() {
    let mut config = AppConfig::default();
    config.metabolism.maturity_age = 100;

    // 1. K-Strategist (Slow maturity, high investment)
    let mut k_parent = Entity::new(10.0, 10.0, 0);
    k_parent.intel.genotype.maturity_gene = 2.0; // Matures at tick 200
    k_parent.intel.genotype.reproductive_investment = 0.8; // Gives 80% energy
    k_parent.metabolism.energy = 400.0;

    // Check maturity at tick 150 - should NOT be mature
    assert!(!k_parent.is_mature(150, config.metabolism.maturity_age));

    // Check at 250 - should be mature
    assert!(k_parent.is_mature(250, config.metabolism.maturity_age));

    // Reproduce
    let child = primordium_lib::model::systems::social::reproduce_asexual(
        &mut k_parent,
        250,
        &config.evolution,
    );

    // Child should have 80% of 400 = 320 energy
    assert!(child.metabolism.energy > 300.0);
    assert!(k_parent.metabolism.energy < 100.0);
}

#[test]
fn test_maturity_body_size_coupling() {
    let config = AppConfig::default();
    let mut genotype = primordium_lib::model::state::entity::Genotype::new_random();

    // Strategy R
    genotype.maturity_gene = 0.5;
    primordium_lib::model::systems::intel::mutate_genotype(&mut genotype, &config.evolution);
    let r_max = genotype.max_energy;

    // Strategy K
    genotype.maturity_gene = 2.0;
    primordium_lib::model::systems::intel::mutate_genotype(&mut genotype, &config.evolution);
    let k_max = genotype.max_energy;

    assert!(
        k_max > r_max,
        "K-strategy should result in larger max energy capacity"
    );
}

#[test]
fn test_k_strategy_slow_reproduction() {
    let mut config = AppConfig::default();
    config.metabolism.maturity_age = 100;

    // 1. K-Strategist (Slow maturity, high investment)
    let mut k_parent = Entity::new(10.0, 10.0, 0);
    k_parent.intel.genotype.maturity_gene = 2.0; // Matures at tick 200
    k_parent.intel.genotype.reproductive_investment = 0.8; // Gives 80% energy
    k_parent.metabolism.energy = 400.0;

    // Check maturity at tick 150 - should NOT be mature
    assert!(!k_parent.is_mature(150, config.metabolism.maturity_age));

    // Check at 250 - should be mature
    assert!(k_parent.is_mature(250, config.metabolism.maturity_age));

    // Reproduce
    let child = primordium_lib::model::systems::social::reproduce_asexual(
        &mut k_parent,
        250,
        &config.evolution,
    );

    // Child should have 80% of 400 = 320 energy
    assert!(child.metabolism.energy > 300.0);
    assert!(k_parent.metabolism.energy < 100.0);
}

#[test]
fn test_maturity_body_size_coupling() {
    let mut config = AppConfig::default();
    let mut genotype = primordium_lib::model::state::entity::Genotype::new_random();

    // Strategy R
    genotype.maturity_gene = 0.5;
    primordium_lib::model::systems::intel::mutate_genotype(&mut genotype, &config.evolution);
    let r_max = genotype.max_energy;

    // Strategy K
    genotype.maturity_gene = 2.0;
    primordium_lib::model::systems::intel::mutate_genotype(&mut genotype, &config.evolution);
    let k_max = genotype.max_energy;

    assert!(
        k_max > r_max,
        "K-strategy should result in larger max energy capacity"
    );
}
