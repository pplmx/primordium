use primordium_lib::model::config::AppConfig;
use primordium_lib::model::history::{HistoryLogger, PopulationStats};
use primordium_lib::model::state::entity::Entity;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::pheromone::PheromoneGrid;
use primordium_lib::model::systems::social::{handle_predation, PredationContext};
use primordium_lib::model::world::{InternalEntitySnapshot, World};
use std::collections::HashSet;

#[test]
fn test_overgrazing_feedback_loop() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).unwrap();
    let mut env = Environment::default();

    let f0 = world.terrain.get_cell(10, 10).fertility;
    world.pop_stats.biomass_h = 10000.0;
    world.update(&mut env).unwrap();

    let f1 = world.terrain.get_cell(10, 10).fertility;
    assert!(f1 < f0, "Massive overgrazing should deplete fertility");
}

#[test]
fn test_hunter_competition_impact() {
    let config = AppConfig::default();
    let mut pop_stats = PopulationStats::new();
    let mut logger = HistoryLogger::new_dummy();
    let mut spatial_hash = primordium_lib::model::quadtree::SpatialHash::new(10.0, 100, 100);
    let mut pheromones = PheromoneGrid::new(100, 100);

    let mut hunter = Entity::new(10.0, 10.0, 0);
    hunter.metabolism.trophic_potential = 1.0;
    hunter.metabolism.energy = 100.0;
    hunter.metabolism.max_energy = 1000.0;

    let mut prey = Entity::new(10.5, 10.5, 0);
    prey.metabolism.energy = 100.0;
    prey.metabolism.max_energy = 1000.0;
    prey.physics.r = 0;

    spatial_hash.insert(10.5, 10.5, 1);

    let snap = vec![
        InternalEntitySnapshot {
            id: hunter.id,
            lineage_id: hunter.metabolism.lineage_id,
            x: 10.0,
            y: 10.0,
            energy: 100.0,
            birth_tick: 0,
            offspring_count: 0,
            r: 255,
            g: 255,
            b: 255,
            rank: 0.5,
            status: primordium_lib::model::state::entity::EntityStatus::Foraging,
        },
        InternalEntitySnapshot {
            id: prey.id,
            lineage_id: prey.metabolism.lineage_id,
            x: 10.5,
            y: 10.5,
            energy: 100.0,
            birth_tick: 0,
            offspring_count: 0,
            r: 0,
            g: 0,
            b: 0,
            rank: 0.5,
            status: primordium_lib::model::state::entity::EntityStatus::Foraging,
        },
    ];

    pop_stats.biomass_c = 0.0;
    let mut entities_1 = vec![hunter.clone(), prey.clone()];
    let mut killed_1 = HashSet::new();
    let mut ctx1 = PredationContext {
        snapshots: &snap,
        killed_ids: &mut killed_1,
        events: &mut vec![],
        config: &config,
        spatial_hash: &spatial_hash,
        pheromones: &mut pheromones,
        pop_stats: &mut pop_stats,
        logger: &mut logger,
        tick: 0,
        energy_transfers: &mut vec![],
        lineage_consumption: &mut vec![],
    };
    handle_predation(0, &mut entities_1, &mut ctx1);
    let energy1 = entities_1[0].metabolism.energy;

    pop_stats.biomass_c = 20000.0;
    let mut entities_2 = vec![hunter.clone(), prey.clone()];
    let mut killed_2 = HashSet::new();
    let mut ctx2 = PredationContext {
        snapshots: &snap,
        killed_ids: &mut killed_2,
        events: &mut vec![],
        config: &config,
        spatial_hash: &spatial_hash,
        pheromones: &mut pheromones,
        pop_stats: &mut pop_stats,
        logger: &mut logger,
        tick: 0,
        energy_transfers: &mut vec![],
        lineage_consumption: &mut vec![],
    };
    handle_predation(0, &mut entities_2, &mut ctx2);
    let energy2 = entities_2[0].metabolism.energy;

    assert!(
        energy2 < energy1,
        "High competition should reduce energy gain from kill"
    );
}
