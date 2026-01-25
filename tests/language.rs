use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_semantic_pheromone_roundtrip() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    // 1. Entity A: Emits Signal A
    let mut e_emitter = Entity::new(10.0, 10.0, 0);
    // [movX, movY, speed, aggro, share, color, emitA, emitB, bond, dig, build]
    let outputs = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];

    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &world.terrain,
        snapshots: &[],
        width: 100,
        height: 100,
    };
    let (p_deps, s_deps) = action_system(&mut e_emitter, outputs, &mut ctx);
    for d in p_deps {
        world.pheromones.deposit(d.x, d.y, d.ptype, d.amount);
    }
    for d in s_deps {
        world.sound.deposit(d.x, d.y, d.amount);
    }

    // 2. Verify Signal A is in the grid
    let cell = world.pheromones.get_cell(10, 10);
    assert!(cell.sig_a_strength > 0.4);
    assert_eq!(cell.sig_b_strength, 0.0);

    // 3. Verify sense_all logic used in world.rs
    let (_, _, sa, sb) = world.pheromones.sense_all(10.0, 10.0, 1.0);
    assert!(sa > 0.0);
    assert_eq!(sb, 0.0);
}
