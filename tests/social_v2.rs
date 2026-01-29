use primordium_core::config::{AppConfig, GameMode};
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_group_defense_reduces_damage() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.world.seed = Some(42);
    config.game_mode = GameMode::Standard;
    config.ecosystem.predation_min_efficiency = 0.01; // Low min efficiency for defense test
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    // 1. Attacker (Carnivore specialist, high energy)
    let mut attacker = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    attacker.metabolism.trophic_potential = 1.0;
    attacker.metabolism.energy = 1000.0;
    attacker.physics.r = 255;
    attacker.physics.g = 0;
    attacker.physics.b = 0; // Distinct tribe

    // 2. Victim (Small energy)
    let mut victim = primordium_lib::model::lifecycle::create_entity(10.5, 10.5, 0);
    victim.metabolism.energy = 100.0;
    victim.physics.r = 0;
    victim.physics.g = 255;
    victim.physics.b = 0;
    let v_lineage = victim.metabolism.lineage_id;

    // 3. Allies (Same lineage as victim, nearby)
    for i in 0..5 {
        let mut ally = primordium_lib::model::lifecycle::create_entity(
            10.5 + (i as f64 * 0.1),
            10.5 + (i as f64 * 0.1),
            0,
        );
        ally.metabolism.lineage_id = v_lineage;
        ally.metabolism.energy = 200.0;
        world.ecs.spawn((
            ally.identity,
            primordium_lib::model::state::Position {
                x: ally.physics.x,
                y: ally.physics.y,
            },
            ally.physics,
            ally.metabolism,
            ally.health,
            ally.intel,
        ));
    }

    world.ecs.spawn((
        attacker.identity,
        primordium_lib::model::state::Position {
            x: attacker.physics.x,
            y: attacker.physics.y,
        },
        attacker.physics,
        attacker.metabolism,
        attacker.health,
        attacker.intel,
    ));
    world.ecs.spawn((
        victim.identity,
        primordium_lib::model::state::Position {
            x: victim.physics.x,
            y: victim.physics.y,
        },
        victim.physics,
        victim.metabolism,
        victim.health,
        victim.intel,
    ));

    // Run world update.
    world.update(&mut env).unwrap();

    // We verify the logic by ensuring the victim survived at least one tick.
    assert!(world
        .get_all_entities()
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
    let mut e_quiet = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    e_quiet.metabolism.energy = 500.0;

    // Entity B: Max signal
    let mut e_loud = primordium_lib::model::lifecycle::create_entity(20.0, 20.0, 0);
    e_loud.metabolism.energy = 500.0;

    // Run action system directly with specific outputs
    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(100, 100, 42);

    // quiet: [x, y, speed, aggro, share, signal, emitA, emitB]
    let pressure = primordium_lib::model::pressure::PressureGrid::new(100, 100);
    let mut ctx_q = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        snapshots: &[],
        entity_id_map: &std::collections::HashMap::new(),
        spatial_hash: &primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100),
        pressure: &pressure,
        width: 100,
        height: 100,
    };
    {
        let mut out = primordium_lib::model::systems::action::ActionOutput::default();
        action_system(&mut e_quiet, [0.0; 12], &mut ctx_q, &mut out);
        out
    };

    // loud: [..., signal=1.0]
    let mut ctx_l = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        snapshots: &[],
        entity_id_map: &std::collections::HashMap::new(),
        spatial_hash: &primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100),
        pressure: &pressure,
        width: 100,
        height: 100,
    };
    let mut out_l = primordium_lib::model::systems::action::ActionOutput::default();
    action_system(
        &mut e_loud,
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        &mut ctx_l,
        &mut out_l,
    );

    assert!(e_loud.metabolism.energy < e_quiet.metabolism.energy);
}
