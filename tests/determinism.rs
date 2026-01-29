use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_determinism_consistency() {
    let mut config = AppConfig::default();
    config.world.seed = Some(12345);
    config.world.deterministic = true;
    config.world.initial_population = 50;

    let mut world1 = World::new(50, config.clone()).unwrap();
    let mut env1 = Environment::default();

    let mut world2 = World::new(50, config.clone()).unwrap();
    let mut env2 = Environment::default();

    // Run for 100 ticks
    for _ in 0..100 {
        world1.update(&mut env1).unwrap();
        world2.update(&mut env2).unwrap();
    }

    // Verify entity counts
    assert_eq!(
        world1.get_population_count(),
        world2.get_population_count(),
        "Entity counts should match"
    );

    // Verify specific entity properties
    let entities1 = world1.get_all_entities();
    let entities2 = world2.get_all_entities();
    for i in 0..entities1.len() {
        let e1 = &entities1[i];
        let e2 = &entities2[i];
        assert_eq!(
            e1.identity.id, e2.identity.id,
            "Entity IDs should match at index {}",
            i
        );
        assert_eq!(
            e1.physics.x, e2.physics.x,
            "Entity X should match at index {}",
            i
        );
        assert_eq!(
            e1.physics.y, e2.physics.y,
            "Entity Y should match at index {}",
            i
        );
        assert_eq!(
            e1.metabolism.energy, e2.metabolism.energy,
            "Entity energy should match at index {}",
            i
        );
    }

    // Verify food
    assert_eq!(
        world1.get_food_count(),
        world2.get_food_count(),
        "Food counts should match"
    );
    let food1: Vec<_> = world1
        .ecs
        .query::<&primordium_lib::model::state::Food>()
        .iter()
        .map(|(_, f)| f.clone())
        .collect();
    let food2: Vec<_> = world2
        .ecs
        .query::<&primordium_lib::model::state::Food>()
        .iter()
        .map(|(_, f)| f.clone())
        .collect();
    for i in 0..food1.len() {
        assert_eq!(food1[i].x, food2[i].x);
        assert_eq!(food1[i].y, food2[i].y);
    }
}
