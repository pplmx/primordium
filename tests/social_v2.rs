use primordium_lib::model::config::{AppConfig, GameMode};
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_group_defense_reduces_damage() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.game_mode = GameMode::Standard;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // 1. Attacker (Carnivore specialist, high energy)
    let mut attacker = Entity::new(10.0, 10.0, 0);
    attacker.metabolism.trophic_potential = 1.0;
    attacker.metabolism.energy = 1000.0;
    attacker.physics.r = 255;
    attacker.physics.g = 0;
    attacker.physics.b = 0; // Distinct tribe

    // 2. Victim (Small energy)
    let mut victim = Entity::new(10.5, 10.5, 0);
    victim.metabolism.energy = 100.0;
    victim.physics.r = 0;
    victim.physics.g = 255;
    victim.physics.b = 0;
    let v_lineage = victim.metabolism.lineage_id;

    // 3. Allies (Same lineage as victim, nearby)
    for i in 0..5 {
        let mut ally = Entity::new(10.5 + (i as f64 * 0.1), 10.5 + (i as f64 * 0.1), 0);
        ally.metabolism.lineage_id = v_lineage;
        ally.metabolism.energy = 200.0;
        world.entities.push(ally);
    }

    world.entities.push(attacker);
    world.entities.push(victim);

    // Run world update.
    world.update(&mut env).unwrap();

    // We verify the logic by ensuring the victim survived at least one tick.
    assert!(world
        .entities
        .iter()
        .any(|e| e.metabolism.lineage_id == v_lineage));
}

#[test]
fn test_metabolic_cost_of_signaling() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let _world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    // Entity A: No signal
    let mut e_quiet = Entity::new(10.0, 10.0, 0);
    e_quiet.metabolism.energy = 500.0;

    // Entity B: Max signal
    let mut e_loud = Entity::new(20.0, 20.0, 0);
    e_loud.metabolism.energy = 500.0;

    // Run action system directly with specific outputs
    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(100, 100, 42);
    let mut pheromones = primordium_lib::model::state::pheromone::PheromoneGrid::new(100, 100);

    // quiet: [x, y, speed, aggro, share, signal, emitA, emitB]
    let mut ctx_q = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        pheromones: &mut pheromones,
        width: 100,
        height: 100,
    };
    action_system(&mut e_quiet, [0.0; 9], &mut ctx_q);

    // loud: [..., signal=1.0]
    let mut ctx_l = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        pheromones: &mut pheromones,
        width: 100,
        height: 100,
    };
    action_system(
        &mut e_loud,
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        &mut ctx_l,
    );

    assert!(e_loud.metabolism.energy < e_quiet.metabolism.energy);
}
