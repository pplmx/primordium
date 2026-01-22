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

/// Context for predation operations, reducing parameter count.
pub struct PredationContext<'a> {
    pub snapshots: &'a [EntitySnapshot],
    pub killed_ids: &'a mut HashSet<uuid::Uuid>,
    pub events: &'a mut Vec<LiveEvent>,
    pub config: &'a AppConfig,
    pub spatial_hash: &'a SpatialHash,
    pub pheromones: &'a mut PheromoneGrid,
    pub pop_stats: &'a mut PopulationStats,
    pub logger: &'a mut HistoryLogger,
    pub tick: u64,
}

/// Handle predation between entities.
pub fn handle_predation(idx: usize, entities: &mut [Entity], ctx: &mut PredationContext) {
    let territorial_bonus = entities[idx].territorial_aggression();
    let targets = ctx
        .spatial_hash
        .query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
    for t_idx in targets {
        let v_id = ctx.snapshots[t_idx].id;
        let v_e = ctx.snapshots[t_idx].energy;
        let v_b = ctx.snapshots[t_idx].birth_tick;
        let v_o = ctx.snapshots[t_idx].offspring_count;
        let can_attack = !matches!(ctx.config.game_mode, GameMode::Cooperative);
        if can_attack
            && v_id != entities[idx].id
            && !ctx.killed_ids.contains(&v_id)
            && v_e < entities[idx].metabolism.energy * territorial_bonus
            && !entities[idx].same_tribe(&entities[t_idx])
        {
            let gain_mult = match entities[idx].metabolism.role {
                EntityRole::Carnivore => 1.2,
                EntityRole::Herbivore => 0.2,
            };
            entities[idx].metabolism.energy = (entities[idx].metabolism.energy + v_e * gain_mult)
                .min(entities[idx].metabolism.max_energy);
            ctx.killed_ids.insert(v_id);
            ctx.pheromones.deposit(
                entities[idx].physics.x,
                entities[idx].physics.y,
                PheromoneType::Danger,
                0.5,
            );
            let v_age = ctx.tick - v_b;
            ctx.pop_stats.record_death(v_age);
            let ev = LiveEvent::Death {
                id: v_id,
                age: v_age,
                offspring: v_o,
                tick: ctx.tick,
                timestamp: Utc::now().to_rfc3339(),
                cause: "predation".to_string(),
            };
            let _ = ctx.logger.log_event(ev.clone());
            ctx.events.push(ev);
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

/// Check if an entity qualifies for legend status.
pub fn is_legend_worthy(entity: &Entity, tick: u64) -> bool {
    let lifespan = tick - entity.metabolism.birth_tick;
    lifespan > 1000
        || entity.metabolism.offspring_count > 10
        || entity.metabolism.peak_energy > 300.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============== Unit Tests ==============

    #[test]
    fn test_is_legend_worthy_by_lifespan() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.birth_tick = 0;

        // Not legend worthy at tick 500
        assert!(!is_legend_worthy(&entity, 500));

        // Legend worthy at tick 1500 (lifespan > 1000)
        assert!(is_legend_worthy(&entity, 1500));
    }

    #[test]
    fn test_is_legend_worthy_by_offspring() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.offspring_count = 5;
        assert!(!is_legend_worthy(&entity, 100));

        entity.metabolism.offspring_count = 15;
        assert!(is_legend_worthy(&entity, 100));
    }

    #[test]
    fn test_is_legend_worthy_by_peak_energy() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.peak_energy = 200.0;
        assert!(!is_legend_worthy(&entity, 100));

        entity.metabolism.peak_energy = 350.0;
        assert!(is_legend_worthy(&entity, 100));
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

    #[test]
    fn test_reproduction_requires_energy_threshold() {
        let mut entities = vec![Entity::new(5.0, 5.0, 0)];
        entities[0].metabolism.energy = 50.0; // Low energy
        entities[0].metabolism.birth_tick = 0;
        let killed_ids = HashSet::new();
        let spatial_hash = SpatialHash::new(5.0);
        let config = AppConfig::default();

        // Tick 200 (mature), but low energy
        let result =
            handle_reproduction(0, &mut entities, &killed_ids, &spatial_hash, &config, 200);
        assert!(result.is_none(), "Low energy entity should not reproduce");
    }

    #[test]
    fn test_reproduction_asexual_when_no_mate() {
        let mut entities = vec![Entity::new(5.0, 5.0, 0)];
        entities[0].metabolism.energy = 200.0;
        entities[0].metabolism.birth_tick = 0;
        let killed_ids = HashSet::new();
        let spatial_hash = SpatialHash::new(5.0);
        let config = AppConfig::default();

        // Tick 200, high energy, no mate nearby
        let result =
            handle_reproduction(0, &mut entities, &killed_ids, &spatial_hash, &config, 200);
        assert!(result.is_some(), "Entity should reproduce asexually");

        let child = result.unwrap();
        assert_eq!(child.parent_id, Some(entities[0].id));
    }

    #[test]
    fn test_reproduction_sexual_with_mate() {
        let mut entities = vec![
            Entity::new(5.0, 5.0, 0),
            Entity::new(5.5, 5.5, 0), // Close enough to be a mate
        ];
        entities[0].metabolism.energy = 200.0;
        entities[0].metabolism.birth_tick = 0;
        entities[1].metabolism.energy = 150.0;
        entities[1].metabolism.birth_tick = 0;

        let killed_ids = HashSet::new();
        let mut spatial_hash = SpatialHash::new(5.0);
        spatial_hash.insert(5.0, 5.0, 0);
        spatial_hash.insert(5.5, 5.5, 1);
        let config = AppConfig::default();

        let initial_energy = entities[0].metabolism.energy;
        let result =
            handle_reproduction(0, &mut entities, &killed_ids, &spatial_hash, &config, 200);

        assert!(result.is_some(), "Entity should reproduce sexually");
        assert!(
            entities[0].metabolism.energy < initial_energy,
            "Parent should lose energy"
        );
    }

    #[test]
    fn test_extinction_event_on_empty_population() {
        let entities: Vec<Entity> = vec![];
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_extinction(&entities, 100, &mut events, &mut logger);

        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], LiveEvent::Extinction { .. }));
    }

    #[test]
    fn test_no_extinction_event_with_population() {
        let entities = vec![Entity::new(5.0, 5.0, 0)];
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_extinction(&entities, 100, &mut events, &mut logger);

        assert!(
            events.is_empty(),
            "No extinction event when population exists"
        );
    }

    #[test]
    fn test_no_extinction_at_tick_zero() {
        let entities: Vec<Entity> = vec![];
        let mut events = Vec::new();
        let mut logger = HistoryLogger::new().unwrap();

        handle_extinction(&entities, 0, &mut events, &mut logger);

        assert!(events.is_empty(), "No extinction event at tick 0");
    }
}
