use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_death_by_brain_bloat() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config.clone()).unwrap();
    world.food.clear();
    let mut env = Environment::default();

    let mut e = Entity::new(10.0, 10.0, 0);
    e.metabolism.energy = 100.0;

    for i in 0..500 {
        e.intel
            .genotype
            .brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 0,
                to: 25,
                weight: 1.0,
                enabled: true,
                innovation: 1000 + i,
            });
    }

    world.entities.push(e);

    for _ in 0..100 {
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

    let mut e_slow = Entity::new(10.0, 10.0, 0);
    e_slow.physics.max_speed = 0.5;
    e_slow.intel.genotype.max_speed = 0.5;
    e_slow.metabolism.energy = 200.0;

    let mut e_fast = Entity::new(20.0, 20.0, 0);
    e_fast.physics.max_speed = 3.0;
    e_fast.intel.genotype.max_speed = 3.0;
    e_fast.metabolism.energy = 200.0;

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

    let outputs = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

    action_system(&mut e_slow, outputs, &mut ctx);
    action_system(&mut e_fast, outputs, &mut ctx);

    assert!(e_fast.metabolism.energy < e_slow.metabolism.energy);
}

#[test]
fn test_inertia_responsiveness_penalty() {
    let config = AppConfig::default();
    let env = Environment::default();
    let terrain = primordium_lib::model::state::terrain::TerrainGrid::generate(100, 100, 42);
    let mut pheromones = primordium_lib::model::state::pheromone::PheromoneGrid::new(100, 100);

    let mut e_light = Entity::new(10.0, 10.0, 0);
    e_light.metabolism.max_energy = 100.0;
    e_light.intel.genotype.max_energy = 100.0;
    e_light.physics.vx = 0.0;

    let mut e_heavy = Entity::new(20.0, 20.0, 0);
    e_heavy.metabolism.max_energy = 500.0;
    e_heavy.intel.genotype.max_energy = 500.0;
    e_heavy.physics.vx = 0.0;

    use primordium_lib::model::systems::action::{action_system, ActionContext};
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        pheromones: &mut pheromones,
        width: 100,
        height: 100,
    };

    let outputs = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    action_system(&mut e_light, outputs, &mut ctx);
    action_system(&mut e_heavy, outputs, &mut ctx);

    assert!(e_light.physics.vx > e_heavy.physics.vx);
}
