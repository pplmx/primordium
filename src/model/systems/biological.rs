//! Biological system - handles infection, pathogen emergence, and death processing.

use crate::model::entity::Entity;
use crate::model::history::{LiveEvent, PopulationStats};
use crate::model::pathogen::Pathogen;
use crate::model::quadtree::SpatialHash;
use chrono::Utc;
use rand::Rng;
use std::collections::HashSet;

/// Process entity infection and immunity.
pub fn biological_system(entity: &mut Entity) {
    entity.process_infection();
}

/// Handle potential pathogen emergence (random spawn).
pub fn handle_pathogen_emergence(active_pathogens: &mut Vec<Pathogen>, rng: &mut impl Rng) {
    if rng.gen_bool(0.0001) {
        active_pathogens.push(Pathogen::new_random());
    }
}

/// Process infection spread between entities.
pub fn handle_infection(
    idx: usize,
    entities: &mut [Entity],
    killed_ids: &HashSet<uuid::Uuid>,
    active_pathogens: &[Pathogen],
    spatial_hash: &SpatialHash,
    rng: &mut impl Rng,
) {
    entities[idx].process_infection();
    for p in active_pathogens {
        if rng.gen_bool(0.005) {
            entities[idx].try_infect(p);
        }
    }
    if let Some(p) = entities[idx].health.pathogen.clone() {
        for n_idx in spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 2.0) {
            if n_idx != idx
                && !killed_ids.contains(&entities[n_idx].id)
                && entities[n_idx].try_infect(&p)
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
    pop_stats.record_death(age);
    let ev = LiveEvent::Death {
        id: entities[idx].id,
        age,
        offspring: entities[idx].metabolism.offspring_count,
        tick,
        timestamp: Utc::now().to_rfc3339(),
        cause: cause.to_string(),
    };
    let _ = logger.log_event(ev.clone());
    events.push(ev);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::entity::Entity;

    #[test]
    fn test_biological_system_processes_infection() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        // Entity without infection should not change
        let initial_energy = entity.metabolism.energy;
        biological_system(&mut entity);
        assert_eq!(entity.metabolism.energy, initial_energy);
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
}
