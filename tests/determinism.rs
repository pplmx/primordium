use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_world_determinism() {
    let mut config = AppConfig::default();
    config.world.seed = Some(12345);
    config.world.deterministic = true;

    // Create two identical worlds
    let mut world1 = World::new(50, config.clone()).unwrap();
    let mut world2 = World::new(50, config.clone()).unwrap();

    let mut env = Environment {
        carbon_level: 300.0,
        oxygen_level: 21.0,
        ..Default::default()
    };

    // Run both worlds for 100 ticks
    for _ in 0..100 {
        world1.update(&mut env).unwrap();
        world2.update(&mut env).unwrap();
    }

    // Compare states via deterministic hash
    let hash1 = world1.deterministic_hash(&env);
    let hash2 = world2.deterministic_hash(&env);

    assert_eq!(hash1, hash2, "Worlds diverged after 100 ticks!");
}
