//! Biological system - handles infection, pathogen emergence, and death processing.

use crate::model::config::AppConfig;
use crate::model::history::{LiveEvent, PopulationStats};
use crate::model::spatial_hash::SpatialHash;
use crate::model::systems::social;
use primordium_data::{Entity, Health, Intel, Metabolism, Pathogen, Physics, Specialization};
use rand::Rng;
use std::collections::HashSet;

pub fn biological_system_components<R: Rng>(
    metabolism: &mut Metabolism,
    intel: &mut Intel,
    health: &mut Health,
    _physics: &Physics,
    population_count: usize,
    config: &AppConfig,
    rng: &mut R,
) {
    process_infection_components(health, metabolism);

    if intel.reputation < 1.0 {
        intel.reputation = (intel.reputation + 0.001).min(1.0);
    }

    if population_count > 0
        && population_count < 10
        && rng.gen_bool(config.evolution.drift_rate as f64)
    {
        match rng.gen_range(0..4) {
            0 => intel.genotype.metabolic_niche = rng.gen_range(0.0..1.0),
            1 => intel.genotype.max_speed = rng.gen_range(0.5..1.5),
            2 => intel.genotype.sensing_range = rng.gen_range(5.0..15.0),
            _ => intel.genotype.max_energy = rng.gen_range(50.0..150.0),
        }
    }

    if intel.specialization.is_none() {
        let progress = 0.01;
        if intel.last_aggression > 0.8 {
            social::increment_spec_meter_components(
                intel,
                Specialization::Soldier,
                progress,
                config,
            );
        }
        if intel.last_share_intent > 0.8 {
            social::increment_spec_meter_components(
                intel,
                Specialization::Provider,
                progress,
                config,
            );
        }
    }

    let brain_maintenance = (intel.genotype.brain.nodes.len() as f64
        * config.brain.hidden_node_cost)
        + (intel.genotype.brain.connections.len() as f64 * config.brain.connection_cost);

    metabolism.energy -= brain_maintenance;
}

pub fn biological_system<R: Rng>(
    entity: &mut Entity,
    population_count: usize,
    config: &AppConfig,
    rng: &mut R,
) {
    biological_system_components(
        &mut entity.metabolism,
        &mut entity.intel,
        &mut entity.health,
        &entity.physics,
        population_count,
        config,
        rng,
    );
}

pub fn try_infect_components<R: Rng>(
    health: &mut Health,
    pathogen: &Pathogen,
    rng: &mut R,
) -> bool {
    if health.pathogen.is_some() {
        return false;
    }
    let chance = (pathogen.virulence - health.immunity).max(0.01);
    if rng.gen::<f32>() < chance {
        health.pathogen = Some(pathogen.clone());
        health.infection_timer = pathogen.duration;
        return true;
    }
    false
}

pub fn try_infect<R: Rng>(entity: &mut Entity, pathogen: &Pathogen, rng: &mut R) -> bool {
    try_infect_components(&mut entity.health, pathogen, rng)
}

pub fn process_infection_components(health: &mut Health, metabolism: &mut Metabolism) {
    if let Some(p) = &health.pathogen {
        metabolism.energy -= p.lethality as f64;
        if health.infection_timer > 0 {
            health.infection_timer -= 1;
        } else {
            health.pathogen = None;
            health.immunity = (health.immunity + 0.1).min(1.0);
        }
    }
}

pub fn process_infection(entity: &mut Entity) {
    process_infection_components(&mut entity.health, &mut entity.metabolism);
}

pub fn handle_pathogen_emergence(active_pathogens: &mut Vec<Pathogen>, _rng: &mut impl Rng) {
    if rand::thread_rng().gen_bool(0.0001) {
        active_pathogens.push(crate::model::pathogen::create_random_pathogen());
    }
}

pub fn handle_infection_ecs<R: Rng>(
    handle: hecs::Entity,
    world: &mut hecs::World,
    entity_handles: &[hecs::Entity],
    killed_ids: &HashSet<uuid::Uuid>,
    active_pathogens: &[Pathogen],
    spatial_hash: &SpatialHash,
    rng: &mut R,
) {
    let mut self_id = None;
    let mut self_phys_x = 0.0;
    let mut self_phys_y = 0.0;
    let mut pathogen_to_spread = None;

    if let (Ok(mut health), Ok(mut metabolism), Ok(physics), Ok(identity)) = (
        world.get::<&mut Health>(handle),
        world.get::<&mut Metabolism>(handle),
        world.get::<&Physics>(handle),
        world.get::<&primordium_data::Identity>(handle),
    ) {
        process_infection_components(&mut health, &mut metabolism);
        for p in active_pathogens {
            if rng.gen_bool(0.005) {
                try_infect_components(&mut health, p, rng);
            }
        }

        if let Some(p) = &health.pathogen {
            self_id = Some(identity.id);
            self_phys_x = physics.x;
            self_phys_y = physics.y;
            pathogen_to_spread = Some(p.clone());
        }
    }

    if let (Some(self_id), Some(p)) = (self_id, pathogen_to_spread) {
        spatial_hash.query_callback(self_phys_x, self_phys_y, 2.0, |n_idx| {
            let n_handle = entity_handles[n_idx];
            if let (Ok(n_identity), Ok(mut n_health)) = (
                world.get::<&primordium_data::Identity>(n_handle),
                world.get::<&mut Health>(n_handle),
            ) {
                if n_identity.id != self_id
                    && !killed_ids.contains(&n_identity.id)
                    && try_infect_components(&mut n_health, &p, rng)
                {}
            }
        });
    }
}

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
        spatial_hash.query_callback(
            entities[idx].physics.x,
            entities[idx].physics.y,
            2.0,
            |n_idx| {
                if n_idx != idx
                    && !killed_ids.contains(&entities[n_idx].identity.id)
                    && try_infect(&mut entities[n_idx], &p, rng)
                {}
            },
        );
    }
}

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
        id: entities[idx].identity.id,
        age,
        offspring: entities[idx].metabolism.offspring_count,
        tick,
        timestamp: chrono::Utc::now().to_rfc3339(),
        cause: cause.to_string(),
    };
    let _ = logger.log_event(ev.clone());
    events.push(ev);
}
