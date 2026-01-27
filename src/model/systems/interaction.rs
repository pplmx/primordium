use crate::model::config::AppConfig;
use crate::model::history::{FossilRegistry, HistoryLogger, LiveEvent, PopulationStats};
use crate::model::state::entity::{Entity, EntityStatus, Specialization};
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::interaction::InteractionCommand;
use crate::model::state::lineage_registry::LineageRegistry;
use crate::model::state::terrain::{TerrainGrid, TerrainType};
use crate::model::systems::social;
use chrono::Utc;
use std::collections::HashSet;
use uuid::Uuid;

pub struct InteractionContext<'a> {
    pub terrain: &'a mut TerrainGrid,
    pub env: &'a mut Environment,
    pub pop_stats: &'a mut PopulationStats,
    pub lineage_registry: &'a mut LineageRegistry,
    pub fossil_registry: &'a mut FossilRegistry,
    pub logger: &'a mut HistoryLogger,
    pub config: &'a AppConfig,
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub social_grid: &'a mut [u8],
    pub lineage_consumption: &'a mut Vec<(Uuid, f64)>,
    pub food: &'a mut Vec<Food>,
}

pub struct InteractionResult {
    pub events: Vec<LiveEvent>,
    pub killed_ids: HashSet<uuid::Uuid>,
    pub eaten_food_indices: HashSet<usize>,
    pub new_babies: Vec<Entity>,
}

pub fn process_interaction_commands(
    entities: &mut [Entity],
    commands: Vec<InteractionCommand>,
    ctx: &mut InteractionContext,
) -> InteractionResult {
    let mut events = Vec::new();
    let mut killed_ids = HashSet::new();
    let mut eaten_food_indices = HashSet::new();
    let mut new_babies = Vec::new();

    for cmd in commands {
        match cmd {
            InteractionCommand::Kill {
                target_idx,
                attacker_idx,
                attacker_lineage,
                cause,
            } => {
                let target = &entities[target_idx];
                let tid = target.id;
                if !killed_ids.contains(&tid) {
                    let mut multiplier = 1.0;
                    let attacker = &entities[attacker_idx];

                    // Phase 56: High-intensity activity oxygen cost
                    ctx.env
                        .consume_oxygen(ctx.config.ecosystem.oxygen_consumption_unit);

                    // Phase 49: Soldier damage bonus
                    let attacker_status = attacker.status(
                        ctx.config.brain.activation_threshold,
                        ctx.tick,
                        ctx.config.metabolism.maturity_age,
                    );
                    if attacker_status == EntityStatus::Soldier
                        || attacker.intel.specialization == Some(Specialization::Soldier)
                    {
                        multiplier *= ctx.config.social.soldier_damage_mult;
                    }

                    // Phase 49: War Zone bonus
                    let ix = (attacker.physics.x as usize).min(ctx.width as usize - 1);
                    let iy = (attacker.physics.y as usize).min(ctx.height as usize - 1);
                    if ctx.social_grid[iy * ctx.width as usize + ix] == 2 {
                        multiplier *= ctx.config.social.soldier_damage_mult;
                    }

                    let energy_gain = target.metabolism.energy
                        * ctx.config.ecosystem.predation_energy_gain_fraction
                        * multiplier;

                    killed_ids.insert(tid);
                    ctx.pop_stats
                        .record_death(ctx.tick - target.metabolism.birth_tick);

                    let ev = LiveEvent::Death {
                        id: tid,
                        age: ctx.tick - target.metabolism.birth_tick,
                        offspring: target.metabolism.offspring_count,
                        tick: ctx.tick,
                        timestamp: Utc::now().to_rfc3339(),
                        cause,
                    };
                    let _ = ctx.logger.log_event(ev.clone());
                    events.push(ev);

                    ctx.lineage_consumption
                        .push((attacker_lineage, energy_gain));

                    // Phase 60: Collective Reinforcement
                    ctx.lineage_registry
                        .boost_memory_value(&attacker_lineage, "goal", 0.5);
                    ctx.lineage_registry.boost_memory_value(&tid, "threat", 1.0);

                    let attacker_mut = &mut entities[attacker_idx];
                    attacker_mut.metabolism.energy = (attacker_mut.metabolism.energy + energy_gain)
                        .min(attacker_mut.metabolism.max_energy);
                }
            }
            InteractionCommand::Bond {
                target_idx,
                partner_id,
            } => {
                entities[target_idx].intel.bonded_to = Some(partner_id);
            }
            InteractionCommand::BondBreak { target_idx } => {
                entities[target_idx].intel.bonded_to = None;
            }
            InteractionCommand::Birth {
                parent_idx,
                mut baby,
                genetic_distance,
            } => {
                ctx.pop_stats.record_birth_distance(genetic_distance);
                ctx.lineage_registry.record_birth(
                    baby.metabolism.lineage_id,
                    baby.metabolism.generation,
                    ctx.tick,
                );
                let ev = LiveEvent::Birth {
                    id: baby.id,
                    parent_id: baby.parent_id,
                    gen: baby.metabolism.generation,
                    tick: ctx.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = ctx.logger.log_event(ev.clone());
                events.push(ev);

                // Phase 52: Nest Nursery Bonus
                let terrain_type = ctx.terrain.get(baby.physics.x, baby.physics.y).terrain_type;
                if matches!(terrain_type, TerrainType::Nest) {
                    baby.metabolism.energy *= ctx.config.metabolism.birth_energy_multiplier;
                    baby.metabolism.peak_energy = baby.metabolism.energy;
                }

                new_babies.push(*baby);

                let parent = &mut entities[parent_idx];
                let inv = parent.intel.genotype.reproductive_investment as f64;
                let c_e = parent.metabolism.energy * inv;
                parent.metabolism.energy -= c_e;
                parent.metabolism.offspring_count += 1;
            }
            InteractionCommand::EatFood {
                food_index,
                attacker_idx,
            } => {
                if !eaten_food_indices.contains(&food_index) {
                    eaten_food_indices.insert(food_index);
                    let e = &entities[attacker_idx];
                    let niche_eff = 1.0
                        - (e.intel.genotype.metabolic_niche - ctx.food[food_index].nutrient_type)
                            .abs();
                    let energy_gain = ctx.config.metabolism.food_value
                        * niche_eff as f64
                        * (1.0 - e.metabolism.trophic_potential) as f64;

                    let attacker_mut = &mut entities[attacker_idx];
                    attacker_mut.metabolism.energy = (attacker_mut.metabolism.energy + energy_gain)
                        .min(attacker_mut.metabolism.max_energy);

                    // Phase 60: Collective Reinforcement (Food abundance)
                    ctx.lineage_registry.boost_memory_value(
                        &attacker_mut.metabolism.lineage_id,
                        "goal",
                        0.2,
                    );

                    ctx.terrain.deplete(
                        attacker_mut.physics.x,
                        attacker_mut.physics.y,
                        ctx.config.ecosystem.soil_depletion_unit,
                    );
                }
            }
            InteractionCommand::TransferEnergy { target_idx, amount } => {
                let mut actual_amount = amount;
                if amount < 0.0 {
                    let sender = &entities[target_idx];
                    if sender.intel.specialization == Some(Specialization::Provider) {
                        actual_amount *= ctx.config.terraform.engineer_discount;
                    }
                }
                let target = &mut entities[target_idx];
                target.metabolism.energy = (target.metabolism.energy + actual_amount)
                    .clamp(0.0, target.metabolism.max_energy);
            }
            InteractionCommand::Infect {
                target_idx,
                pathogen,
            } => {
                let target = &mut entities[target_idx];
                target.health.pathogen = Some(pathogen.clone());
                target.health.infection_timer = pathogen.duration;
            }
            InteractionCommand::UpdateReputation { target_idx, delta } => {
                let target = &mut entities[target_idx];
                target.intel.reputation = (target.intel.reputation + delta).clamp(0.0, 1.0);
            }
            InteractionCommand::Fertilize { x, y, amount } => {
                ctx.terrain.fertilize(x, y, amount);
                ctx.terrain.add_biomass(x, y, amount * 10.0);
            }
            InteractionCommand::TribalTerritory { x, y, is_war } => {
                let ix = (x as usize).min(ctx.width as usize - 1);
                let iy = (y as usize).min(ctx.height as usize - 1);
                ctx.social_grid[iy * ctx.width as usize + ix] = if is_war { 2 } else { 1 };
            }
            InteractionCommand::Dig { x, y, attacker_idx } => {
                let cell = ctx.terrain.get(x, y);
                let attacker = &mut entities[attacker_idx];
                let mut energy_cost = ctx.config.terraform.dig_cost;

                ctx.env.consume_oxygen(ctx.config.terraform.dig_oxygen_cost);

                if attacker.intel.specialization == Some(Specialization::Engineer) {
                    energy_cost *= ctx.config.terraform.engineer_discount;
                }

                if matches!(cell.terrain_type, TerrainType::Wall | TerrainType::Mountain) {
                    if attacker.metabolism.energy > energy_cost {
                        attacker.metabolism.energy -= energy_cost;
                        ctx.terrain
                            .set_cell_type(x as u16, y as u16, TerrainType::Plains);
                        social::increment_spec_meter(
                            attacker,
                            Specialization::Engineer,
                            1.0,
                            ctx.config,
                        );
                    }
                } else if matches!(cell.terrain_type, TerrainType::Plains) {
                    let eff_hydro_cost =
                        if attacker.intel.specialization == Some(Specialization::Engineer) {
                            ctx.config.terraform.canal_cost * ctx.config.terraform.engineer_discount
                        } else {
                            ctx.config.terraform.canal_cost
                        };

                    if attacker.metabolism.energy > 50.0
                        && ctx
                            .terrain
                            .has_neighbor_type(x as u16, y as u16, TerrainType::River)
                    {
                        attacker.metabolism.energy -= eff_hydro_cost;
                        ctx.terrain
                            .set_cell_type(x as u16, y as u16, TerrainType::River);
                        social::increment_spec_meter(
                            attacker,
                            Specialization::Engineer,
                            2.0,
                            ctx.config,
                        );
                    }
                }
            }
            InteractionCommand::Build {
                x,
                y,
                attacker_idx,
                is_nest,
                is_outpost,
            } => {
                let cell = ctx.terrain.get(x, y);
                let attacker = &mut entities[attacker_idx];
                let mut energy_cost = if is_outpost {
                    ctx.config.terraform.nest_energy_req * 2.0
                } else {
                    ctx.config.terraform.build_cost
                };

                ctx.env
                    .consume_oxygen(ctx.config.terraform.build_oxygen_cost);

                if attacker.intel.specialization == Some(Specialization::Engineer) {
                    energy_cost *= ctx.config.terraform.engineer_discount;
                }

                if matches!(cell.terrain_type, TerrainType::Plains)
                    && attacker.metabolism.energy > energy_cost
                {
                    let new_type = if is_outpost
                        && attacker.metabolism.energy > ctx.config.terraform.nest_energy_req * 3.0
                    {
                        TerrainType::Outpost
                    } else if is_nest
                        && attacker.metabolism.energy > ctx.config.terraform.nest_energy_req
                    {
                        TerrainType::Nest
                    } else {
                        TerrainType::Wall
                    };
                    attacker.metabolism.energy -= energy_cost;
                    let idx = ctx.terrain.index(x as u16, y as u16);
                    ctx.terrain.set_cell_type(x as u16, y as u16, new_type);
                    if let Some(c) = ctx.terrain.cells.get_mut(idx) {
                        c.owner_id = Some(attacker.metabolism.lineage_id);
                    }

                    social::increment_spec_meter(
                        attacker,
                        Specialization::Engineer,
                        if is_outpost { 5.0 } else { 1.0 },
                        ctx.config,
                    );
                }
            }
            InteractionCommand::Metamorphosis { target_idx } => {
                let e = &mut entities[target_idx];
                e.metabolism.has_metamorphosed = true;
                e.metabolism.max_energy *= ctx.config.metabolism.adult_energy_multiplier;
                e.metabolism.peak_energy = e.metabolism.max_energy;

                e.intel.genotype.brain.remodel_for_adult();

                // Phase 58 Fix: Apply physical buffs to Physics component only to prevent genetic runaway
                e.physics.max_speed *= ctx.config.metabolism.adult_speed_multiplier;
                e.physics.sensing_range *= ctx.config.metabolism.adult_sensing_multiplier;

                let ev = LiveEvent::Metamorphosis {
                    id: e.id,
                    name: e.name(),
                    tick: ctx.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = ctx.logger.log_event(ev.clone());
                events.push(ev);
            }
            InteractionCommand::TribalSplit {
                target_idx,
                new_color,
            } => {
                let e = &mut entities[target_idx];
                e.physics.r = new_color.0;
                e.physics.g = new_color.1;
                e.physics.b = new_color.2;
                e.intel.genotype.lineage_id = uuid::Uuid::new_v4();
                e.metabolism.lineage_id = e.intel.genotype.lineage_id;
                ctx.lineage_registry.record_birth(
                    e.metabolism.lineage_id,
                    e.metabolism.generation,
                    ctx.tick,
                );
                let ev = LiveEvent::TribalSplit {
                    id: e.id,
                    lineage_id: e.metabolism.lineage_id,
                    tick: ctx.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = ctx.logger.log_event(ev.clone());
                events.push(ev);
            }
        }
    }

    InteractionResult {
        events,
        killed_ids,
        eaten_food_indices,
        new_babies,
    }
}
