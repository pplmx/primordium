use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::systems::environment as environment_system;
use primordium_lib::model::world::World;

#[test]
fn test_wall_collisions() {
    let mut config = AppConfig::default();
    config.world.width = 10;
    config.world.height = 10;
    config.world.initial_population = 0;

    let mut world = World::new(0, config).unwrap();

    // Place a wall block to prevent tunneling
    for dx in 5..=7 {
        for dy in 5..=7 {
            world.terrain.set_cell_type(
                dx,
                dy,
                primordium_lib::model::state::terrain::TerrainType::Wall,
            );
        }
    }

    // Starting at 4.5 ensuring we hit the wall block
    let mut entity = Entity::new(4.5, 4.5, 0);
    entity.physics.vx = 1.0;
    entity.physics.vy = 1.0;

    world.entities.push(entity);

    let mut env = Environment::default();
    world.update(&mut env).unwrap();

    let e = &world.entities[0];
    assert!(
        e.physics.vx < 0.0,
        "vx should be reversed, got {}",
        e.physics.vx
    );
    assert!(
        e.physics.vy < 0.0,
        "vy should be reversed, got {}",
        e.physics.vy
    );
}

#[test]
fn test_dust_bowl_trigger() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.disaster_chance = 1.0; // P47: Forced deterministic trigger
    let mut world = World::new(0, config).unwrap();

    let mut env = Environment {
        cpu_usage: 95.0,
        ..Environment::default()
    };
    for _ in 0..11 {
        environment_system::update_events(&mut env);
    }

    // Need > 300 entities for trigger
    for _ in 0..310 {
        let mut e = Entity::new(5.0, 5.0, 0);
        e.metabolism.energy = 1000.0;
        world.entities.push(e);
    }

    let mut triggered = false;
    // P47: Drastically reduced loop as it's now deterministic
    for _ in 0..10 {
        world.update(&mut env).unwrap();
        if world.terrain.dust_bowl_timer > 0 {
            triggered = true;
            break;
        }
    }

    assert!(
        triggered,
        "Dust Bowl should trigger immediately under high heat and population with chance=1.0"
    );
}
