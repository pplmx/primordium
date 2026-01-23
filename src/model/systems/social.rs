//! Social system - handles predation, reproduction, and legendary archiving.

use crate::model::config::{AppConfig, GameMode};
use crate::model::history::{HistoryLogger, Legend, LiveEvent, PopulationStats};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::{Entity, EntityRole};
use crate::model::state::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::systems::intel;
use crate::model::world::EntitySnapshot;
use chrono::Utc;
use rand::Rng;
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
    pub energy_transfers: &'a mut Vec<(usize, f64)>,
}

/// Territorial aggression bonus calculation.
pub fn get_territorial_aggression(entity: &Entity) -> f64 {
    let dist_from_home = ((entity.physics.x - entity.physics.home_x).powi(2)
        + (entity.physics.y - entity.physics.home_y).powi(2))
    .sqrt();
    if dist_from_home < 8.0 {
        1.5 // 50% more aggressive near home
    } else {
        1.0
    }
}

/// Check if two entities are in the same tribe.
pub fn are_same_tribe(e1: &Entity, e2: &Entity) -> bool {
    let color_dist = (e1.physics.r as i32 - e2.physics.r as i32).abs()
        + (e1.physics.g as i32 - e2.physics.g as i32).abs()
        + (e1.physics.b as i32 - e2.physics.b as i32).abs();
    color_dist < 60
}

/// Check if entity can share energy (>70% full)
pub fn can_share(entity: &Entity) -> bool {
    entity.metabolism.energy > entity.metabolism.max_energy * 0.7
}

/// Share energy with another entity.
pub fn share_energy(entity: &mut Entity, max_amount: f64) -> f64 {
    let share = max_amount.min(entity.metabolism.energy * 0.15); // Share up to 15%
    entity.metabolism.energy -= share;
    share
}

/// Handle predation between entities.
pub fn handle_predation(idx: usize, entities: &mut [Entity], ctx: &mut PredationContext) {
    let territorial_bonus = get_territorial_aggression(&entities[idx]);
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
            && !are_same_tribe(&entities[idx], &entities[t_idx])
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
/// Handle energy sharing between same-tribe entities.
pub fn handle_sharing(idx: usize, entities: &mut [Entity], ctx: &mut PredationContext) {
    if !can_share(&entities[idx]) || entities[idx].intel.last_share_intent < 0.5 {
        return;
    }

    let targets = ctx
        .spatial_hash
        .query(entities[idx].physics.x, entities[idx].physics.y, 2.0);

    for t_idx in targets {
        if t_idx == idx {
            continue;
        }

        let target_snap = &ctx.snapshots[t_idx];
        if target_snap.id == entities[idx].id {
            continue;
        }

        let color_dist = (entities[idx].physics.r as i32 - target_snap.r as i32).abs()
            + (entities[idx].physics.g as i32 - target_snap.g as i32).abs()
            + (entities[idx].physics.b as i32 - target_snap.b as i32).abs();

        if color_dist < 60 && target_snap.energy < entities[idx].metabolism.energy {
            let sharing_amount = entities[idx].metabolism.energy * 0.05; // Share 5%
            if sharing_amount > 1.0 {
                entities[idx].metabolism.energy -= sharing_amount;
                ctx.energy_transfers.push((t_idx, sharing_amount));
            }
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
    repro_logic(idx, entities, killed_ids, spatial_hash, config, tick)
}

pub(crate) fn repro_logic(
    idx: usize,
    entities: &mut [Entity],
    killed_ids: &HashSet<uuid::Uuid>,
    spatial_hash: &SpatialHash,
    config: &AppConfig,
    tick: u64,
) -> Option<Entity> {
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
        let mut child_genotype = intel::crossover_genotypes(
            &entities[idx].intel.genotype,
            &entities[m_idx].intel.genotype,
        );
        intel::mutate_genotype(&mut child_genotype, &config.evolution);

        let child = reproduce_with_mate(
            &mut entities[idx],
            tick,
            child_genotype,
            config.evolution.speciation_rate,
        );

        entities[idx].metabolism.energy -= 50.0;
        Some(child)
    } else {
        let child = reproduce_asexual(&mut entities[idx], tick, &config.evolution);
        Some(child)
    }
}

/// Perform asexual reproduction.
pub fn reproduce_asexual(
    parent: &mut Entity,
    tick: u64,
    config: &crate::model::config::EvolutionConfig,
) -> Entity {
    let mut rng = rand::thread_rng();

    let child_energy = parent.metabolism.energy / 2.0;
    parent.metabolism.energy = child_energy;
    parent.metabolism.offspring_count += 1;

    let mut child_genotype = parent.intel.genotype.clone();
    intel::mutate_genotype(&mut child_genotype, config);

    let r = {
        let change = rng.gen_range(-15..=15);
        (parent.physics.r as i16 + change).clamp(0, 255) as u8
    };
    let g = {
        let change = rng.gen_range(-15..=15);
        (parent.physics.g as i16 + change).clamp(0, 255) as u8
    };
    let b = {
        let change = rng.gen_range(-15..=15);
        (parent.physics.b as i16 + change).clamp(0, 255) as u8
    };

    let mut child_role = parent.metabolism.role;
    if rng.gen::<f32>() < config.speciation_rate {
        child_role = match parent.metabolism.role {
            crate::model::state::entity::EntityRole::Herbivore => {
                crate::model::state::entity::EntityRole::Carnivore
            }
            crate::model::state::entity::EntityRole::Carnivore => {
                crate::model::state::entity::EntityRole::Herbivore
            }
        };
    }

    use crate::model::state::entity::{Health, Intel, Metabolism, Physics};
    use uuid::Uuid;

    Entity {
        id: Uuid::new_v4(),
        parent_id: Some(parent.id),
        physics: Physics {
            x: parent.physics.x,
            y: parent.physics.y,
            vx: parent.physics.vx,
            vy: parent.physics.vy,
            r,
            g,
            b,
            symbol: '●',
            home_x: parent.physics.x,
            home_y: parent.physics.y,
            sensing_range: child_genotype.sensing_range,
            max_speed: child_genotype.max_speed,
        },
        metabolism: Metabolism {
            role: child_role,
            energy: child_energy,
            max_energy: child_genotype.max_energy,
            peak_energy: child_energy,
            birth_tick: tick,
            generation: parent.metabolism.generation + 1,
            offspring_count: 0,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: (parent.health.immunity + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0),
        },
        intel: Intel {
            genotype: child_genotype,
            last_hidden: [0.0; 6],
            last_aggression: 0.0,
            last_share_intent: 0.0,
        },
    }
}

/// Perform sexual reproduction with a mate.
pub fn reproduce_with_mate(
    parent: &mut Entity,
    tick: u64,
    child_genotype: crate::model::state::entity::Genotype,
    speciation_rate: f32,
) -> Entity {
    let mut rng = rand::thread_rng();
    let child_energy = parent.metabolism.energy / 2.0;
    parent.metabolism.energy = child_energy;
    parent.metabolism.offspring_count += 1;

    let mut child_role = parent.metabolism.role;
    if rng.gen::<f32>() < speciation_rate {
        child_role = match parent.metabolism.role {
            crate::model::state::entity::EntityRole::Herbivore => {
                crate::model::state::entity::EntityRole::Carnivore
            }
            crate::model::state::entity::EntityRole::Carnivore => {
                crate::model::state::entity::EntityRole::Herbivore
            }
        };
    }

    use crate::model::state::entity::{Health, Intel, Metabolism, Physics};
    use uuid::Uuid;

    Entity {
        id: Uuid::new_v4(),
        parent_id: Some(parent.id),
        physics: Physics {
            x: parent.physics.x,
            y: parent.physics.y,
            vx: parent.physics.vx,
            vy: parent.physics.vy,
            r: parent.physics.r,
            g: parent.physics.g,
            b: parent.physics.b,
            symbol: '●',
            home_x: parent.physics.x,
            home_y: parent.physics.y,
            sensing_range: child_genotype.sensing_range,
            max_speed: child_genotype.max_speed,
        },
        metabolism: Metabolism {
            role: child_role,
            energy: child_energy,
            max_energy: child_genotype.max_energy,
            peak_energy: child_energy,
            birth_tick: tick,
            generation: parent.metabolism.generation + 1,
            offspring_count: 0,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: (parent.health.immunity + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0),
        },
        intel: Intel {
            genotype: child_genotype,
            last_hidden: [0.0; 6],
            last_aggression: 0.0,
            last_share_intent: 0.0,
        },
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
            brain_dna: entity.intel.genotype.brain.clone(),
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

    #[test]
    fn test_reproduce_asexual_splits_energy() {
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 0.0,
            mutation_amount: 0.0,
            drift_rate: 0.0,
            drift_amount: 0.0,
            speciation_rate: 0.0,
        };

        let mut parent = Entity::new(50.0, 25.0, 0);
        parent.metabolism.energy = 200.0;

        let child = reproduce_asexual(&mut parent, 100, &config);

        assert_eq!(parent.metabolism.energy, 100.0);
        assert_eq!(child.metabolism.energy, 100.0);
        assert_eq!(parent.metabolism.offspring_count, 1);
        assert_eq!(child.parent_id, Some(parent.id));
        assert_eq!(child.metabolism.generation, 2);
    }

    #[test]
    fn test_are_same_tribe_similar_colors() {
        let mut entity1 = Entity::new(0.0, 0.0, 0);
        entity1.physics.r = 100;
        entity1.physics.g = 100;
        entity1.physics.b = 100;

        let mut entity2 = Entity::new(0.0, 0.0, 0);
        entity2.physics.r = 110;
        entity2.physics.g = 105;
        entity2.physics.b = 120;

        assert!(are_same_tribe(&entity1, &entity2));
    }

    #[test]
    fn test_can_share_high_energy() {
        let mut entity = Entity::new(0.0, 0.0, 0);
        entity.metabolism.energy = 160.0;
        assert!(can_share(&entity));
    }

    #[test]
    fn test_territorial_aggression_near_home() {
        let mut entity = Entity::new(50.0, 50.0, 0);
        entity.physics.home_x = 50.0;
        entity.physics.home_y = 50.0;
        entity.physics.x = 52.0;
        entity.physics.y = 52.0;

        assert_eq!(get_territorial_aggression(&entity), 1.5);
    }
}
