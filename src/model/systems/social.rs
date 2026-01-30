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

pub fn are_same_tribe_components(phys1: &Physics, phys2: &Physics, config: &AppConfig) -> bool {
    let dist = (phys1.r as i32 - phys2.r as i32).abs()
        + (phys1.g as i32 - phys2.g as i32).abs()
        + (phys1.b as i32 - phys2.b as i32).abs();
    dist < config.social.tribe_color_threshold
}

type EntityComponents<'a> = (
    hecs::Entity,
    (
        &'a primordium_data::Identity,
        &'a primordium_data::Position,
        &'a primordium_data::Velocity,
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
        let self_pos = components[idx].1 .1;
        let self_phys = components[idx].1 .3;

        spatial_hash.query_callback(
            self_pos.x,
            self_pos.y,
            config.social.territorial_range,
            |t_idx| {
                if idx != t_idx && partner_id.is_none() {
                    let target_identity = components[t_idx].1 .0;
                    let target_phys = components[t_idx].1 .3;
                    let target_intel = &components[t_idx].1 .5;
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

pub struct ReproductionContext<'a, R: Rng> {
    pub tick: u64,
    pub config: &'a crate::model::config::AppConfig,
    pub population: usize,
    pub traits: std::collections::HashSet<AncestralTrait>,
    pub is_radiation_storm: bool,
    pub rng: &'a mut R,
    pub ancestral_genotype: Option<&'a primordium_data::Genotype>,
}

pub fn reproduce_asexual_parallel_components_decomposed<R: Rng>(
    pos: &primordium_data::Position,
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

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(ctx.rng.gen()),
            name: String::new(),
            parent_id: None,
        },
        position: primordium_data::Position { x: pos.x, y: pos.y },
        velocity: primordium_data::Velocity::default(),
        appearance: primordium_data::Appearance {
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
        },
        physics: Physics {
            home_x: pos.x,
            home_y: pos.y,
            x: pos.x,
            y: pos.y,
            vx: 0.0,
            vy: 0.0,
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
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

pub fn reproduce_sexual_parallel_components_decomposed<R: Rng>(
    p1_pos: &primordium_data::Position,
    p1_met: &Metabolism,
    p1_intel: &Intel,
    _p2_pos: &primordium_data::Position,
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

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(ctx.rng.gen()),
            name: String::new(),
            parent_id: None,
        },
        position: primordium_data::Position {
            x: p1_pos.x,
            y: p1_pos.y,
        },
        velocity: primordium_data::Velocity::default(),
        appearance: primordium_data::Appearance::default(), // Will be updated by update_name or similar
        physics: Physics {
            x: p1_pos.x,
            y: p1_pos.y,
            vx: 0.0,
            vy: 0.0,
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
            home_x: p1_pos.x,
            home_y: p1_pos.y,
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
            immunity: (p1_met.energy + p2_met.energy) as f32 / 200.0,
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

pub fn archive_if_legend_components(
    identity: &primordium_data::Identity,
    metabolism: &Metabolism,
    intel: &Intel,
    physics: &Physics,
    tick: u64,
    logger: &HistoryLogger,
) -> Option<Legend> {
    let lifespan = tick - metabolism.birth_tick;
    if lifespan > 1000 || metabolism.offspring_count > 10 || metabolism.peak_energy > 300.0 {
        let legend = Legend {
            id: identity.id,
            parent_id: identity.parent_id,
            lineage_id: metabolism.lineage_id,
            birth_tick: metabolism.birth_tick,
            death_tick: tick,
            lifespan,
            generation: metabolism.generation,
            offspring_count: metabolism.offspring_count,
            peak_energy: metabolism.peak_energy,
            birth_timestamp: "".to_string(),
            death_timestamp: Utc::now().to_rfc3339(),
            genotype: intel.genotype.clone(),
            color_rgb: (physics.r, physics.g, physics.b),
        };
        let _ = logger.archive_legend(legend.clone());
        Some(legend)
    } else {
        None
    }
}

pub fn is_legend_worthy_components(metabolism: &Metabolism, tick: u64) -> bool {
    let lifespan = tick - metabolism.birth_tick;
    lifespan > 1000 || metabolism.offspring_count > 10 || metabolism.peak_energy > 300.0
}
