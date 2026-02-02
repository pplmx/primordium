use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_long_term_determinism_regression() {
    let mut config = AppConfig::default();
    config.world.width = 100;
    config.world.height = 100;
    config.world.seed = Some(42);
    config.world.deterministic = true;

    // Run 1
    let mut world1 = World::new(50, config.clone()).unwrap();
    let mut env1 = Environment::default();
    let mut hashes1 = Vec::new();

    for _ in 0..200 {
        let _ = world1.update(&mut env1).unwrap();
        hashes1.push(world1.deterministic_hash(&env1));
    }

    // Run 2
    let mut world2 = World::new(50, config.clone()).unwrap();
    let mut env2 = Environment::default();
    let mut hashes2 = Vec::new();

    for _ in 0..200 {
        let _ = world2.update(&mut env2).unwrap();
        hashes2.push(world2.deterministic_hash(&env2));
    }

    // Compare
    for i in 0..hashes1.len() {
        assert_eq!(
            hashes1[i],
            hashes2[i],
            "Non-deterministic state at tick {}",
            i + 1
        );
    }
}

#[tokio::test]
async fn test_determinism_with_parallelism_interference() {
    let mut config = AppConfig::default();
    config.world.width = 60;
    config.world.height = 60;
    config.world.seed = Some(999);
    config.world.deterministic = true;

    let mut world1 = World::new(30, config.clone()).unwrap();
    let mut env1 = Environment::default();
    for _ in 0..100 {
        let _ = world1.update(&mut env1).unwrap();
    }
    let final_hash1 = world1.deterministic_hash(&env1);

    let mut world2 = World::new(30, config.clone()).unwrap();
    let mut env2 = Environment::default();
    for _ in 0..100 {
        let _ = world2.update(&mut env2).unwrap();
    }
    let final_hash2 = world2.deterministic_hash(&env2);

    assert_eq!(final_hash1, final_hash2);
}
