use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_parallel_determinism() {
    let mut config = AppConfig::default();
    config.world.width = 50;
    config.world.height = 50;
    config.world.seed = Some(12345);
    config.world.deterministic = true;

    let mut world1 = World::new(20, config.clone()).unwrap();
    let mut world2 = World::new(20, config.clone()).unwrap();
    let mut env1 = Environment::default();
    let mut env2 = Environment::default();

    for _ in 0..50 {
        let _ = world1.update(&mut env1).unwrap();
        let _ = world2.update(&mut env2).unwrap();
    }

    let entities1 = world1.get_all_entities();
    let entities2 = world2.get_all_entities();

    assert_eq!(entities1.len(), entities2.len());
    for (e1, e2) in entities1.iter().zip(entities2.iter()) {
        assert_eq!(e1.identity.id, e2.identity.id);
        assert_eq!(e1.position.x, e2.position.x);
        assert_eq!(e1.position.y, e2.position.y);
        assert_eq!(e1.metabolism.energy, e2.metabolism.energy);
    }
}
