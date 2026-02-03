mod common;
use common::{EntityBuilder, WorldBuilder};

#[tokio::test]
async fn test_simulation_lifecycle() {
    let initial_pop = 50;

    // We can use WorldBuilder for cleaner setup
    let mut world_builder = WorldBuilder::new();

    for _ in 0..initial_pop {
        world_builder = world_builder.with_entity(EntityBuilder::new().build());
    }

    let (mut world, mut env) = world_builder.build();

    assert_eq!(world.get_population_count(), initial_pop);

    // Run for 100 ticks
    for _ in 0..100 {
        world.update(&mut env).expect("World update failed");
    }

    println!(
        "Population after 100 ticks: {}",
        world.get_population_count()
    );

    // Hall of fame should be populated if there were any high performers
    // Or at least initialized
    assert!(world.hall_of_fame.top_living.len() <= 3);

    // Check if time progressed
    assert_eq!(world.tick, 100);
}

#[tokio::test]
async fn test_reproduction_and_genetics() {
    let mut world_builder = WorldBuilder::new().with_config(|c| {
        c.metabolism.maturity_age = 10; // Rapid maturity for test
    });

    for _ in 0..10 {
        world_builder = world_builder.with_entity(
            EntityBuilder::new()
                .energy(200.0) // Start with high energy
                .max_energy(200.0)
                .build(),
        );
    }

    let (mut world, mut env) = world_builder.build();

    // Run ticks - some should reproduce
    let mut total_births = 0;
    for _ in 0..50 {
        // Keep energy high to trigger reproduction
        for (_handle, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 200.0;
        }

        let events = world.update(&mut env).expect("Update failed");

        for event in events {
            if matches!(
                event,
                primordium_lib::model::history::LiveEvent::Birth { .. }
            ) {
                total_births += 1;
            }
        }

        // Safety break if population explodes too much for a unit test
        if world.get_population_count() > 1000 {
            break;
        }
    }

    assert!(
        total_births > 0,
        "No births occurred even with high energy and rapid maturity"
    );
    println!("Total births in 50 ticks: {}", total_births);
}
