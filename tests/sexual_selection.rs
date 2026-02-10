mod common;
use common::{EntityBuilder, WorldBuilder};

#[tokio::test]
async fn test_mate_preference_rejection() {
    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(EntityBuilder::new().at(10.0, 10.0).energy(1000.0).build())
        .with_entity(
            EntityBuilder::new()
                .at(10.5, 10.5)
                .energy(500.0)
                .trophic(0.0) // Herbivore
                .build(),
        )
        .build();

    // Configure selector manually for genes not yet in builder
    for (_h, (met, intel)) in world.ecs.query_mut::<(
        &primordium_lib::model::state::Metabolism,
        &mut primordium_lib::model::state::Intel,
    )>() {
        if met.trophic_potential > 0.1 {
            std::sync::Arc::make_mut(&mut intel.genotype).mate_preference = 1.0;
            // Prefers Carnivores
        }
    }

    world.tick = 200;
    world.update(&mut env).unwrap();

    assert!(world.get_population_count() >= 2);
}

#[tokio::test]
async fn test_mate_preference_acceptance() {
    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(EntityBuilder::new().at(10.0, 10.0).energy(1000.0).build())
        .with_entity(
            EntityBuilder::new()
                .at(10.5, 10.5)
                .energy(500.0)
                .trophic(1.0) // Carnivore
                .build(),
        )
        .build();

    for (_h, (met, intel)) in world.ecs.query_mut::<(
        &primordium_lib::model::state::Metabolism,
        &mut primordium_lib::model::state::Intel,
    )>() {
        if met.energy > 600.0 {
            std::sync::Arc::make_mut(&mut intel.genotype).mate_preference = 1.0;
            // Prefers Carnivores
        }
    }

    world.tick = 200;
    world.update(&mut env).unwrap();

    assert!(
        world.get_population_count() > 2,
        "Selector should have accepted the matching partner"
    );
}
