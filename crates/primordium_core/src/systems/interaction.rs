use crate::brain::BrainLogic;
use crate::config::AppConfig;
use crate::environment::Environment;
use crate::history::{FossilRegistry, LiveEvent, PopulationStats};
use crate::interaction::InteractionCommand;
use crate::lifecycle;
use crate::lineage_registry::LineageRegistry;
use crate::systems::{biological, social};
use crate::terrain::{TerrainGrid, TerrainType};
use chrono::Utc;
use primordium_data::{Entity, Health, Intel, Metabolism, Physics, Specialization};
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use uuid::Uuid;

pub struct InteractionContext<'a, R: Rng> {
    pub terrain: &'a mut TerrainGrid,
    pub env: &'a mut Environment,
    pub pop_stats: &'a mut PopulationStats,
    pub lineage_registry: &'a mut LineageRegistry,
    pub fossil_registry: &'a mut FossilRegistry,
    pub config: &'a AppConfig,
    pub tick: u64,
    pub width: u16,
    pub height: u16,
    pub social_grid: &'a mut [u8],
    pub lineage_consumption: &'a mut Vec<(Uuid, f64)>,
    pub food_handles: &'a [hecs::Entity],
    pub spatial_hash: &'a crate::spatial_hash::SpatialHash,
    pub rng: &'a mut R,
    pub food_count: &'a std::sync::atomic::AtomicUsize,
    pub world_seed: u64,
}

pub struct InteractionResult {
    pub events: Vec<LiveEvent>,
    pub killed_ids: HashSet<uuid::Uuid>,
    pub eaten_food_indices: HashSet<usize>,
    pub new_babies: Vec<Entity>,
}

pub fn process_interaction_commands_ecs<R: Rng>(
    world: &mut hecs::World,
    entity_handles: &[hecs::Entity],
    commands: Vec<InteractionCommand>,
    ctx: &mut InteractionContext<R>,
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
                precalculated_energy_gain,
                success_chance,
            } => {
                let target_handle = entity_handles[target_idx];
                let attacker_handle = entity_handles[attacker_idx];

                let mut target_info = None;
                if let (Ok(target_identity), Ok(target_metabolism)) = (
                    world.get::<&primordium_data::Identity>(target_handle),
                    world.get::<&Metabolism>(target_handle),
                ) {
                    if !killed_ids.contains(&target_identity.id) {
                        target_info = Some((
                            target_identity.id,
                            target_metabolism.birth_tick,
                            target_metabolism.offspring_count,
                        ));
                    }
                }

                if let Some((tid, target_birth, target_offspring)) = target_info {
                    let u = tid.as_u128();
                    let mut seed = ctx
                        .tick
                        .wrapping_add(ctx.world_seed)
                        .wrapping_mul(0x517CC1B727220A95);
                    seed ^= (u >> 64) as u64;
                    seed = seed.wrapping_mul(0x517CC1B727220A95);
                    seed ^= u as u64;
                    seed ^= 0xDEADBEEF;

                    let mut local_rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
                    if local_rng.gen::<f32>() < success_chance {
                        killed_ids.insert(tid);
                        crate::systems::stats::record_stat_death(
                            ctx.pop_stats,
                            ctx.tick - target_birth,
                        );

                        let ev = LiveEvent::Death {
                            id: tid,
                            age: ctx.tick - target_birth,
                            offspring: target_offspring,
                            tick: ctx.tick,
                            timestamp: Utc::now().to_rfc3339(),
                            cause: cause.clone(),
                        };
                        events.push(ev);

                        ctx.lineage_consumption
                            .push((attacker_lineage, precalculated_energy_gain));
                        ctx.lineage_registry
                            .boost_memory_value(&attacker_lineage, "goal", 0.5);
                        ctx.lineage_registry.boost_memory_value(&tid, "threat", 1.0);

                        if let Ok(mut attacker_met_mut) =
                            world.get::<&mut Metabolism>(attacker_handle)
                        {
                            attacker_met_mut.energy = (attacker_met_mut.energy
                                + precalculated_energy_gain)
                                .min(attacker_met_mut.max_energy);
                        }
                    }
                }
            }
            InteractionCommand::Bond {
                target_idx,
                partner_id,
            } => {
                let handle = entity_handles[target_idx];
                if let Ok(mut intel) = world.get::<&mut Intel>(handle) {
                    intel.bonded_to = Some(partner_id);
                }
            }
            InteractionCommand::BondBreak { target_idx } => {
                let handle = entity_handles[target_idx];
                if let Ok(mut intel) = world.get::<&mut Intel>(handle) {
                    intel.bonded_to = None;
                }
            }
            InteractionCommand::Birth {
                parent_idx,
                mut baby,
                genetic_distance,
            } => {
                ctx.lineage_registry.record_birth(
                    baby.metabolism.lineage_id,
                    baby.metabolism.generation,
                    ctx.tick,
                );
                crate::systems::stats::record_stat_birth_distance(ctx.pop_stats, genetic_distance);
                let ev = LiveEvent::Birth {
                    id: baby.identity.id,
                    parent_id: baby.identity.parent_id,
                    gen: baby.metabolism.generation,
                    tick: ctx.tick,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                events.push(ev);

                let terrain_type = ctx.terrain.get(baby.physics.x, baby.physics.y).terrain_type;
                if matches!(terrain_type, TerrainType::Nest) {
                    baby.metabolism.energy *= ctx.config.metabolism.birth_energy_multiplier;
                    baby.metabolism.peak_energy = baby.metabolism.energy;
                }

                new_babies.push(*baby);

                let parent_handle = entity_handles[parent_idx];
                if let (Ok(mut parent_met), Ok(parent_intel)) = (
                    world.get::<&mut Metabolism>(parent_handle),
                    world.get::<&Intel>(parent_handle),
                ) {
                    let inv = parent_intel.genotype.reproductive_investment as f64;
                    let c_e = parent_met.energy * inv;
                    parent_met.energy -= c_e;
                    parent_met.offspring_count += 1;
                }
            }
            InteractionCommand::EatFood {
                food_index,
                attacker_idx,
                x,
                y,
                precalculated_energy_gain,
            } => {
                if !eaten_food_indices.contains(&food_index) {
                    let food_handle = ctx.food_handles[food_index];
                    let handle = entity_handles[attacker_idx];

                    eaten_food_indices.insert(food_index);
                    let _ = world.despawn(food_handle);
                    ctx.food_count
                        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    if let Ok(mut met_mut) = world.get::<&mut Metabolism>(handle) {
                        met_mut.energy =
                            (met_mut.energy + precalculated_energy_gain).min(met_mut.max_energy);
                        let lid = met_mut.lineage_id;
                        ctx.lineage_registry.boost_memory_value(&lid, "goal", 0.2);
                        ctx.terrain
                            .deplete(x, y, ctx.config.ecosystem.soil_depletion_unit);
                        ctx.lineage_consumption
                            .push((lid, precalculated_energy_gain));
                    }
                }
            }
            InteractionCommand::TransferEnergy { target_idx, amount } => {
                let handle = entity_handles[target_idx];
                let mut actual_amount = amount;
                if amount < 0.0 {
                    if let Ok(intel) = world.get::<&Intel>(handle) {
                        if intel.specialization == Some(Specialization::Provider) {
                            actual_amount *= ctx.config.terraform.engineer_discount;
                        }
                    }
                }
                if let Ok(mut met) = world.get::<&mut Metabolism>(handle) {
                    met.energy = (met.energy + actual_amount).clamp(0.0, met.max_energy);
                }
            }
            InteractionCommand::Infect {
                target_idx,
                pathogen,
            } => {
                let handle = entity_handles[target_idx];
                if let Ok(mut health) = world.get::<&mut Health>(handle) {
                    biological::try_infect_components(&mut health, &pathogen, ctx.rng);
                }
            }
            InteractionCommand::UpdateReputation { target_idx, delta } => {
                let handle = entity_handles[target_idx];
                if let Ok(mut intel) = world.get::<&mut Intel>(handle) {
                    intel.reputation = (intel.reputation + delta).clamp(0.0, 1.0);
                }
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
                let handle = entity_handles[attacker_idx];
                let cell = ctx.terrain.get(x, y);
                if let (Ok(mut met), Ok(mut intel)) = (
                    world.get::<&mut Metabolism>(handle),
                    world.get::<&mut Intel>(handle),
                ) {
                    let mut energy_cost = ctx.config.terraform.dig_cost;
                    ctx.env.consume_oxygen(ctx.config.terraform.dig_oxygen_cost);
                    if intel.specialization == Some(Specialization::Engineer) {
                        energy_cost *= ctx.config.terraform.engineer_discount;
                    }
                    if matches!(cell.terrain_type, TerrainType::Wall | TerrainType::Mountain) {
                        if met.energy > energy_cost {
                            met.energy -= energy_cost;
                            ctx.terrain
                                .set_cell_type(x as u16, y as u16, TerrainType::Barren);
                            social::increment_spec_meter_components(
                                &mut intel,
                                Specialization::Engineer,
                                1.0,
                                ctx.config,
                            );
                        }
                    } else if matches!(cell.terrain_type, TerrainType::Plains) {
                        let eff_hydro_cost = if intel.specialization
                            == Some(Specialization::Engineer)
                        {
                            ctx.config.terraform.canal_cost * ctx.config.terraform.engineer_discount
                        } else {
                            ctx.config.terraform.canal_cost
                        };
                        if met.energy > 50.0
                            && ctx
                                .terrain
                                .has_neighbor_type(x as u16, y as u16, TerrainType::River)
                        {
                            met.energy -= eff_hydro_cost;
                            ctx.terrain
                                .set_cell_type(x as u16, y as u16, TerrainType::River);
                            social::increment_spec_meter_components(
                                &mut intel,
                                Specialization::Engineer,
                                2.0,
                                ctx.config,
                            );
                        }
                    }
                }
            }
            InteractionCommand::Build {
                x,
                y,
                attacker_idx,
                is_nest,
                is_outpost,
                outpost_spec,
            } => {
                let handle = entity_handles[attacker_idx];
                let cell = ctx.terrain.get(x, y);
                if let (Ok(mut met), Ok(mut intel)) = (
                    world.get::<&mut Metabolism>(handle),
                    world.get::<&mut Intel>(handle),
                ) {
                    let mut energy_cost = if is_outpost {
                        ctx.config.terraform.nest_energy_req * 2.0
                    } else {
                        ctx.config.terraform.build_cost
                    };
                    ctx.env
                        .consume_oxygen(ctx.config.terraform.build_oxygen_cost);
                    if intel.specialization == Some(Specialization::Engineer) {
                        energy_cost *= ctx.config.terraform.engineer_discount;
                    }
                    if matches!(cell.terrain_type, TerrainType::Plains) && met.energy > energy_cost
                    {
                        let new_type = if is_outpost
                            && met.energy > ctx.config.terraform.nest_energy_req * 3.0
                        {
                            TerrainType::Outpost
                        } else if is_nest && met.energy > ctx.config.terraform.nest_energy_req {
                            TerrainType::Nest
                        } else {
                            TerrainType::Wall
                        };
                        met.energy -= energy_cost;
                        let idx = ctx.terrain.index(x as u16, y as u16);
                        ctx.terrain.set_cell_type(x as u16, y as u16, new_type);
                        if let Some(c) = ctx.terrain.cells.get_mut(idx) {
                            c.owner_id = Some(met.lineage_id);
                            if is_outpost {
                                if let Some(s) = outpost_spec {
                                    c.outpost_spec = s;
                                }
                            }
                        }
                        social::increment_spec_meter_components(
                            &mut intel,
                            Specialization::Engineer,
                            if is_outpost { 5.0 } else { 1.0 },
                            ctx.config,
                        );
                    }
                }
            }
            InteractionCommand::Metamorphosis { target_idx } => {
                let handle = entity_handles[target_idx];
                if let (Ok(mut met), Ok(mut intel), Ok(mut phys)) = (
                    world.get::<&mut Metabolism>(handle),
                    world.get::<&mut Intel>(handle),
                    world.get::<&mut Physics>(handle),
                ) {
                    met.has_metamorphosed = true;
                    met.max_energy *= ctx.config.metabolism.adult_energy_multiplier;
                    met.peak_energy = met.max_energy;
                    std::sync::Arc::make_mut(&mut intel.genotype)
                        .brain
                        .remodel_for_adult_with_rng(ctx.rng);
                    phys.max_speed *= ctx.config.metabolism.adult_speed_multiplier;
                    phys.sensing_range *= ctx.config.metabolism.adult_sensing_multiplier;
                    if let Ok(identity) = world.get::<&primordium_data::Identity>(handle) {
                        let id = identity.id;
                        let ev = LiveEvent::Metamorphosis {
                            id,
                            name: lifecycle::get_name_components(&id, &met),
                            tick: ctx.tick,
                            timestamp: Utc::now().to_rfc3339(),
                        };
                        events.push(ev);
                    }
                }
            }
            InteractionCommand::TribalSplit {
                target_idx,
                new_color,
            } => {
                let handle = entity_handles[target_idx];
                if let (Ok(mut phys), Ok(mut intel), Ok(mut met)) = (
                    world.get::<&mut Physics>(handle),
                    world.get::<&mut Intel>(handle),
                    world.get::<&mut Metabolism>(handle),
                ) {
                    phys.r = new_color.0;
                    phys.g = new_color.1;
                    phys.b = new_color.2;
                    let new_lineage_id = Uuid::from_u128(ctx.rng.gen());
                    std::sync::Arc::make_mut(&mut intel.genotype).lineage_id = new_lineage_id;
                    met.lineage_id = new_lineage_id;
                    ctx.lineage_registry
                        .record_birth(met.lineage_id, met.generation, ctx.tick);
                    if let Ok(identity) = world.get::<&primordium_data::Identity>(handle) {
                        let id = identity.id;
                        let ev = LiveEvent::TribalSplit {
                            id,
                            lineage_id: met.lineage_id,
                            tick: ctx.tick,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };
                        events.push(ev);
                    }
                }
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
