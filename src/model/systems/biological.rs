//! Biological system - handles infection, pathogen emergence, and death processing.

use crate::model::history::{LiveEvent, PopulationStats};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::Entity;
use crate::model::state::pathogen::Pathogen;
use chrono::Utc;
use rand::Rng;
use std::collections::HashSet;

/// Process entity infection and immunity.
pub fn biological_system(entity: &mut Entity) {
    process_infection(entity);
}

/// Try to infect an entity with a pathogen.
pub fn try_infect(entity: &mut Entity, pathogen: &Pathogen) -> bool {
    if entity.health.pathogen.is_some() {
        return false;
    }

    let mut rng = rand::thread_rng();
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
    process_infection(&mut entities[idx]);
    for p in active_pathogens {
        if rng.gen_bool(0.005) {
            try_infect(&mut entities[idx], p);
        }
    }
    if let Some(p) = entities[idx].health.pathogen.clone() {
        for n_idx in spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 2.0) {
            if n_idx != idx
                && !killed_ids.contains(&entities[n_idx].id)
                && try_infect(&mut entities[n_idx], &p)
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
    use crate::model::history::{HistoryLogger, PopulationStats};
    use crate::model::state::entity::Entity;

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

    #[test]
    fn test_handle_death_creates_event() {
        let entities = vec![Entity::new(5.0, 5.0, 0)];
        let mut pop_stats = PopulationStats::new();
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
        let entities = vec![Entity::new(5.0, 5.0, 0)];
        let mut pop_stats = PopulationStats::new();
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
        let mut entities = vec![Entity::new(5.0, 5.0, 0)];
        entities[0].metabolism.birth_tick = 0;
        let mut pop_stats = PopulationStats::new();
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
        let mut entities = vec![Entity::new(5.0, 5.0, 0), Entity::new(5.5, 5.5, 0)];
        let killed_ids = HashSet::new();
        let active_pathogens: Vec<Pathogen> = vec![];
        let mut spatial_hash = SpatialHash::new(5.0);
        spatial_hash.insert(5.0, 5.0, 0);
        spatial_hash.insert(5.5, 5.5, 1);
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

    #[test]
    fn test_biological_system_with_infected_entity() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.health.pathogen = Some(Pathogen::new_random());
        entity.health.infection_timer = 5;

        let initial_timer = entity.health.infection_timer;
        biological_system(&mut entity);

        // Timer should decrease or infection should progress
        // The exact behavior depends on process_infection implementation
        assert!(
            entity.health.infection_timer != initial_timer || entity.health.pathogen.is_none(),
            "Infection should progress"
        );
    }
}
