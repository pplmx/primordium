use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_r_vs_k_dominance_in_resource_boom() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.max_food = 500; // Abundant food
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // Strategy R: Fast maturity (50), Low energy (100)
    let mut r_type = Entity::new(10.0, 10.0, 0);
    r_type.intel.genotype.maturity_gene = 0.5;
    r_type.intel.genotype.reproductive_investment = 0.2;
    r_type.metabolism.energy = 100.0;

    // Strategy K: Slow maturity (200), High energy (400)
    let mut k_type = Entity::new(20.0, 20.0, 0);
    k_type.intel.genotype.maturity_gene = 2.0;
    k_type.intel.genotype.reproductive_investment = 0.8;
    k_type.metabolism.energy = 400.0;

    world.entities.push(r_type);
    world.entities.push(k_type);

    // In a resource boom, Strategy R should multiply faster
    for _ in 0..100 {
        world.update(&mut env).unwrap();
        // Keep energy high to simulate boom
        for e in &mut world.entities {
            e.metabolism.energy = 500.0;
        }
        if world.entities.len() > 100 {
            break;
        }
    }

    let r_count = world
        .entities
        .iter()
        .filter(|e| e.intel.genotype.maturity_gene < 1.0)
        .count();
    let k_count = world
        .entities
        .iter()
        .filter(|e| e.intel.genotype.maturity_gene > 1.0)
        .count();

    assert!(
        r_count > k_count,
        "R-strategists should out-multiply K-strategists in resource booms. R: {}, K: {}",
        r_count,
        k_count
    );
}
