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
        world1.entities.len(),
        world2.entities.len(),
        "Entity counts should match"
    );

    // Verify specific entity properties
    for i in 0..world1.entities.len() {
        let e1 = &world1.entities[i];
        let e2 = &world2.entities[i];
        assert_eq!(e1.id, e2.id, "Entity IDs should match at index {}", i);
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
        world1.food.len(),
        world2.food.len(),
        "Food counts should match"
    );
    for i in 0..world1.food.len() {
        assert_eq!(world1.food[i].x, world2.food[i].x);
        assert_eq!(world1.food[i].y, world2.food[i].y);
    }
}
