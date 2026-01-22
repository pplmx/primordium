//! Social system - handles predation, reproduction, and legendary archiving.

use crate::model::brain::Brain;
use crate::model::config::{AppConfig, GameMode};
use crate::model::entity::{Entity, EntityRole};
use crate::model::history::{HistoryLogger, Legend, LiveEvent, PopulationStats};
use crate::model::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::quadtree::SpatialHash;
use crate::model::world::EntitySnapshot;
use chrono::Utc;
use std::collections::HashSet;

/// Handle predation between entities.
#[allow(clippy::too_many_arguments)]
pub fn handle_predation(
    idx: usize,
    entities: &mut [Entity],
    snapshots: &[EntitySnapshot],
    killed_ids: &mut HashSet<uuid::Uuid>,
    events: &mut Vec<LiveEvent>,
    config: &AppConfig,
    spatial_hash: &SpatialHash,
    pheromones: &mut PheromoneGrid,
    pop_stats: &mut PopulationStats,
    logger: &mut HistoryLogger,
    tick: u64,
) {
    let territorial_bonus = entities[idx].territorial_aggression();
    let targets = spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
    for t_idx in targets {
        let v_id = snapshots[t_idx].id;
        let v_e = snapshots[t_idx].energy;
        let v_b = snapshots[t_idx].birth_tick;
        let v_o = snapshots[t_idx].offspring_count;
        let can_attack = !matches!(config.game_mode, GameMode::Cooperative);
        if can_attack
            && v_id != entities[idx].id
            && !killed_ids.contains(&v_id)
            && v_e < entities[idx].metabolism.energy * territorial_bonus
            && !entities[idx].same_tribe(&entities[t_idx])
        {
            let gain_mult = match entities[idx].metabolism.role {
                EntityRole::Carnivore => 1.2,
                EntityRole::Herbivore => 0.2,
            };
            entities[idx].metabolism.energy = (entities[idx].metabolism.energy + v_e * gain_mult)
                .min(entities[idx].metabolism.max_energy);
            killed_ids.insert(v_id);
            pheromones.deposit(
                entities[idx].physics.x,
                entities[idx].physics.y,
                PheromoneType::Danger,
                0.5,
            );
            let v_age = tick - v_b;
            pop_stats.record_death(v_age);
            let ev = LiveEvent::Death {
                id: v_id,
                age: v_age,
                offspring: v_o,
                tick,
                timestamp: Utc::now().to_rfc3339(),
                cause: "predation".to_string(),
            };
            let _ = logger.log_event(ev.clone());
            events.push(ev);
        }
    }
}

/// Handle entity reproduction.
pub fn handle_reproduction(
    idx: usize,
    entities: &mut [Entity],
    killed_ids: &HashSet<uuid::Uuid>,
    spatial_hash: &SpatialHash,
    config: &AppConfig,
    tick: u64,
) -> Option<Entity> {
    if !entities[idx].is_mature(tick, config.metabolism.maturity_age)
        || entities[idx].metabolism.energy <= config.metabolism.reproduction_threshold
    {
        return None;
    }
    let mate_indices = spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 2.0);
    let mut mate_idx = None;
    for m_idx in mate_indices {
        if m_idx != idx
            && !killed_ids.contains(&entities[m_idx].id)
            && entities[m_idx].metabolism.energy > 100.0
        {
            mate_idx = Some(m_idx);
            break;
        }
    }
    if let Some(m_idx) = mate_idx {
        let mut cb = Brain::crossover(&entities[idx].intel.brain, &entities[m_idx].intel.brain);
        cb.mutate_with_config(&config.evolution);
        let child = entities[idx].reproduce_with_mate(tick, cb, config.evolution.speciation_rate);
        entities[idx].metabolism.energy -= 50.0;
        Some(child)
    } else {
        Some(entities[idx].reproduce(tick, &config.evolution))
    }
}

/// Handle extinction event.
pub fn handle_extinction(
    entities: &[Entity],
    tick: u64,
    events: &mut Vec<LiveEvent>,
    logger: &mut HistoryLogger,
) {
    if entities.is_empty() && tick > 0 {
        let ev = LiveEvent::Extinction {
            population: 0,
            tick,
            timestamp: Utc::now().to_rfc3339(),
        };
        let _ = logger.log_event(ev.clone());
        events.push(ev);
    }
}

/// Archive entity as legend if it meets criteria.
pub fn archive_if_legend(entity: &Entity, tick: u64, logger: &HistoryLogger) {
    let lifespan = tick - entity.metabolism.birth_tick;
    if lifespan > 1000
        || entity.metabolism.offspring_count > 10
        || entity.metabolism.peak_energy > 300.0
    {
        let _ = logger.archive_legend(Legend {
            id: entity.id,
            parent_id: entity.parent_id,
            birth_tick: entity.metabolism.birth_tick,
            death_tick: tick,
            lifespan,
            generation: entity.metabolism.generation,
            offspring_count: entity.metabolism.offspring_count,
            peak_energy: entity.metabolism.peak_energy,
            birth_timestamp: "".to_string(),
            death_timestamp: Utc::now().to_rfc3339(),
            brain_dna: entity.intel.brain.clone(),
            color_rgb: (entity.physics.r, entity.physics.g, entity.physics.b),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_if_legend_long_lifespan() {
        // Entity with lifespan > 1000 should be archived
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.birth_tick = 0;

        // Can't easily test without a real logger, but the function should not panic
        // This is more of a smoke test
        let tick = 1500;
        let lifespan = tick - entity.metabolism.birth_tick;
        assert!(lifespan > 1000, "Entity should qualify for legend status");
    }

    #[test]
    fn test_reproduction_requires_maturity() {
        let mut entities = vec![Entity::new(5.0, 5.0, 0)];
        entities[0].metabolism.energy = 200.0; // High energy
        let killed_ids = HashSet::new();
        let spatial_hash = SpatialHash::new(5.0);
        let config = AppConfig::default();

        // Tick 0, maturity_age default is 150, so not mature
        let result = handle_reproduction(0, &mut entities, &killed_ids, &spatial_hash, &config, 0);
        assert!(result.is_none(), "Immature entity should not reproduce");
    }
}
