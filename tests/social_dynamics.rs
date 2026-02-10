mod common;
use common::{EntityBuilder, TestBehavior, WorldBuilder};
use uuid::Uuid;

#[tokio::test]
async fn test_tribe_solidarity_no_aggression() {
    let lid = Uuid::from_u128(100);
    let id1 = Uuid::from_u128(1);
    let id2 = Uuid::from_u128(2);

    let e1 = EntityBuilder::new()
        .id(id1)
        .at(10.0, 10.0)
        .energy(5000.0)
        .max_energy(10000.0)
        .color(100, 100, 100)
        .lineage(lid)
        .build();

    let mut e1_mut = e1.clone();
    e1_mut.metabolism.trophic_potential = 1.0;

    let e2 = EntityBuilder::new()
        .id(id2)
        .at(10.5, 10.5)
        .energy(5000.0)
        .max_energy(10000.0)
        .color(100, 100, 100)
        .lineage(lid)
        .build();

    let mut e2_mut = e2.clone();
    e2_mut.metabolism.trophic_potential = 0.0;

    let world_builder = WorldBuilder::new()
        .with_seed(123)
        .with_config(|c| {
            c.world.deterministic = true;
            c.world.disaster_chance = 0.0;
            c.metabolism.reproduction_threshold = 1000000.0;
        })
        .with_entity(e1_mut)
        .with_entity(e2_mut);

    let (mut world, mut env) = world_builder.build();
    world.update(&mut env).expect("Warmup failed");

    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
    }
    assert!(
        world.get_population_count() >= 2,
        "Hunter attacked its own tribe!"
    );
}

#[tokio::test]
async fn test_energy_sharing_between_allies() {
    let id1 = Uuid::from_u128(1);
    let id2 = Uuid::from_u128(2);

    let e1 = EntityBuilder::new()
        .id(id1)
        .at(10.0, 10.0)
        .energy(800.0)
        .max_energy(1000.0)
        .color(200, 200, 200)
        .with_behavior(TestBehavior::Altruist)
        .build();

    let e2 = EntityBuilder::new()
        .id(id2)
        .at(10.2, 10.2)
        .energy(10.0)
        .max_energy(1000.0)
        .color(200, 200, 200)
        .build();

    let mut e2_clone = e2.clone();
    e2_clone.intel.genotype = e1.intel.genotype.clone();
    e2_clone.metabolism.lineage_id = e1.metabolism.lineage_id;

    let world_builder = WorldBuilder::new().with_seed(456).with_config(|c| {
        c.world.deterministic = true;
        c.world.disaster_chance = 0.0;
        c.metabolism.reproduction_threshold = 1000000.0;
    });
    let (mut world, mut env) = world_builder.with_entity(e1).with_entity(e2_clone).build();

    let e2_id = id2;

    let mut shared = false;
    for _ in 0..100 {
        world.update(&mut env).expect("Update failed");

        let entities = world.get_all_entities();
        if let Some(e2_curr) = entities.iter().find(|e| e.identity.id == e2_id) {
            if e2_curr.metabolism.energy > 15.0 {
                shared = true;
                break;
            }
        }

        let handles: Vec<_> = world.get_sorted_handles();
        for h in handles {
            if let (Ok(phys), Ok(mut met), Ok(mut intel), Ok(ident)) = (
                world.ecs.get::<&primordium_lib::model::state::Physics>(h),
                world
                    .ecs
                    .get::<&mut primordium_lib::model::state::Metabolism>(h),
                world.ecs.get::<&mut primordium_lib::model::state::Intel>(h),
                world.ecs.get::<&primordium_lib::model::state::Identity>(h),
            ) {
                if phys.r == 200 && ident.id != e2_id {
                    met.energy = 800.0;
                    intel.last_share_intent = 1.0;
                }
            }
        }
    }
    assert!(shared, "Energy sharing did not occur between allies");
}

#[tokio::test]
async fn test_inter_tribe_predation() {
    let id1 = Uuid::from_u128(1);
    let id2 = Uuid::from_u128(2);

    let world_builder = WorldBuilder::new().with_seed(789).with_config(|c| {
        c.world.deterministic = true;
        c.world.disaster_chance = 0.0;
        c.metabolism.reproduction_threshold = 1000000.0;
    });
    let mut e1 = EntityBuilder::new()
        .id(id1)
        .at(10.0, 10.0)
        .color(255, 0, 0)
        .energy(5000.0)
        .max_energy(10000.0)
        .with_behavior(TestBehavior::Aggressive)
        .lineage(Uuid::from_u128(777))
        .build();
    e1.metabolism.trophic_potential = 1.0;
    e1.physics.sensing_range = 20.0;
    std::sync::Arc::make_mut(&mut e1.intel.genotype).sensing_range = 20.0;

    let mut e2 = EntityBuilder::new()
        .id(id2)
        .at(10.1, 10.1)
        .color(0, 0, 255)
        .energy(100.0)
        .lineage(Uuid::from_u128(888))
        .build();
    e2.metabolism.trophic_potential = 0.0;
    e2.physics.max_speed = 0.0;
    std::sync::Arc::make_mut(&mut e2.intel.genotype).max_speed = 0.0;

    let (mut world, mut env) = world_builder.with_entity(e1).with_entity(e2).build();

    for _ in 0..200 {
        world.update(&mut env).expect("Update failed");

        if world.get_population_count() == 1 {
            break;
        }
    }
    assert_eq!(
        world.get_population_count(),
        1,
        "Predator failed to survive or failed to eat prey (Pop: {})",
        world.get_population_count()
    );
}
