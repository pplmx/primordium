use primordium_lib::model::brain::Connection;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_death_by_brain_bloat() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    let mut food_handles = Vec::new();
    for (h, _) in world
        .ecs
        .query::<&primordium_lib::model::state::Food>()
        .iter()
    {
        food_handles.push(h);
    }
    for h in food_handles {
        let _ = world.ecs.despawn(h);
    }

    let mut env = Environment::default();

    let mut e = lifecycle::create_entity(10.0, 10.0, 0);
    e.metabolism.energy = 100.0;

    for i in 0..500 {
        e.intel.genotype.brain.connections.push(Connection {
            from: 0,
            to: 25,
            weight: 1.0,
            enabled: true,
            innovation: 1000 + i,
        });
    }

    world.spawn_entity(e);

    for _ in 0..100 {
        world.update(&mut env).unwrap();
    }

    assert!(
        world.get_population_count() == 0,
        "Organism with bloated brain should have starved to death"
    );
}

#[tokio::test]
async fn test_high_speed_metabolic_exhaustion() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let _world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    let mut e_slow = lifecycle::create_entity(10.0, 10.0, 0);
    e_slow.physics.max_speed = 0.5;
    e_slow.intel.genotype.max_speed = 0.5;
    e_slow.metabolism.energy = 200.0;

    let mut e_fast = lifecycle::create_entity(20.0, 20.0, 0);
    e_fast.physics.max_speed = 3.0;
    e_fast.intel.genotype.max_speed = 3.0;
    e_fast.metabolism.energy = 200.0;

    use primordium_lib::model::systems::action::{action_system, ActionContext};

    let terrain = primordium_lib::model::terrain::TerrainGrid::generate(100, 100, 42);
    let pressure = primordium_lib::model::pressure::PressureGrid::new(100, 100);
    let influence = primordium_lib::model::influence::InfluenceGrid::new(100, 100);
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        influence: &influence,
        snapshots: &[],
        entity_id_map: &std::collections::HashMap::new(),
        spatial_hash: &primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100),
        pressure: &pressure,
        width: 100,
        height: 100,
    };

    let outputs = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    {
        let mut out = primordium_lib::model::systems::action::ActionOutput::default();
        action_system(&mut e_slow, outputs, &mut ctx, &mut out);
    }
    {
        let mut out = primordium_lib::model::systems::action::ActionOutput::default();
        action_system(&mut e_fast, outputs, &mut ctx, &mut out);
    }

    assert!(e_fast.metabolism.energy < e_slow.metabolism.energy);
}

#[tokio::test]
async fn test_inertia_responsiveness_penalty() {
    let config = AppConfig::default();
    let env = Environment::default();
    let terrain = primordium_lib::model::terrain::TerrainGrid::generate(100, 100, 42);

    let mut e_light = lifecycle::create_entity(10.0, 10.0, 0);
    e_light.metabolism.max_energy = 100.0;
    e_light.intel.genotype.max_energy = 100.0;
    e_light.velocity.vx = 0.0;

    let mut e_heavy = lifecycle::create_entity(20.0, 20.0, 0);
    e_heavy.metabolism.max_energy = 500.0;
    e_heavy.intel.genotype.max_energy = 500.0;
    e_heavy.velocity.vx = 0.0;

    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let pressure = primordium_lib::model::pressure::PressureGrid::new(100, 100);
    let influence = primordium_lib::model::influence::InfluenceGrid::new(100, 100);
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        influence: &influence,
        snapshots: &[],
        entity_id_map: &std::collections::HashMap::new(),
        spatial_hash: &primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100),
        pressure: &pressure,
        width: 100,
        height: 100,
    };

    let outputs = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    {
        let mut out = primordium_lib::model::systems::action::ActionOutput::default();
        action_system(&mut e_light, outputs, &mut ctx, &mut out);
    }
    {
        let mut out = primordium_lib::model::systems::action::ActionOutput::default();
        action_system(&mut e_heavy, outputs, &mut ctx, &mut out);
    }

    assert!(e_light.velocity.vx > e_heavy.velocity.vx);
}
