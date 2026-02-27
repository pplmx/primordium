use crate::brain::GenotypeLogic;
use crate::systems::intel;
use primordium_data::{AncestralTrait, Entity, Health, Intel, Metabolism, Physics, Specialization};
use rand::Rng;
use std::f64;
use uuid::Uuid;

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

    // Density-dependent birth rate limiting
    let carrying_capacity =
        input.ctx.config.world.width as f64 * input.ctx.config.world.height as f64;
    let population_density = input.ctx.population as f64 / carrying_capacity;
    let investment = if population_density > 0.1 {
        let overcapacity_factor = (population_density - 0.1) / 0.9;
        let penalty = overcapacity_factor * overcapacity_factor;
        investment * (1.0 - penalty.max(0.0) * 0.8)
    } else {
        investment
    };

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
    let stress_factor = (1.0 - (input.energy / input.genotype.max_energy)).max(0.0_f64) as f32;

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

    let _stress_factor = (1.0 - (p1.energy / p1.genotype.max_energy)).max(0.0_f64) as f32;

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
