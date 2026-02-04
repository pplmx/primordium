mod common;
use common::{EntityBuilder, TestBehavior, WorldBuilder};
use uuid::Uuid;

#[tokio::test]
async fn test_r_vs_k_dominance_in_resource_boom() {
    let lid_r = Uuid::from_u128(100);
    let lid_k = Uuid::from_u128(200);
    let id_r = Uuid::from_u128(1);
    let id_k = Uuid::from_u128(2);

    let mut world_builder = WorldBuilder::new()
        .with_seed(42)
        .with_config(|c| {
            c.world.deterministic = true;
            c.world.max_food = 500;
            c.metabolism.maturity_age = 50;
            c.world.disaster_chance = 0.0;
        })
        .with_memory(lid_r, "goal", 1.0)
        .with_memory(lid_k, "threat", 1.0);

    let r_type = EntityBuilder::new()
        .id(id_r)
        .at(10.0, 10.0)
        .energy(500.0)
        .max_energy(1000.0)
        .lineage(lid_r)
        .with_behavior(TestBehavior::Altruist)
        .build();
    let mut r_type = r_type;
    r_type.intel.genotype.maturity_gene = 0.5;
    r_type.intel.genotype.reproductive_investment = 0.2;
    world_builder = world_builder.with_entity(r_type);

    let k_type = EntityBuilder::new()
        .id(id_k)
        .at(20.0, 20.0)
        .energy(500.0)
        .max_energy(1000.0)
        .lineage(lid_k)
        .with_behavior(TestBehavior::Aggressive)
        .build();
    let mut k_type = k_type;
    k_type.intel.genotype.maturity_gene = 5.0;
    k_type.intel.genotype.reproductive_investment = 0.8;
    world_builder = world_builder.with_entity(k_type);

    let (mut world, mut env) = world_builder.build();
    world.update(&mut env).expect("Warmup failed");

    for _ in 0..150 {
        world.update(&mut env).expect("Update failed");

        let handles: Vec<_> = world.get_sorted_handles();
        for h in handles {
            if let Ok(mut met) = world
                .ecs
                .get::<&mut primordium_lib::model::state::Metabolism>(h)
            {
                met.energy = 800.0;
                met.prev_energy = 800.0;
            }
        }
        if world.get_population_count() > 100 {
            break;
        }
    }

    let entities = world.get_all_entities();
    let r_count = entities
        .iter()
        .filter(|e| e.metabolism.lineage_id == lid_r)
        .count();
    let k_count = entities
        .iter()
        .filter(|e| e.metabolism.lineage_id == lid_k)
        .count();

    assert!(
        r_count > k_count,
        "R-strategists should out-multiply K-strategists in resource booms. R: {}, K: {}",
        r_count,
        k_count
    );
}
