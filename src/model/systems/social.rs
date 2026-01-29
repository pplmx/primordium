//! Social system - handles predation, reproduction, and symbiotic relationships.

use crate::model::brain::GenotypeLogic;
use crate::model::config::AppConfig;
use crate::model::history::{HistoryLogger, Legend, LiveEvent, PopulationStats};
use crate::model::pheromone::PheromoneGrid;
use crate::model::spatial_hash::SpatialHash;
use crate::model::systems::intel;
use crate::model::world::InternalEntitySnapshot;
use chrono::Utc;
use primordium_data::{AncestralTrait, Entity, Health, Intel, Metabolism, Physics, Specialization};
use rand::Rng;
use std::collections::HashSet;
use uuid::Uuid;

pub struct PredationContext<'a> {
    pub snapshots: &'a [InternalEntitySnapshot],
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

pub fn soldier_territorial_bonus(entity: &Entity, config: &AppConfig) -> f64 {
    let dist = ((entity.physics.x - entity.physics.home_x).powi(2)
        + (entity.physics.y - entity.physics.home_y).powi(2))
    .sqrt();
    if dist < config.social.territorial_range {
        config.social.soldier_damage_mult
    } else {
        1.0
    }
}

pub fn calculate_social_rank_components(
    metabolism: &Metabolism,
    intel: &Intel,
    tick: u64,
    config: &AppConfig,
) -> f32 {
    let energy_score = (metabolism.energy / metabolism.max_energy).clamp(0.0, 1.0) as f32;
    let age = tick - metabolism.birth_tick;
    let age_score = (age as f32 / config.social.age_rank_normalization).min(1.0);
    let offspring_score =
        (metabolism.offspring_count as f32 / config.social.offspring_rank_normalization).min(1.0);
    let rep_score = intel.reputation.clamp(0.0, 1.0);

    let w = config.social.rank_weights;
    w[0] * energy_score + w[1] * age_score + w[2] * offspring_score + w[3] * rep_score
}

pub fn calculate_social_rank(entity: &Entity, tick: u64, config: &AppConfig) -> f32 {
    calculate_social_rank_components(&entity.metabolism, &entity.intel, tick, config)
}

pub fn start_tribal_split_components<R: Rng>(
    _phys: &Physics,
    _met: &Metabolism,
    intel: &Intel,
    crowding: f32,
    config: &AppConfig,
    rng: &mut R,
) -> Option<(u8, u8, u8)> {
    if crowding > config.evolution.crowding_threshold
        && intel.rank < config.social.sharing_threshold * 0.4
    {
        Some((
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        ))
    } else {
        None
    }
}

pub fn start_tribal_split<R: Rng>(
    entity: &Entity,
    crowding: f32,
    config: &AppConfig,
    rng: &mut R,
) -> Option<(u8, u8, u8)> {
    start_tribal_split_components(
        &entity.physics,
        &entity.metabolism,
        &entity.intel,
        crowding,
        config,
        rng,
    )
}

pub fn are_same_tribe_components(phys1: &Physics, phys2: &Physics, config: &AppConfig) -> bool {
    let dist = (phys1.r as i32 - phys2.r as i32).abs()
        + (phys1.g as i32 - phys2.g as i32).abs()
        + (phys1.b as i32 - phys2.b as i32).abs();
    dist < config.social.tribe_color_threshold
}

pub fn are_same_tribe(e1: &Entity, e2: &Entity, config: &AppConfig) -> bool {
    are_same_tribe_components(&e1.physics, &e2.physics, config)
}

pub fn can_share(entity: &Entity, config: &AppConfig) -> bool {
    entity.metabolism.energy > entity.metabolism.max_energy * config.social.sharing_threshold as f64
}

type EntityComponents<'a> = (
    hecs::Entity,
    (
        &'a primordium_data::Identity,
        &'a Physics,
        &'a Metabolism,
        &'a mut Intel,
        &'a Health,
    ),
);

pub fn handle_symbiosis_components(
    idx: usize,
    components: &[EntityComponents],
    outputs: [f32; 12],
    spatial_hash: &SpatialHash,
    config: &AppConfig,
) -> Option<Uuid> {
    if outputs[8] > 0.5 {
        let mut partner_id = None;
        let self_phys = components[idx].1 .1;

        spatial_hash.query_callback(
            self_phys.x,
            self_phys.y,
            config.social.territorial_range,
            |t_idx| {
                if idx != t_idx && partner_id.is_none() {
                    let target_identity = components[t_idx].1 .0;
                    let target_phys = components[t_idx].1 .1;
                    let target_intel = &components[t_idx].1 .3;
                    if target_intel.bonded_to.is_none()
                        && are_same_tribe_components(self_phys, target_phys, config)
                    {
                        partner_id = Some(target_identity.id);
                    }
                }
            },
        );
        partner_id
    } else {
        None
    }
}

pub fn handle_symbiosis(
    idx: usize,
    entities: &[Entity],
    outputs: [f32; 12],
    spatial_hash: &SpatialHash,
    config: &AppConfig,
) -> Option<Uuid> {
    if outputs[8] > 0.5 {
        let mut partner_id = None;
        spatial_hash.query_callback(
            entities[idx].physics.x,
            entities[idx].physics.y,
            config.social.territorial_range,
            |t_idx| {
                if idx != t_idx && partner_id.is_none() {
                    let target = &entities[t_idx];
                    if target.intel.bonded_to.is_none()
                        && are_same_tribe(&entities[idx], target, config)
                    {
                        partner_id = Some(target.identity.id);
                    }
                }
            },
        );
        partner_id
    } else {
        None
    }
}

pub struct ReproductionContext<'a, R: Rng> {
    pub tick: u64,
    pub config: &'a crate::model::config::AppConfig,
    pub population: usize,
    pub traits: std::collections::HashSet<AncestralTrait>,
    pub is_radiation_storm: bool,
    pub rng: &'a mut R,
    pub ancestral_genotype: Option<&'a primordium_data::Genotype>,
}

pub fn reproduce_asexual_parallel_components<R: Rng>(
    phys: &Physics,
    met: &Metabolism,
    intel: &Intel,
    ctx: &mut ReproductionContext<R>,
) -> (Entity, f32) {
    let investment = intel.genotype.reproductive_investment as f64;
    let child_energy = met.energy * investment;

    let mut child_genotype = intel.genotype.clone();
    intel::mutate_genotype(
        &mut child_genotype,
        ctx.config,
        ctx.population,
        ctx.is_radiation_storm,
        intel.specialization,
        ctx.rng,
        ctx.ancestral_genotype,
    );
    let dist = intel.genotype.distance(&child_genotype);
    if dist > ctx.config.evolution.speciation_threshold {
        child_genotype.lineage_id = Uuid::from_u128(ctx.rng.gen());
    }

    let r = (phys.r as i16 + ctx.rng.gen_range(-15..=15)).clamp(0, 255) as u8;
    let g = (phys.g as i16 + ctx.rng.gen_range(-15..=15)).clamp(0, 255) as u8;
    let b = (phys.b as i16 + ctx.rng.gen_range(-15..=15)).clamp(0, 255) as u8;

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(ctx.rng.gen()),
            name: String::new(),
            parent_id: None,
        },
        physics: Physics {
            x: phys.x,
            y: phys.y,
            vx: phys.vx,
            vy: phys.vy,
            r,
            g,
            b,
            symbol: '●',
            home_x: phys.x,
            home_y: phys.y,
            sensing_range: child_genotype.sensing_range,
            max_speed: child_genotype.max_speed,
        },
        metabolism: Metabolism {
            trophic_potential: child_genotype.trophic_potential,
            energy: child_energy,
            prev_energy: child_energy,
            max_energy: child_genotype.max_energy,
            peak_energy: child_energy,
            birth_tick: ctx.tick,
            generation: met.generation + 1,
            offspring_count: 0,
            lineage_id: child_genotype.lineage_id,
            has_metamorphosed: false,
            is_in_transit: false,
            migration_id: None,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: 0.0,
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
            last_inputs: [0.0; 29],
            last_activations: primordium_data::Activations::default(),
            specialization: None,
            spec_meters: std::collections::HashMap::new(),
            ancestral_traits: ctx.traits.clone(),
        },
    };

    for trait_item in &ctx.traits {
        match trait_item {
            AncestralTrait::AcuteSenses => {
                baby.physics.sensing_range *= 1.2;
            }
            AncestralTrait::SwiftMovement => {
                baby.physics.max_speed *= 1.1;
            }
            _ => {}
        }
    }

    baby.update_name();
    (baby, dist)
}

pub fn reproduce_asexual_parallel<R: Rng>(
    parent: &Entity,
    ctx: &mut ReproductionContext<R>,
) -> (Entity, f32) {
    let (mut baby, dist) = reproduce_asexual_parallel_components(
        &parent.physics,
        &parent.metabolism,
        &parent.intel,
        ctx,
    );
    baby.identity.parent_id = Some(parent.identity.id);
    (baby, dist)
}

pub fn reproduce_sexual_parallel_components<R: Rng>(
    p1_phys: &Physics,
    p1_met: &Metabolism,
    p1_intel: &Intel,
    p2_phys: &Physics,
    p2_met: &Metabolism,
    p2_intel: &Intel,
    ctx: &mut ReproductionContext<R>,
) -> (Entity, f32) {
    let investment = p1_intel.genotype.reproductive_investment as f64;
    let child_energy = (p1_met.energy + p2_met.energy) * investment / 2.0;

    let mut child_genotype = p1_intel
        .genotype
        .crossover_with_rng(&p2_intel.genotype, ctx.rng);
    intel::mutate_genotype(
        &mut child_genotype,
        ctx.config,
        100,
        ctx.is_radiation_storm,
        None,
        ctx.rng,
        ctx.ancestral_genotype,
    );

    let r_mut = ((p1_phys.r as i16 + p2_phys.r as i16) / 2 + ctx.rng.gen_range(-10..=10))
        .clamp(0, 255) as u8;
    let g_mut = ((p1_phys.g as i16 + p2_phys.g as i16) / 2 + ctx.rng.gen_range(-10..=10))
        .clamp(0, 255) as u8;
    let b_mut = ((p1_phys.b as i16 + p2_phys.b as i16) / 2 + ctx.rng.gen_range(-10..=10))
        .clamp(0, 255) as u8;

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(ctx.rng.gen()),
            name: String::new(),
            parent_id: None,
        },
        physics: Physics {
            x: p1_phys.x,
            y: p1_phys.y,
            vx: p1_phys.vx,
            vy: p1_phys.vy,
            r: r_mut,
            g: g_mut,
            b: b_mut,
            symbol: '●',
            home_x: p1_phys.x,
            home_y: p1_phys.y,
            sensing_range: child_genotype.sensing_range,
            max_speed: child_genotype.max_speed,
        },
        metabolism: Metabolism {
            trophic_potential: child_genotype.trophic_potential,
            energy: child_energy,
            prev_energy: child_energy,
            max_energy: child_genotype.max_energy,
            peak_energy: child_energy,
            birth_tick: ctx.tick,
            generation: p1_met.generation.max(p2_met.generation) + 1,
            offspring_count: 0,
            lineage_id: child_genotype.lineage_id,
            has_metamorphosed: false,
            is_in_transit: false,
            migration_id: None,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: (p1_met.energy + p2_met.energy) as f32 / 200.0, // Placeholder
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
            last_inputs: [0.0; 29],
            last_activations: primordium_data::Activations::default(),
            specialization: None,
            spec_meters: std::collections::HashMap::new(),
            ancestral_traits: ctx.traits.clone(),
        },
    };

    for trait_item in &ctx.traits {
        match trait_item {
            AncestralTrait::AcuteSenses => {
                baby.physics.sensing_range *= 1.2;
            }
            AncestralTrait::SwiftMovement => {
                baby.physics.max_speed *= 1.1;
            }
            _ => {}
        }
    }

    baby.update_name();
    (baby, 0.1)
}

pub fn reproduce_asexual(
    parent: &mut Entity,
    tick: u64,
    config: &AppConfig,
    population: usize,
    traits: std::collections::HashSet<AncestralTrait>,
    is_radiation_storm: bool,
) -> Entity {
    let mut rng = rand::thread_rng();
    let mut ctx = ReproductionContext {
        tick,
        config,
        population,
        traits,
        is_radiation_storm,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (baby, _) = reproduce_asexual_parallel(parent, &mut ctx);
    let investment = parent.intel.genotype.reproductive_investment as f64;
    parent.metabolism.energy *= 1.0 - investment;
    parent.metabolism.offspring_count += 1;
    baby
}

pub fn increment_spec_meter_components(
    intel: &mut Intel,
    spec: Specialization,
    amount: f32,
    config: &AppConfig,
) {
    if intel.specialization.is_none() {
        let bias_idx = match spec {
            Specialization::Soldier => 0,
            Specialization::Engineer => 1,
            Specialization::Provider => 2,
        };
        let meter = intel.spec_meters.entry(spec).or_insert(0.0);
        *meter += amount * (1.0 + intel.genotype.specialization_bias[bias_idx]);
        if *meter >= config.social.specialization_threshold {
            intel.specialization = Some(spec);
        }
    }
}

pub fn increment_spec_meter(
    entity: &mut Entity,
    spec: Specialization,
    amount: f32,
    config: &AppConfig,
) {
    increment_spec_meter_components(&mut entity.intel, spec, amount, config);
}

pub fn archive_if_legend(entity: &Entity, tick: u64, logger: &HistoryLogger) -> Option<Legend> {
    let lifespan = tick - entity.metabolism.birth_tick;
    if lifespan > 1000
        || entity.metabolism.offspring_count > 10
        || entity.metabolism.peak_energy > 300.0
    {
        let legend = Legend {
            id: entity.identity.id,
            parent_id: entity.identity.parent_id,
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

pub fn handle_predation(idx: usize, entities: &mut [Entity], ctx: &mut PredationContext) {
    ctx.spatial_hash.query_callback(
        entities[idx].physics.x,
        entities[idx].physics.y,
        1.5,
        |t_idx| {
            let v_snap = &ctx.snapshots[t_idx];
            if v_snap.id != entities[idx].identity.id
                && !ctx.killed_ids.contains(&v_snap.id)
                && !are_same_tribe(&entities[idx], &entities[t_idx], ctx.config)
            {
                let gain = v_snap.energy
                    * entities[idx].metabolism.trophic_potential as f64
                    * ctx.config.ecosystem.predation_energy_gain_fraction
                    * (1.0
                        - (ctx.pop_stats.biomass_c
                            / ctx.config.ecosystem.predation_competition_scale))
                        .max(ctx.config.ecosystem.predation_min_efficiency);
                entities[idx].metabolism.energy = (entities[idx].metabolism.energy + gain)
                    .min(entities[idx].metabolism.max_energy);
                ctx.killed_ids.insert(v_snap.id);
                ctx.lineage_consumption
                    .push((entities[idx].metabolism.lineage_id, gain));
                ctx.events.push(LiveEvent::Death {
                    id: v_snap.id,
                    age: ctx.tick - v_snap.birth_tick,
                    offspring: v_snap.offspring_count,
                    tick: ctx.tick,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    cause: "predation".to_string(),
                });
            }
        },
    );
}
