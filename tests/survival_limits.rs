use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_death_by_brain_bloat() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    world.food.clear(); // Ensure no food for starvation test
    let mut env = Environment::default();

    // 1. Create an entity with a massive brain
    let mut e = Entity::new(10.0, 10.0, 0);
    e.metabolism.energy = 100.0;

    // Manually add 500 connections to the brain
    for i in 0..500 {
        e.intel
            .genotype
            .brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 0,
                to: 27,
                weight: 1.0,
                enabled: true,
                innovation: 1000 + i,
            });
    }

    world.entities.push(e);

    // 2. Run update. Maintenance cost = 500 * 0.005 = 2.5 + 6 * 0.02 = 2.62 per tick.
    // Plus base idle cost (0.5). Total approx 3.1 per tick.
    // In 40 ticks, it should be dead.
    for _ in 0..40 {
        world.update(&mut env).unwrap();
    }

    assert!(
        world.entities.is_empty(),
        "Organism with bloated brain should have starved to death"
    );
}

#[test]
fn test_high_speed_metabolic_exhaustion() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let _world = World::new(0, config.clone()).unwrap();
    let env = Environment::default();

    // 1. Entity A: Low speed Factor (0.5)
    let mut e_slow = Entity::new(10.0, 10.0, 0);
    e_slow.physics.max_speed = 0.5;
    e_slow.intel.genotype.max_speed = 0.5;
    e_slow.metabolism.energy = 200.0;

    // 2. Entity B: High speed Factor (3.0)
    let mut e_fast = Entity::new(20.0, 20.0, 0);
    e_fast.physics.max_speed = 3.0;
    e_fast.intel.genotype.max_speed = 3.0;
    e_fast.metabolism.energy = 200.0;

    // 3. Run action system directly to verify cost scaling
    use primordium_lib::model::systems::action::{action_system, ActionContext};

    let terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(100, 100, 42);
    let mut pheromones = primordium_lib::model::state::pheromone::PheromoneGrid::new(100, 100);
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        pheromones: &mut pheromones,
        width: 100,
        height: 100,
    };

    // Use same "Full Speed" output for both: [0.0, 0.0, Speed=1.0, ...]
    let outputs = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    action_system(&mut e_slow, outputs, &mut ctx);
    action_system(&mut e_fast, outputs, &mut ctx);

    // Fast entity has 3.0 cap, slow has 0.5 cap.
    // Cost scales with speed_mult which scales with cap.
    assert!(
        e_fast.metabolism.energy < e_slow.metabolism.energy,
        "High speed capability must increase energy drain. Slow: {}, Fast: {}",
        e_slow.metabolism.energy,
        e_fast.metabolism.energy
    );
}

#[test]
fn test_inertia_responsiveness_penalty() {
    let config = AppConfig::default();
    let env = Environment::default();
    let terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(100, 100, 42);
    let mut pheromones = primordium_lib::model::state::pheromone::PheromoneGrid::new(100, 100);

    // 1. Entity A: Small Energy Tank (100) -> Low inertia
    let mut e_light = Entity::new(10.0, 10.0, 0);
    e_light.metabolism.max_energy = 100.0;
    e_light.intel.genotype.max_energy = 100.0;
    e_light.physics.vx = 0.0;

    // 2. Entity B: Huge Energy Tank (500) -> High inertia
    let mut e_heavy = Entity::new(20.0, 20.0, 0);
    e_heavy.metabolism.max_energy = 500.0;
    e_heavy.intel.genotype.max_energy = 500.0;
    e_heavy.physics.vx = 0.0;

    // 3. Apply same neural output (Move X = 1.0)
    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        pheromones: &mut pheromones,
        width: 100,
        height: 100,
    };

    // outputs: [MoveX=1.0, ...]
    let outputs = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    action_system(&mut e_light, outputs, &mut ctx);
    action_system(&mut e_heavy, outputs, &mut ctx);

    // Light entity should have accelerated more than heavy entity
    assert!(
        e_light.physics.vx > e_heavy.physics.vx,
        "Light organism should have higher responsiveness (lower inertia)"
    );
}
