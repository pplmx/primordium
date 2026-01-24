//! Social system - handles predation, reproduction, and symbiotic relationships.

use crate::model::config::AppConfig;
use crate::model::history::{HistoryLogger, Legend, LiveEvent, PopulationStats};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::{Entity, Health, Intel, Metabolism, Physics};
use crate::model::state::pheromone::PheromoneGrid;
use crate::model::systems::intel;
use crate::model::world::EntitySnapshot;
use chrono::Utc;
use rand::Rng;
use std::collections::HashSet;
use uuid::Uuid;

pub struct PredationContext<'a> {
    pub snapshots: &'a [EntitySnapshot],
    pub killed_ids: &'a mut HashSet<Uuid>,
    pub events: &'a mut Vec<LiveEvent>,
    pub config: &'a AppConfig,
    pub spatial_hash: &'a SpatialHash,
    pub pheromones: &'a mut PheromoneGrid,
    pub pop_stats: &'a mut PopulationStats,
    pub logger: &'a mut HistoryLogger,
    pub tick: u64,
    pub energy_transfers: &'a mut Vec<(usize, f64)>,
    pub lineage_consumption: &'a mut Vec<(Uuid, f64)>,
}

pub fn get_territorial_aggression(entity: &Entity) -> f64 {
    let dist = ((entity.physics.x - entity.physics.home_x).powi(2)
        + (entity.physics.y - entity.physics.home_y).powi(2))
    .sqrt();
    if dist < 8.0 {
        1.5
    } else {
        1.0
    }
}

pub fn calculate_social_rank(entity: &Entity, tick: u64) -> f32 {
    let energy_score =
        (entity.metabolism.energy / entity.metabolism.max_energy).clamp(0.0, 1.0) as f32;
    let age = tick - entity.metabolism.birth_tick;
    let age_score = (age as f32 / 2000.0).min(1.0);
    let offspring_score = (entity.metabolism.offspring_count as f32 / 20.0).min(1.0);
    let rep_score = entity.intel.reputation.clamp(0.0, 1.0);
    0.3 * energy_score + 0.3 * age_score + 0.1 * offspring_score + 0.3 * rep_score
}

pub fn start_tribal_split(entity: &Entity, crowding: f32) -> Option<(u8, u8, u8)> {
    if crowding > 0.8 && entity.intel.rank < 0.2 {
        let mut rng = rand::thread_rng();
        Some((
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        ))
    } else {
        None
    }
}

pub fn are_same_tribe(e1: &Entity, e2: &Entity) -> bool {
    let dist = (e1.physics.r as i32 - e2.physics.r as i32).abs()
        + (e1.physics.g as i32 - e2.physics.g as i32).abs()
        + (e1.physics.b as i32 - e2.physics.b as i32).abs();
    dist < 60
}

pub fn can_share(entity: &Entity) -> bool {
    entity.metabolism.energy > entity.metabolism.max_energy * 0.7
}

pub fn handle_symbiosis(
    idx: usize,
    entities: &[Entity],
    outputs: [f32; 9],
    spatial_hash: &SpatialHash,
) -> Option<Uuid> {
    if outputs[8] < 0.5 {
        return None;
    }
    let nearby = spatial_hash.query(entities[idx].physics.x, entities[idx].physics.y, 2.0);
    for &n_idx in &nearby {
        if n_idx == idx {
            continue;
        }
        if entities[n_idx].intel.bonded_to.is_none() {
            let dist = entities[idx]
                .intel
                .genotype
                .distance(&entities[n_idx].intel.genotype);
            let bias = (entities[idx].intel.genotype.pairing_bias
                + entities[n_idx].intel.genotype.pairing_bias)
                / 2.0;
            if dist < 5.0 || bias > 0.8 {
                return Some(entities[n_idx].id);
            }
        }
    }
    None
}

pub fn reproduce_asexual_parallel(
    parent: &Entity,
    tick: u64,
    config: &crate::model::config::EvolutionConfig,
    population: usize,
) -> (Entity, f32) {
    let mut rng = rand::thread_rng();
    let investment = parent.intel.genotype.reproductive_investment as f64;
    let child_energy = parent.metabolism.energy * investment;
    let mut child_genotype = parent.intel.genotype.clone();
    intel::mutate_genotype(&mut child_genotype, config, population);
    let dist = parent.intel.genotype.distance(&child_genotype);
    if dist > config.speciation_threshold {
        child_genotype.lineage_id = Uuid::new_v4();
    }

    let r = (parent.physics.r as i16 + rng.gen_range(-15..=15)).clamp(0, 255) as u8;
    let g = (parent.physics.g as i16 + rng.gen_range(-15..=15)).clamp(0, 255) as u8;
    let b = (parent.physics.b as i16 + rng.gen_range(-15..=15)).clamp(0, 255) as u8;

    (
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
                trophic_potential: child_genotype.trophic_potential,
                energy: child_energy,
                prev_energy: child_energy,
                max_energy: child_genotype.max_energy,
                peak_energy: child_energy,
                birth_tick: tick,
                generation: parent.metabolism.generation + 1,
                offspring_count: 0,
                lineage_id: child_genotype.lineage_id,
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
                last_signal: 0.0,
                last_vocalization: 0.0,
                reputation: 1.0,
                rank: 0.5,
                bonded_to: None,
                last_inputs: [0.0; 16],
            },
        },
        dist,
    )
}

pub fn reproduce_asexual(
    parent: &mut Entity,
    tick: u64,
    config: &crate::model::config::EvolutionConfig,
    population: usize,
) -> Entity {
    let investment = parent.intel.genotype.reproductive_investment as f64;
    let (baby, _) = reproduce_asexual_parallel(parent, tick, config, population);
    parent.metabolism.energy *= 1.0 - investment;
    parent.metabolism.offspring_count += 1;
    baby
}

pub fn reproduce_with_mate_parallel(
    parent: &Entity,
    tick: u64,
    child_genotype: crate::model::state::entity::Genotype,
) -> Entity {
    let mut rng = rand::thread_rng();
    let investment = parent.intel.genotype.reproductive_investment as f64;
    let child_energy = parent.metabolism.energy * investment;
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
            trophic_potential: child_genotype.trophic_potential,
            energy: child_energy,
            prev_energy: child_energy,
            max_energy: child_genotype.max_energy,
            peak_energy: child_energy,
            birth_tick: tick,
            generation: parent.metabolism.generation + 1,
            offspring_count: 0,
            lineage_id: child_genotype.lineage_id,
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
            last_signal: 0.0,
            last_vocalization: 0.0,
            reputation: 1.0,
            rank: 0.5,
            bonded_to: None,
            last_inputs: [0.0; 16],
        },
    }
}

pub fn archive_if_legend(entity: &Entity, tick: u64, logger: &HistoryLogger) -> Option<Legend> {
    let lifespan = tick - entity.metabolism.birth_tick;
    if lifespan > 1000
        || entity.metabolism.offspring_count > 10
        || entity.metabolism.peak_energy > 300.0
    {
        let legend = Legend {
            id: entity.id,
            parent_id: entity.parent_id,
            lineage_id: entity.metabolism.lineage_id,
            birth_tick: entity.metabolism.birth_tick,
            death_tick: tick,
            lifespan,
            generation: entity.metabolism.generation,
            offspring_count: entity.metabolism.offspring_count,
            peak_energy: entity.metabolism.peak_energy,
            birth_timestamp: "".to_string(),
            death_timestamp: Utc::now().to_rfc3339(),
            genotype: entity.intel.genotype.clone(),
            color_rgb: (entity.physics.r, entity.physics.g, entity.physics.b),
        };
        let _ = logger.archive_legend(legend.clone());
        Some(legend)
    } else {
        None
    }
}

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

pub fn is_legend_worthy(entity: &Entity, tick: u64) -> bool {
    let lifespan = tick - entity.metabolism.birth_tick;
    lifespan > 1000
        || entity.metabolism.offspring_count > 10
        || entity.metabolism.peak_energy > 300.0
}

// DEPRECATED WRAPPERS
pub fn handle_predation(idx: usize, entities: &mut [Entity], ctx: &mut PredationContext) {
    let targets = ctx
        .spatial_hash
        .query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
    for t_idx in targets {
        let v_snap = &ctx.snapshots[t_idx];
        if v_snap.id != entities[idx].id
            && !ctx.killed_ids.contains(&v_snap.id)
            && !are_same_tribe(&entities[idx], &entities[t_idx])
        {
            let gain = v_snap.energy
                * entities[idx].metabolism.trophic_potential as f64
                * (1.0 - (ctx.pop_stats.biomass_c / 10000.0)).max(0.5);
            entities[idx].metabolism.energy =
                (entities[idx].metabolism.energy + gain).min(entities[idx].metabolism.max_energy);
            ctx.killed_ids.insert(v_snap.id);
            ctx.lineage_consumption
                .push((entities[idx].metabolism.lineage_id, gain));
            ctx.events.push(LiveEvent::Death {
                id: v_snap.id,
                age: ctx.tick - v_snap.birth_tick,
                offspring: v_snap.offspring_count,
                tick: ctx.tick,
                timestamp: Utc::now().to_rfc3339(),
                cause: "predation".to_string(),
            });
        }
    }
}
pub fn handle_sharing(_idx: usize, _entities: &mut [Entity], _ctx: &mut PredationContext) {}
pub fn handle_reproduction(
    _idx: usize,
    _entities: &mut [Entity],
    _killed_ids: &HashSet<Uuid>,
    _sh: &SpatialHash,
    _cfg: &AppConfig,
    _t: u64,
) -> Option<Entity> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_legend_worthy_by_lifespan() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.birth_tick = 0;
        assert!(is_legend_worthy(&entity, 1500));
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
}
