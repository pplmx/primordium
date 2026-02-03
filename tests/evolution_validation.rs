use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_r_vs_k_dominance_in_resource_boom() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.max_food = 500; // Abundant food
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // Strategy R: Fast maturity (50), Low investment (0.2)
    // Low investment means babies start weak, but they mature fast.
    let mut r_type = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    r_type.intel.genotype.maturity_gene = 0.5;
    r_type.intel.genotype.reproductive_investment = 0.2;
    r_type.metabolism.energy = 500.0;
    r_type.metabolism.max_energy = 500.0;
    r_type.metabolism.lineage_id = uuid::Uuid::new_v4();
    r_type.intel.genotype.lineage_id = r_type.metabolism.lineage_id;

    // Strategy K: Slow maturity (200), High investment (0.8)
    // High investment means babies start with 80% parent energy (400.0).
    // If reproduction threshold is < 400.0, they might reproduce instantly IF they are mature.
    // But they have high maturity gene (2.0), so they should take longer to mature.
    let mut k_type = primordium_lib::model::lifecycle::create_entity(20.0, 20.0, 0);
    k_type.intel.genotype.maturity_gene = 5.0; // Increase maturity time significantly to handicap K
    k_type.intel.genotype.reproductive_investment = 0.8;
    k_type.metabolism.energy = 500.0;
    k_type.metabolism.max_energy = 1000.0;
    k_type.metabolism.lineage_id = uuid::Uuid::new_v4();
    k_type.intel.genotype.lineage_id = k_type.metabolism.lineage_id;

    world.spawn_entity(r_type);
    world.spawn_entity(k_type);

    // In a resource boom, Strategy R should multiply faster
    for _ in 0..100 {
        world.update(&mut env).unwrap();
        // Keep energy high to simulate boom
        for (_handle, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 500.0;
        }
        if world.get_population_count() > 100 {
            break;
        }
    }

    let entities = world.get_all_entities();
    let r_count = entities
        .iter()
        .filter(|e| e.intel.genotype.maturity_gene < 1.0)
        .count();
    let k_count = entities
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
