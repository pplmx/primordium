//! Biological system - handles infection, pathogen emergence, and death processing.

use crate::model::config::AppConfig;
use crate::model::history::{LiveEvent, PopulationStats};
use crate::model::spatial_hash::SpatialHash;
use crate::model::systems::social;
use primordium_data::{Entity, Pathogen, Specialization};
use rand::Rng;
use std::collections::HashSet;

/// Process entity infection, immunity, and genetic drift.
pub fn biological_system<R: Rng>(
    entity: &mut Entity,
    population_count: usize,
    config: &AppConfig,
    rng: &mut R,
) {
    process_infection(entity);

    // Phase 46: Reputation Recovery
    if entity.intel.reputation < 1.0 {
        entity.intel.reputation = (entity.intel.reputation + 0.001).min(1.0);
    }

    // Phase 39: Genetic Drift
    if population_count > 0
        && population_count < 10
        && rng.gen_bool(config.evolution.drift_rate as f64)
    {
        match rng.gen_range(0..4) {
            0 => {
                entity.intel.genotype.metabolic_niche = rng.gen_range(0.0..1.0);
            }
            1 => {
                entity.intel.genotype.max_speed = rng.gen_range(0.5..1.5);
            }
            2 => {
                entity.intel.genotype.sensing_range = rng.gen_range(5.0..15.0);
            }
            _ => {
                entity.intel.genotype.max_energy = rng.gen_range(50.0..150.0);
            }
        }
    }

    // Phase 53: Specialized Castes
    if entity.intel.specialization.is_none() {
        let progress = 0.01;
        if entity.intel.last_aggression > 0.8 {
            social::increment_spec_meter(entity, Specialization::Soldier, progress, config);
        }
        if entity.intel.last_share_intent > 0.8 {
            social::increment_spec_meter(entity, Specialization::Provider, progress, config);
        }
    }

    // Apply brain metabolic cost
    let mut brain_maintenance = (entity.intel.genotype.brain.nodes.len() as f64
        * config.brain.hidden_node_cost)
        + (entity.intel.genotype.brain.connections.len() as f64 * config.brain.connection_cost);

    // Caste adjustments
    if let Some(Specialization::Soldier) = entity.intel.specialization {
        brain_maintenance *= config.social.war_zone_mult * 0.6; // Using war zone mult as proxy
    }

    entity.metabolism.energy -= brain_maintenance;
}

/// Try to infect an entity with a pathogen.
pub fn try_infect<R: Rng>(entity: &mut Entity, pathogen: &Pathogen, rng: &mut R) -> bool {
    if entity.health.pathogen.is_some() {
        return false;
    }

    // Roll for infection: virulence vs immunity
    let chance = (pathogen.virulence - entity.health.immunity).max(0.01);
    if rng.gen::<f32>() < chance {
        entity.health.pathogen = Some(pathogen.clone());
        entity.health.infection_timer = pathogen.duration;
        return true;
    }
    false
}

/// Process active infection effects on an entity.
pub fn process_infection(entity: &mut Entity) {
    if let Some(p) = &entity.health.pathogen {
        entity.metabolism.energy -= p.lethality as f64;
        if entity.health.infection_timer > 0 {
            entity.health.infection_timer -= 1;
        } else {
            // Recovered! Gain immunity
            entity.health.pathogen = None;
            entity.health.immunity = (entity.health.immunity + 0.1).min(1.0);
        }
    }
}

/// Handle potential pathogen emergence (random spawn).
pub fn handle_pathogen_emergence(active_pathogens: &mut Vec<Pathogen>, _rng: &mut impl Rng) {
    if rand::thread_rng().gen_bool(0.0001) {
        active_pathogens.push(crate::model::pathogen::create_random_pathogen());
    }
}

/// Process infection spread between entities.
pub fn handle_infection<R: Rng>(
    idx: usize,
    entities: &mut [Entity],
    killed_ids: &HashSet<uuid::Uuid>,
    active_pathogens: &[Pathogen],
    spatial_hash: &SpatialHash,
    rng: &mut R,
) {
    process_infection(&mut entities[idx]);
    for p in active_pathogens {
        if rng.gen_bool(0.005) {
            try_infect(&mut entities[idx], p, rng);
        }
    }
    if let Some(p) = entities[idx].health.pathogen.clone() {
        for n_idx in spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 2.0) {
            if n_idx != idx
                && !killed_ids.contains(&entities[n_idx].id)
                && try_infect(&mut entities[n_idx], &p, rng)
            {}
        }
    }
}

/// Handle entity death, logging and statistics.
pub fn handle_death(
    idx: usize,
    entities: &[Entity],
    tick: u64,
    cause: &str,
    pop_stats: &mut PopulationStats,
    events: &mut Vec<LiveEvent>,
    logger: &mut crate::model::history::HistoryLogger,
) {
    let age = tick - entities[idx].metabolism.birth_tick;
    crate::model::history::record_stat_death(pop_stats, age);
    let ev = LiveEvent::Death {
        id: entities[idx].id,
        age,
        offspring: entities[idx].metabolism.offspring_count,
        tick,
        timestamp: chrono::Utc::now().to_rfc3339(),
        cause: cause.to_string(),
    };
    let _ = logger.log_event(ev.clone());
    events.push(ev);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::history::{HistoryLogger, PopulationStats};

    #[test]
    fn test_biological_system_processes_infection() {
        let mut entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        let config = AppConfig::default();
        let mut rng = rand::thread_rng();
        // Entity without infection should only have brain metabolic cost
        let initial_energy = entity.metabolism.energy;
        biological_system(&mut entity, 100, &config, &mut rng);

        // Calculate expected brain maintenance
        let nodes_count = entity.intel.genotype.brain.nodes.len();
        let connections_count = entity.intel.genotype.brain.connections.len();
        let brain_maintenance = (nodes_count as f64 * config.brain.hidden_node_cost)
            + (connections_count as f64 * config.brain.connection_cost);

        assert!((entity.metabolism.energy - (initial_energy - brain_maintenance)).abs() < 1e-6);
    }

    #[test]
    fn test_biological_system_with_infected_entity() {
        let mut entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        let config = AppConfig::default();
        let mut rng = rand::thread_rng();
        entity.health.pathogen = Some(crate::model::pathogen::create_random_pathogen());
        entity.health.infection_timer = 5;

        let initial_timer = entity.health.infection_timer;
        let initial_energy = entity.metabolism.energy;
        biological_system(&mut entity, 100, &config, &mut rng);

        // Timer should decrease or infection should progress
        assert!(
            entity.health.infection_timer != initial_timer || entity.health.pathogen.is_none(),
            "Infection should progress"
        );

        // Energy should decrease due to infection and brain metabolic cost
        assert!(
            entity.metabolism.energy < initial_energy,
            "Energy should decrease"
        );
    }

    #[test]
    fn test_biological_system_genetic_drift() {
        let mut entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        let mut config = AppConfig::default();
        config.evolution.drift_rate = 1.0; // Force drift
        let mut rng = rand::thread_rng();

        let initial_niche = entity.intel.genotype.metabolic_niche;
        let initial_speed = entity.intel.genotype.max_speed;
        let initial_range = entity.intel.genotype.sensing_range;
        let initial_energy_max = entity.intel.genotype.max_energy;

        let mut drifted = false;
        for _ in 0..100 {
            biological_system(&mut entity, 5, &config, &mut rng);
            if (entity.intel.genotype.metabolic_niche - initial_niche).abs() > 1e-6
                || (entity.intel.genotype.max_speed - initial_speed).abs() > 1e-6
                || (entity.intel.genotype.sensing_range - initial_range).abs() > 1e-6
                || (entity.intel.genotype.max_energy - initial_energy_max).abs() > 1e-6
            {
                drifted = true;
                break;
            }
        }

        assert!(drifted, "Genotype should have drifted under low population");
    }

    #[test]
    fn test_pathogen_emergence_probability() {
        let mut pathogens = Vec::new();
        let mut rng = rand::thread_rng();

        // Run many times to check probabilistic behavior
        for _ in 0..100000 {
            handle_pathogen_emergence(&mut pathogens, &mut rng);
        }

        // With 0.0001 probability over 100k iterations, expect ~10 pathogens
        assert!(
            !pathogens.is_empty(),
            "Should have spawned at least one pathogen"
        );
        assert!(
            pathogens.len() < 50,
            "Should not have spawned too many pathogens"
        );
    }

    #[test]
    fn test_handle_death_creates_event() {
        let entities = vec![crate::model::lifecycle::create_entity(5.0, 5.0, 0)];
        let mut pop_stats = PopulationStats::default();
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_death(
            0,
            &entities,
            100,
            "starvation",
            &mut pop_stats,
            &mut events,
            &mut logger,
        );

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], LiveEvent::Death { .. }));
    }

    #[test]
    fn test_handle_death_records_cause() {
        let entities = vec![crate::model::lifecycle::create_entity(5.0, 5.0, 0)];
        let mut pop_stats = PopulationStats::default();
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_death(
            0,
            &entities,
            100,
            "predation",
            &mut pop_stats,
            &mut events,
            &mut logger,
        );

        if let LiveEvent::Death { cause, .. } = &events[0] {
            assert_eq!(cause, "predation");
        } else {
            panic!("Expected Death event");
        }
    }

    #[test]
    fn test_handle_death_updates_stats() {
        let mut entities = vec![crate::model::lifecycle::create_entity(5.0, 5.0, 0)];
        entities[0].metabolism.birth_tick = 0;
        let mut pop_stats = PopulationStats::default();
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_death(
            0,
            &entities,
            100,
            "starvation",
            &mut pop_stats,
            &mut events,
            &mut logger,
        );

        // avg_lifespan should be updated
        assert!(
            pop_stats.avg_lifespan > 0.0,
            "Lifespan stat should be recorded"
        );
    }

    #[test]
    fn test_handle_infection_no_pathogen_no_spread() {
        let mut entities = vec![
            crate::model::lifecycle::create_entity(5.0, 5.0, 0),
            crate::model::lifecycle::create_entity(5.5, 5.5, 0),
        ];
        let killed_ids = HashSet::new();
        let active_pathogens: Vec<Pathogen> = vec![];
        let mut spatial_hash = SpatialHash::new(5.0, 100, 100);
        spatial_hash.build_parallel(&[(5.0, 5.0), (5.5, 5.5)], 100, 100);
        let mut rng = rand::thread_rng();

        // Without any pathogens, neither entity should get infected
        handle_infection(
            0,
            &mut entities,
            &killed_ids,
            &active_pathogens,
            &spatial_hash,
            &mut rng,
        );

        assert!(
            entities[0].health.pathogen.is_none(),
            "Entity should not be infected without pathogens"
        );
        assert!(
            entities[1].health.pathogen.is_none(),
            "Neighbor should not be infected"
        );
    }
}
