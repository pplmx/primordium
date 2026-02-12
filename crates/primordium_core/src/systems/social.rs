use crate::brain::GenotypeLogic;
use crate::config::AppConfig;
use crate::history::{Legend, LiveEvent, PopulationStats};
use crate::pheromone::PheromoneGrid;
use crate::snapshot::InternalEntitySnapshot;
use crate::spatial_hash::SpatialHash;
use crate::systems::intel;
use chrono::Utc;
use primordium_data::{
    AncestralTrait, Entity, Health, Identity, Intel, Metabolism, Physics, Specialization,
};
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
    // logger removed
    pub tick: u64,
    pub energy_transfers: &'a mut Vec<(usize, f64)>,
    pub lineage_consumption: &'a mut Vec<(Uuid, f64)>,
}

pub fn archive_if_legend_components(
    identity: &Identity,
    metabolism: &Metabolism,
    intel: &Intel,
    physics: &Physics,
    tick: u64,
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
            genotype: (*intel.genotype).clone(),
            color_rgb: (physics.r, physics.g, physics.b),
        };
        Some(legend)
    } else {
        None
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

    // Age with decay: peaks at age_rank_normalization * 0.7, then slowly declines
    // This prevents "elder exploit" where old age alone guarantees high rank
    let peak_age = config.social.age_rank_normalization * 0.7;
    let age_score_raw = (age as f32 / config.social.age_rank_normalization).min(1.0);
    let age_score = if age as f32 > peak_age {
        // Apply bell-curve decay after peak age
        let excess = (age as f32 - peak_age) / (config.social.age_rank_normalization - peak_age);
        (1.0 - excess.powi(2)).max(0.3) * age_score_raw
    } else {
        age_score_raw
    };

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
    // Redesigned split condition: High rank + High crowding = Alpha-led migration
    // This ensures the fittest lead the new tribe, not the weakest
    if crowding > config.evolution.crowding_threshold
        && intel.rank > config.social.sharing_threshold * 0.6
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

pub fn handle_symbiosis_components(
    idx: usize,
    snapshots: &[InternalEntitySnapshot],
    outputs: [f32; 12],
    spatial_hash: &SpatialHash,
    config: &AppConfig,
) -> Option<Uuid> {
    if outputs[8] > 0.5 {
        let mut partner_id = None;
        let self_snap = &snapshots[idx];

        spatial_hash.query_callback(
            self_snap.x,
            self_snap.y,
            config.social.territorial_range,
            |t_idx| {
                if idx != t_idx && partner_id.is_none() {
                    let target_snap = &snapshots[t_idx];
                    let color_dist = (self_snap.r as i32 - target_snap.r as i32).abs()
                        + (self_snap.g as i32 - target_snap.g as i32).abs()
                        + (self_snap.b as i32 - target_snap.b as i32).abs();

                    if color_dist < config.social.tribe_color_threshold
                        && target_snap.status != primordium_data::EntityStatus::Bonded
                    {
                        partner_id = Some(target_snap.id);
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
    pub config: &'a crate::config::AppConfig,
    pub population: usize,
    pub traits: std::collections::HashSet<AncestralTrait>,
    pub is_radiation_storm: bool,
    pub rng: &'a mut R,
    pub ancestral_genotype: Option<&'a primordium_data::Genotype>,
}

pub struct AsexualReproductionContext<'a, R: Rng> {
    pub pos: &'a primordium_data::Position,
    pub energy: f64,
    pub generation: u32,
    pub genotype: &'a primordium_data::Genotype,
    pub specialization: Option<Specialization>,
    pub ctx: &'a mut ReproductionContext<'a, R>,
}

pub fn reproduce_asexual_parallel_components_decomposed<R: Rng>(
    input: AsexualReproductionContext<R>,
) -> (Entity, f32) {
    const MIN_PARENT_REMAINING: f64 = 20.0;
    const SAFE_INVESTMENT_CAP: f64 = 0.7;

    // Early exit: insufficient energy to reproduce safely
    if input.energy < MIN_PARENT_REMAINING {
        // Return a placeholder that will be filtered out by the caller
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: uuid::Uuid::new_v4(),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: input.pos.x,
                    y: input.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: input.pos.x,
                    home_y: input.pos.y,
                    x: input.pos.x,
                    y: input.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: input.genotype.sensing_range,
                    max_speed: input.genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: input.genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: input.genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: input.ctx.tick,
                    generation: input.generation + 1,
                    offspring_count: 0,
                    lineage_id: input.genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(input.genotype.clone()),
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
                    ancestral_traits: input.ctx.traits.clone(),
                },
            },
            0.0,
        );
    }

    let investment = (input.genotype.reproductive_investment as f64).min(SAFE_INVESTMENT_CAP);

    // Calculate safe investment that leaves parent with minimum energy
    let max_safe_invest =
        ((input.energy - MIN_PARENT_REMAINING) / input.energy).clamp(0.1, SAFE_INVESTMENT_CAP);
    let actual_investment = investment.min(max_safe_invest);

    let child_energy = input.energy * actual_investment;
    let parent_remaining = input.energy - child_energy;

    // Skip reproduction if parent would be left with critical energy
    if parent_remaining < MIN_PARENT_REMAINING {
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: uuid::Uuid::from_u128(input.ctx.rng.gen()),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: input.pos.x,
                    y: input.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: input.pos.x,
                    home_y: input.pos.y,
                    x: input.pos.x,
                    y: input.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: input.genotype.sensing_range,
                    max_speed: input.genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: input.genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: input.genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: input.ctx.tick,
                    generation: input.generation + 1,
                    offspring_count: 0,
                    lineage_id: input.genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(input.genotype.clone()),
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
                    ancestral_traits: input.ctx.traits.clone(),
                },
            },
            0.0,
        );
    }

    let investment = (input.genotype.reproductive_investment as f64).min(SAFE_INVESTMENT_CAP);

    // Calculate safe investment that leaves parent with minimum energy
    let max_safe_invest =
        ((input.energy - MIN_PARENT_REMAINING) / input.energy).clamp(0.1, SAFE_INVESTMENT_CAP);
    let actual_investment = investment.min(max_safe_invest);

    let child_energy = input.energy * actual_investment;
    let parent_remaining = input.energy - child_energy;

    // Skip reproduction if parent would be left with critical energy
    if parent_remaining < MIN_PARENT_REMAINING {
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: uuid::Uuid::new_v4(),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: input.pos.x,
                    y: input.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: input.pos.x,
                    home_y: input.pos.y,
                    x: input.pos.x,
                    y: input.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: input.genotype.sensing_range,
                    max_speed: input.genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: input.genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: input.genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: input.ctx.tick,
                    generation: input.generation + 1,
                    offspring_count: 0,
                    lineage_id: input.genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(input.genotype.clone()),
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
                    ancestral_traits: input.ctx.traits.clone(),
                },
            },
            0.0,
        );
    }

    let mut child_genotype = std::sync::Arc::new(input.genotype.clone());
    let stress_factor = (1.0 - (input.energy / input.genotype.max_energy)).max(0.0) as f32;

    intel::mutate_genotype(
        &mut child_genotype,
        &intel::MutationParams {
            config: input.ctx.config,
            population: input.ctx.population,
            is_radiation_storm: input.ctx.is_radiation_storm,
            specialization: input.specialization,
            ancestral_genotype: input.ctx.ancestral_genotype,
            stress_factor,
        },
        input.ctx.rng,
    );
    let dist = input.genotype.distance(&child_genotype);
    if dist > input.ctx.config.evolution.speciation_threshold {
        std::sync::Arc::make_mut(&mut child_genotype).lineage_id =
            Uuid::from_u128(input.ctx.rng.gen());
    }

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(input.ctx.rng.gen()),
            parent_id: None,
        },
        position: primordium_data::Position {
            x: input.pos.x,
            y: input.pos.y,
        },
        velocity: primordium_data::Velocity::default(),
        appearance: primordium_data::Appearance {
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
        },
        physics: Physics {
            home_x: input.pos.x,
            home_y: input.pos.y,
            x: input.pos.x,
            y: input.pos.y,
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
            birth_tick: input.ctx.tick,
            generation: input.generation + 1,
            offspring_count: 0,
            lineage_id: child_genotype.lineage_id,
            has_metamorphosed: false,
            is_in_transit: false,
            migration_id: None,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: (input.energy / 200.0) as f32,
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
            ancestral_traits: input.ctx.traits.clone(),
        },
    };

    for trait_item in &input.ctx.traits {
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
    (baby, dist)
}

pub struct ParentData<'a> {
    pub pos: &'a primordium_data::Position,
    pub energy: f64,
    pub generation: u32,
    pub genotype: &'a primordium_data::Genotype,
}

pub fn reproduce_sexual_parallel_components_decomposed<R: Rng>(
    p1: &ParentData<'_>,
    ctx: &mut ReproductionContext<R>,
) -> (Entity, f32) {
    const MIN_PARENT_REMAINING: f64 = 20.0;
    const SAFE_INVESTMENT_CAP: f64 = 0.7;

    // Early exit: insufficient energy to reproduce safely
    if p1.energy < MIN_PARENT_REMAINING {
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: uuid::Uuid::from_u128(ctx.rng.gen()),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: p1.pos.x,
                    y: p1.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: p1.pos.x,
                    home_y: p1.pos.y,
                    x: p1.pos.x,
                    y: p1.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: p1.genotype.sensing_range,
                    max_speed: p1.genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: p1.genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: p1.genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: ctx.tick,
                    generation: p1.generation + 1,
                    offspring_count: 0,
                    lineage_id: p1.genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(p1.genotype.clone()),
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
            },
            0.0,
        );
    }

    let investment = (p1.genotype.reproductive_investment as f64).min(SAFE_INVESTMENT_CAP);

    // Calculate safe investment that leaves parent with minimum energy
    let max_safe_invest =
        ((p1.energy - MIN_PARENT_REMAINING) / p1.energy).clamp(0.1, SAFE_INVESTMENT_CAP);
    let actual_investment = investment.min(max_safe_invest);

    let child_energy = p1.energy * actual_investment;
    let parent_remaining = p1.energy - child_energy;

    // Skip reproduction if parent would be left with critical energy
    if parent_remaining < MIN_PARENT_REMAINING {
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: uuid::Uuid::from_u128(ctx.rng.gen()),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: p1.pos.x,
                    y: p1.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: p1.pos.x,
                    home_y: p1.pos.y,
                    x: p1.pos.x,
                    y: p1.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: p1.genotype.sensing_range,
                    max_speed: p1.genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: p1.genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: p1.genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: ctx.tick,
                    generation: p1.generation + 1,
                    offspring_count: 0,
                    lineage_id: p1.genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(p1.genotype.clone()),
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
            },
            0.0,
        );
    }

    let mut child_genotype = std::sync::Arc::new(p1.genotype.clone());

    // Apply mutations
    intel::mutate_genotype(
        &mut child_genotype,
        &intel::MutationParams {
            config: ctx.config,
            population: ctx.population,
            is_radiation_storm: ctx.is_radiation_storm,
            specialization: None,
            ancestral_genotype: ctx.ancestral_genotype,
            stress_factor: 0.0,
        },
        ctx.rng,
    );

    let dist = p1.genotype.distance(&child_genotype);
    if dist > ctx.config.evolution.speciation_threshold {
        std::sync::Arc::make_mut(&mut child_genotype).lineage_id = Uuid::from_u128(ctx.rng.gen());
    }

    let _stress_factor = (1.0 - (p1.energy / p1.genotype.max_energy)).max(0.0) as f32;

    let child_energy =
        p1.energy * (p1.genotype.reproductive_investment as f64).min(SAFE_INVESTMENT_CAP);
    let parent_remaining = p1.energy - child_energy;

    if parent_remaining < MIN_PARENT_REMAINING || child_energy <= 0.0 {
        return (
            Entity {
                identity: primordium_data::Identity {
                    id: Uuid::from_u128(ctx.rng.gen()),
                    parent_id: None,
                },
                position: primordium_data::Position {
                    x: p1.pos.x,
                    y: p1.pos.y,
                },
                velocity: primordium_data::Velocity::default(),
                appearance: primordium_data::Appearance::default(),
                physics: primordium_data::Physics {
                    home_x: p1.pos.x,
                    home_y: p1.pos.y,
                    x: p1.pos.x,
                    y: p1.pos.y,
                    vx: 0.0,
                    vy: 0.0,
                    r: 100,
                    g: 200,
                    b: 100,
                    symbol: '●',
                    sensing_range: child_genotype.sensing_range,
                    max_speed: child_genotype.max_speed,
                },
                metabolism: primordium_data::Metabolism {
                    trophic_potential: child_genotype.trophic_potential,
                    energy: 0.0,
                    prev_energy: 0.0,
                    max_energy: child_genotype.max_energy,
                    peak_energy: 0.0,
                    birth_tick: ctx.tick,
                    generation: p1.generation + 1,
                    offspring_count: 0,
                    lineage_id: child_genotype.lineage_id,
                    has_metamorphosed: false,
                    is_in_transit: false,
                    migration_id: None,
                },
                health: primordium_data::Health {
                    pathogen: None,
                    infection_timer: 0,
                    immunity: 0.0,
                },
                intel: primordium_data::Intel {
                    genotype: std::sync::Arc::new(p1.genotype.clone()),
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
            },
            dist,
        );
    }

    let mut baby = Entity {
        identity: primordium_data::Identity {
            id: Uuid::from_u128(ctx.rng.gen()),
            parent_id: None,
        },
        position: primordium_data::Position {
            x: p1.pos.x,
            y: p1.pos.y,
        },
        velocity: primordium_data::Velocity::default(),
        appearance: primordium_data::Appearance {
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
        },
        physics: Physics {
            home_x: p1.pos.x,
            home_y: p1.pos.y,
            x: p1.pos.x,
            y: p1.pos.y,
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
            generation: p1.generation + 1,
            offspring_count: 0,
            lineage_id: child_genotype.lineage_id,
            has_metamorphosed: false,
            is_in_transit: false,
            migration_id: None,
        },
        health: Health {
            pathogen: None,
            infection_timer: 0,
            immunity: (p1.energy / 200.0) as f32,
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

    (baby, dist)
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

pub fn is_legend_worthy_components(metabolism: &Metabolism, tick: u64) -> bool {
    let lifespan = tick - metabolism.birth_tick;
    lifespan > 1000 || metabolism.offspring_count > 10 || metabolism.peak_energy > 300.0
}
