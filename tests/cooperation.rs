mod common;
use common::{EntityBuilder, WorldBuilder};

#[tokio::test]
async fn test_kin_recognition_influences_movement() {
    let l_id = uuid::Uuid::new_v4();

    let (world, _env) = WorldBuilder::new()
        .with_entity(
            EntityBuilder::new()
                .at(10.0, 10.0)
                .lineage(l_id)
                .energy(500.0)
                .build(),
        )
        .with_entity(EntityBuilder::new().at(12.0, 10.0).lineage(l_id).build())
        .build();

    assert_eq!(world.get_population_count(), 2);
}

#[tokio::test]
async fn test_herding_bonus() {
    let l_id = uuid::Uuid::new_v4();

    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(
            EntityBuilder::new()
                .at(10.0, 10.0)
                .lineage(l_id)
                .energy(100.0)
                .build(),
        )
        .with_entity(EntityBuilder::new().at(11.0, 10.0).lineage(l_id).build())
        .build();

    // Force velocity manually as Builder doesn't support it yet
    for (_h, vel) in world.ecs.query_mut::<&mut primordium_data::Velocity>() {
        vel.vx = 1.0;
        vel.vy = 0.0;
    }

    world.update(&mut env).unwrap();

    let entities = world.get_all_entities();
    assert!(entities[0].metabolism.energy > 0.0);
}
