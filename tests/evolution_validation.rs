mod common;
use common::{EntityBuilder, WorldBuilder};

#[tokio::test]
async fn test_r_vs_k_dominance_in_resource_boom() {
    let mut world_builder = WorldBuilder::new().with_config(|c| {
        c.world.max_food = 500; // Abundant food
    });

    // Strategy R: Fast maturity (50), Low investment (0.2)
    // Low investment means babies start weak, but they mature fast.
    let r_type = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(500.0)
        .max_energy(500.0)
        .lineage(uuid::Uuid::new_v4())
        .build();
    // Manual modification needed as Builder lacks genetic tuner yet
    let mut r_type = r_type;
    r_type.intel.genotype.maturity_gene = 0.5;
    r_type.intel.genotype.reproductive_investment = 0.2;
    world_builder = world_builder.with_entity(r_type);

    // Strategy K: Slow maturity (200), High investment (0.8)
    // High investment means babies start with 80% parent energy.
    let k_type = EntityBuilder::new()
        .at(20.0, 20.0)
        .energy(500.0)
        .max_energy(1000.0)
        .lineage(uuid::Uuid::new_v4())
        .build();
    let mut k_type = k_type;
    k_type.intel.genotype.maturity_gene = 5.0; // Handicap K maturity
    k_type.intel.genotype.reproductive_investment = 0.8;
    world_builder = world_builder.with_entity(k_type);

    let (mut world, mut env) = world_builder.build();

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
