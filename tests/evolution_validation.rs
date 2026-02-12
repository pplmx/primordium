mod common;
use common::{EntityBuilder, TestBehavior, WorldBuilder};
use uuid::Uuid;

#[tokio::test]
#[ignore]
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
            c.metabolism.metamorphosis_trigger_maturity = 0.1;
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
    std::sync::Arc::make_mut(&mut r_type.intel.genotype).maturity_gene = 0.5;
    std::sync::Arc::make_mut(&mut r_type.intel.genotype).reproductive_investment = 0.2;
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
    std::sync::Arc::make_mut(&mut k_type.intel.genotype).maturity_gene = 0.8;
    std::sync::Arc::make_mut(&mut k_type.intel.genotype).reproductive_investment = 0.7;
    world_builder = world_builder.with_entity(k_type);

    let (mut world, mut env) = world_builder.build();
    assert!(
        world.get_population_count() >= 2,
        "Should have at least 2 entities after spawn"
    );
    world.update(&mut env).expect("Warmup failed");

    let mut max_pop = 0;
    for i in 0..150 {
        let pop_before = world.get_population_count();
        world.update(&mut env).expect("Update failed");
        let pop_after = world.get_population_count();

        if pop_after > max_pop {
            max_pop = pop_after;
        }

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

        if world.get_population_count() == 0 {
            println!(
                "Population died at tick {}: before={}, after={}",
                i, pop_before, pop_after
            );
            break;
        }
        if world.get_population_count() > 100 {
            break;
        }
    }
    println!("Max population: {}", max_pop);

    let entities = world.get_all_entities();
    let r_count = entities
        .iter()
        .filter(|e| e.metabolism.lineage_id == lid_r)
        .count();
    let k_count = entities
        .iter()
        .filter(|e| e.metabolism.lineage_id == lid_k)
        .count();

    println!("Total entities: {}", entities.len());
    println!(
        "Unique lineage IDs: {}",
        entities
            .iter()
            .map(|e| e.metabolism.lineage_id)
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
    println!("R count (lid_r): {}", r_count);
    println!("K count (lid_k): {}", k_count);

    assert!(
        !entities.is_empty(),
        "Population should not be zero. Max was: {}",
        max_pop
    );
}
