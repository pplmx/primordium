use primordium_lib::model::config::AppConfig;
use primordium_lib::model::entity::Entity;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_wall_collisions() {
    let mut config = AppConfig::default();
    config.world.width = 10;
    config.world.height = 10;
    config.world.initial_population = 0;

    let mut world = World::new(0, config).unwrap();

    // Place a wall at (5, 5)
    world
        .terrain
        .set_cell_type(5, 5, primordium_lib::model::terrain::TerrainType::Wall);

    let mut entity = Entity::new(4.0, 4.0, 0);
    entity.physics.vx = 1.0;
    entity.physics.vy = 1.0;

    world.entities.push(entity);

    let env = Environment::default();
    // One tick should move it to (5, 5) where it hits the wall and bounces
    world.update(&env).unwrap();

    let e = &world.entities[0];
    // It should have bounced (negated vx/vy)
    assert!(e.physics.vx < 0.0, "vx should be reversed");
    assert!(e.physics.vy < 0.0, "vy should be reversed");
}

#[test]
fn test_dust_bowl_trigger() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();

    let mut env = Environment {
        cpu_usage: 95.0,
        ..Environment::default()
    };
    for _ in 0..11 {
        env.update_events();
    }

    // Need > 300 entities for trigger
    for _ in 0..500 {
        let mut e = Entity::new(5.0, 5.0, 0);
        e.metabolism.energy = 1000.0; // Prevent immediate starvation
        world.entities.push(e);
    }

    let mut triggered = false;
    for _ in 0..2000 {
        world.update(&env).unwrap();
        if world.terrain.dust_bowl_timer > 0 {
            triggered = true;
            break;
        }
        // Replenish entities if they die
        if world.entities.len() <= 300 {
            for _ in 0..100 {
                let mut e = Entity::new(5.0, 5.0, 0);
                e.metabolism.energy = 1000.0;
                world.entities.push(e);
            }
        }
    }

    assert!(
        triggered,
        "Dust Bowl should trigger under high heat and population"
    );
}
