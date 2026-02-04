use crate::model::environment::Environment;
use crate::model::world::{EntityComponents, EntityDecision, SystemContext};
use hecs;
use primordium_core::interaction::InteractionCommand;
use primordium_core::systems::{action, ecological, intel, social};
use primordium_data::{Food, Position, Specialization};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use std::collections::HashMap;

use crate::model::brain::BrainLogic;
use crate::model::lifecycle;
use primordium_core::pheromone::PheromoneGrid;
use primordium_core::pressure::PressureGrid;
use primordium_core::sound::SoundGrid;
use social::ReproductionContext;

pub fn perceive_and_decide_internal(
    ctx: &SystemContext,
    env: &Environment,
    biomass_c: f64,
    entity_data: &mut [(hecs::Entity, EntityComponents)],
    id_map: &HashMap<uuid::Uuid, usize>,
    interaction_commands: &mut Vec<InteractionCommand>,
    decision_buffer: &mut Vec<EntityDecision>,
) {
    decision_buffer.clear();
    decision_buffer.resize(entity_data.len(), EntityDecision::default());
    interaction_commands.clear();
    let pop_len = entity_data.len();

    entity_data
        .par_iter_mut()
        .zip(decision_buffer.par_iter_mut())
        .for_each(
            |((_handle, (identity, pos, _vel, phys, met, intel, health)), decision)| {
                let nearby_kin = ctx.spatial_hash.count_nearby_kin_fast(
                    pos.x,
                    pos.y,
                    phys.sensing_range,
                    met.lineage_id,
                );

                let (speed_mod, sensing_mod, repro_mod) = intel::apply_grn_rules(
                    &intel.genotype,
                    met,
                    env.oxygen_level,
                    env.carbon_level,
                    nearby_kin,
                    ctx.tick,
                );

                let eff_sensing_range = phys.sensing_range * sensing_mod;

                let (best_idx_f, dx_f, dy_f, f_type) = ecological::sense_nearest_food_data(
                    pos,
                    eff_sensing_range,
                    ctx.food_hash,
                    ctx.food_data,
                );
                let sensed_food = best_idx_f.map(|idx| (idx, dx_f, dy_f, f_type));
                let nearby_count = ctx
                    .spatial_hash
                    .count_nearby(pos.x, pos.y, eff_sensing_range);
                let (ph_f, tribe_d, sa, sb) =
                    ctx.pheromones
                        .sense_all(pos.x, pos.y, eff_sensing_range / 2.0);
                let (kx, ky) =
                    ctx.spatial_hash
                        .sense_kin(pos.x, pos.y, eff_sensing_range, met.lineage_id);
                let wall_dist = ctx.terrain.sense_wall(pos.x, pos.y, 5.0);
                let age_ratio = (ctx.tick - met.birth_tick) as f32 / 2000.0;
                let sound_sense = ctx.sound.sense(pos.x, pos.y, eff_sensing_range);
                let mut partner_energy = 0.0;
                if let Some(p_id) = intel.bonded_to {
                    if let Some(&p_idx) = id_map.get(&p_id) {
                        partner_energy =
                            (ctx.snapshots[p_idx].energy / met.max_energy.max(1.0)) as f32;
                    }
                }
                let (d_press, b_press) = ctx.pressure.sense(pos.x, pos.y, eff_sensing_range);
                let shared_goal = ctx.registry.get_memory_value(&met.lineage_id, "goal");
                let shared_threat = ctx.registry.get_memory_value(&met.lineage_id, "threat");
                let mut lin_pop = 0.0;
                let mut lin_energy = 0.0;
                let mut overmind_signal = 0.0;
                if let Some(record) = ctx.registry.lineages.get(&met.lineage_id) {
                    lin_pop = (record.current_population as f32 / 100.0).min(1.0);
                    lin_energy = (record.total_energy_consumed as f32 / 10000.0).min(1.0);
                    overmind_signal = ctx.registry.get_memory_value(&met.lineage_id, "overmind");
                }

                let inputs = [
                    (dx_f / 20.0) as f32,
                    (dy_f / 20.0) as f32,
                    (met.energy / met.max_energy.max(1.0)) as f32,
                    (nearby_count as f32 / 10.0).min(1.0),
                    ph_f,
                    tribe_d,
                    kx as f32,
                    ky as f32,
                    sa,
                    sb,
                    wall_dist,
                    age_ratio.min(1.0),
                    f_type,
                    met.trophic_potential,
                    intel.last_hidden[0],
                    intel.last_hidden[1],
                    intel.last_hidden[2],
                    intel.last_hidden[3],
                    intel.last_hidden[4],
                    intel.last_hidden[5],
                    sound_sense,
                    partner_energy,
                    b_press,
                    d_press,
                    shared_goal,
                    shared_threat,
                    lin_pop,
                    lin_energy,
                    overmind_signal,
                ];

                let u = identity.id.as_u128();
                let seed = ctx
                    .world_seed
                    .wrapping_add(ctx.tick)
                    .wrapping_mul(0x517CC1B727220A95)
                    ^ (u >> 64) as u64
                    ^ u as u64;
                let _local_rng = ChaCha8Rng::seed_from_u64(seed);

                let (mut outputs, next_hidden) = intel.genotype.brain.forward_internal(
                    inputs,
                    intel.last_hidden,
                    &mut intel.last_activations,
                );
                if let Some(ref path) = health.pathogen {
                    if let Some((idx, offset)) = path.behavior_manipulation {
                        let out_idx = idx.saturating_sub(22);
                        if out_idx < 11 {
                            outputs[out_idx] = (outputs[out_idx] + offset).clamp(-1.0, 1.0);
                        }
                    }
                }
                intel.last_hidden = next_hidden;
                *decision = EntityDecision {
                    outputs,
                    nearby_count,
                    grn_speed_mod: speed_mod,
                    grn_sensing_mod: sensing_mod,
                    grn_repro_mod: repro_mod,
                    sensed_food,
                };
            },
        );

    // Deterministic Command Collection
    let all_cmds_flat: Vec<InteractionCommand> = entity_data
        .par_iter_mut()
        .enumerate()
        .fold(
            Vec::new,
            |mut acc, (i, (_handle, (identity, pos, _vel, phys, met, intel, health)))| {
                let u = identity.id.as_u128();
                let seed = ctx
                    .world_seed
                    .wrapping_add(ctx.tick)
                    .wrapping_mul(0x517CC1B727220A95)
                    ^ (u >> 64) as u64
                    ^ (u as u64);
                let mut local_rng = ChaCha8Rng::seed_from_u64(seed);

                let decision = &decision_buffer[i];
                let outputs = decision.outputs;

                if let Some((f_idx, dx_f, dy_f, _)) = decision.sensed_food {
                    if dx_f.abs() < 1.5 && dy_f.abs() < 1.5 {
                        let food_handle = ctx.food_handles[f_idx];
                        let mut energy_gain = 0.0;
                        if let Ok(food_data) = ctx.ecs.get::<&Food>(food_handle) {
                            let trophic_eff = 1.0 - met.trophic_potential as f64;
                            if trophic_eff > 0.1 {
                                let niche_eff = 1.0
                                    - (intel.genotype.metabolic_niche - food_data.nutrient_type)
                                        .abs();
                                energy_gain = ctx.config.metabolism.food_value
                                    * niche_eff as f64
                                    * trophic_eff;
                            }
                        }
                        if energy_gain > 0.0 {
                            acc.push(InteractionCommand::EatFood {
                                food_index: f_idx,
                                attacker_idx: i,
                                x: pos.x,
                                y: pos.y,
                                precalculated_energy_gain: energy_gain,
                            });
                        }
                    }
                }

                if outputs[8] > ctx.config.social.sharing_threshold {
                    if let Some(p_id) = social::handle_symbiosis_components(
                        i,
                        ctx.snapshots,
                        outputs,
                        ctx.spatial_hash,
                        ctx.config,
                    ) {
                        if let Some(&p_idx) = id_map.get(&p_id) {
                            let partner_snap = &ctx.snapshots[p_idx];
                            if let Some(partner_genotype) = partner_snap.genotype.as_ref() {
                                if met.energy > ctx.config.metabolism.reproduction_threshold
                                    && partner_snap.energy
                                        > ctx.config.metabolism.reproduction_threshold
                                {
                                    acc.push(InteractionCommand::Bond {
                                        target_idx: i,
                                        partner_id: p_id,
                                    });

                                    let mut repro_ctx = ReproductionContext {
                                        tick: ctx.tick,
                                        config: ctx.config,
                                        population: pop_len,
                                        traits: ctx.registry.get_traits(&met.lineage_id),
                                        is_radiation_storm: env.is_radiation_storm(),
                                        rng: &mut local_rng,
                                        ancestral_genotype: ctx
                                            .registry
                                            .lineages
                                            .get(&met.lineage_id)
                                            .and_then(|r| r.max_fitness_genotype.as_ref()),
                                    };

                                    let mut modified_genotype = intel.genotype.clone();
                                    modified_genotype.reproductive_investment = (modified_genotype
                                        .reproductive_investment
                                        * decision.grn_repro_mod)
                                        .clamp(0.1, 0.9);

                                    let (baby, dist) =
                                        social::reproduce_sexual_parallel_components_decomposed(
                                            pos,
                                            met.energy,
                                            met.generation,
                                            &modified_genotype,
                                            &Position {
                                                x: partner_snap.x,
                                                y: partner_snap.y,
                                            },
                                            partner_snap.energy,
                                            partner_snap.generation,
                                            partner_genotype,
                                            &mut repro_ctx,
                                        );
                                    acc.push(InteractionCommand::Birth {
                                        parent_idx: i,
                                        baby: Box::new(baby),
                                        genetic_distance: dist,
                                    });
                                    acc.push(InteractionCommand::TransferEnergy {
                                        target_idx: p_idx,
                                        amount: -(partner_snap.energy
                                            * partner_genotype.reproductive_investment as f64),
                                    });
                                }
                            }
                        }
                        let self_energy = met.energy;
                        let partner_energy = ctx.snapshots[i].energy;
                        if self_energy > partner_energy + 2.0 {
                            let diff = self_energy - partner_energy;
                            let amount = diff * ctx.config.social.sharing_fraction;
                            if amount > 0.1 {
                                if let Some(p_id) = social::handle_symbiosis_components(
                                    i,
                                    ctx.snapshots,
                                    outputs,
                                    ctx.spatial_hash,
                                    ctx.config,
                                ) {
                                    if let Some(&p_idx) = id_map.get(&p_id) {
                                        acc.push(InteractionCommand::TransferEnergy {
                                            target_idx: p_idx,
                                            amount,
                                        });
                                        acc.push(InteractionCommand::TransferEnergy {
                                            target_idx: i,
                                            amount: -amount,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(p_id) = intel.bonded_to {
                    if outputs[8] < 0.2 {
                        acc.push(InteractionCommand::BondBreak { target_idx: i });
                    } else if let Some(&p_idx) = id_map.get(&p_id) {
                        let partner_snap = &ctx.snapshots[p_idx];
                        let self_energy = met.energy;
                        let partner_energy = partner_snap.energy;
                        if self_energy > partner_energy + 2.0 {
                            let diff = self_energy - partner_energy;
                            let amount = diff * ctx.config.social.sharing_fraction;
                            if amount > 0.1 {
                                acc.push(InteractionCommand::TransferEnergy {
                                    target_idx: p_idx,
                                    amount,
                                });
                                acc.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: -amount,
                                });
                            }
                        }
                    }
                }

                if outputs[4] > ctx.config.social.aggression_threshold
                    && met.energy > met.max_energy * 0.7
                {
                    ctx.spatial_hash.query_callback(pos.x, pos.y, 3.0, |t_idx| {
                        let target_snap = &ctx.snapshots[t_idx];
                        if i != t_idx {
                            let color_dist = (phys.r as i32 - target_snap.r as i32).abs()
                                + (phys.g as i32 - target_snap.g as i32).abs()
                                + (phys.b as i32 - target_snap.b as i32).abs();

                            if color_dist < ctx.config.social.tribe_color_threshold
                                && target_snap.energy
                                    < target_snap.max_energy
                                        * ctx.config.social.energy_sharing_low_threshold as f64
                            {
                                acc.push(InteractionCommand::TransferEnergy {
                                    target_idx: t_idx,
                                    amount: met.energy * ctx.config.social.sharing_fraction,
                                });
                                acc.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: -(met.energy * ctx.config.social.sharing_fraction),
                                });
                            }
                        }
                    });
                }

                if outputs[3] > 0.5 {
                    ctx.spatial_hash.query_callback(pos.x, pos.y, 1.5, |t_idx| {
                        if i != t_idx {
                            let target_snap = &ctx.snapshots[t_idx];
                            let color_dist = (phys.r as i32 - target_snap.r as i32).abs()
                                + (phys.g as i32 - target_snap.g as i32).abs()
                                + (phys.b as i32 - target_snap.b as i32).abs();

                            if target_snap.lineage_id == met.lineage_id {
                                return;
                            }

                            if color_dist >= ctx.config.social.tribe_color_threshold {
                                let mut multiplier = 1.0;
                                let attacker_status = lifecycle::calculate_status(
                                    met,
                                    health,
                                    intel,
                                    ctx.config.brain.activation_threshold,
                                    ctx.tick,
                                    ctx.config.metabolism.maturity_age,
                                );
                                if attacker_status == primordium_data::EntityStatus::Soldier
                                    || intel.specialization == Some(Specialization::Soldier)
                                {
                                    multiplier *= ctx.config.social.soldier_damage_mult;
                                }

                                let allies = ctx.spatial_hash.get_lineage_density(
                                    target_snap.x,
                                    target_snap.y,
                                    target_snap.lineage_id,
                                ) as f64;

                                let defense_mult = (1.0
                                    - allies * ctx.config.social.defense_per_ally_reduction)
                                    .max(ctx.config.social.min_defense_multiplier);
                                let success_chance = (multiplier * defense_mult).min(1.0) as f32;

                                let competition_mult = (1.0
                                    - (biomass_c
                                        / ctx.config.ecosystem.predation_competition_scale))
                                    .max(ctx.config.ecosystem.predation_min_efficiency);

                                let energy_gain = target_snap.energy
                                    * ctx.config.ecosystem.predation_energy_gain_fraction
                                    * multiplier
                                    * defense_mult
                                    * competition_mult;

                                acc.push(InteractionCommand::Kill {
                                    target_idx: t_idx,
                                    attacker_idx: i,
                                    attacker_lineage: met.lineage_id,
                                    cause: "Predation".to_string(),
                                    precalculated_energy_gain: energy_gain,
                                    success_chance,
                                });
                            }
                        }
                    });
                }

                if lifecycle::is_mature_components(
                    met,
                    intel,
                    ctx.tick,
                    ctx.config.metabolism.maturity_age,
                ) && met.energy > ctx.config.metabolism.reproduction_threshold
                {
                    let mut repro_ctx = ReproductionContext {
                        tick: ctx.tick,
                        config: ctx.config,
                        population: pop_len,
                        traits: ctx.registry.get_traits(&met.lineage_id),
                        is_radiation_storm: env.is_radiation_storm(),
                        rng: &mut local_rng,
                        ancestral_genotype: ctx
                            .registry
                            .lineages
                            .get(&met.lineage_id)
                            .and_then(|r| r.max_fitness_genotype.as_ref()),
                    };

                    let mut modified_genotype = intel.genotype.clone();
                    modified_genotype.reproductive_investment =
                        (modified_genotype.reproductive_investment * decision.grn_repro_mod)
                            .clamp(0.1, 0.9);

                    let (baby, dist) = social::reproduce_asexual_parallel_components_decomposed(
                        pos,
                        met.energy,
                        met.generation,
                        &modified_genotype,
                        intel.specialization,
                        &mut repro_ctx,
                    );
                    acc.push(InteractionCommand::Birth {
                        parent_idx: i,
                        baby: Box::new(baby),
                        genetic_distance: dist,
                    });
                }

                if (outputs[9] > 0.5 || outputs[10] > 0.5) && met.has_metamorphosed {
                    if outputs[9] > outputs[10] {
                        acc.push(InteractionCommand::Dig {
                            x: phys.x,
                            y: phys.y,
                            attacker_idx: i,
                        });
                    } else {
                        let build_val = outputs[10];
                        let spec = if build_val > 0.9 {
                            Some(primordium_data::OutpostSpecialization::Nursery)
                        } else if build_val > 0.8 {
                            Some(primordium_data::OutpostSpecialization::Silo)
                        } else {
                            Some(primordium_data::OutpostSpecialization::Standard)
                        };
                        acc.push(InteractionCommand::Build {
                            x: phys.x,
                            y: phys.y,
                            attacker_idx: i,
                            is_nest: true,
                            is_outpost: build_val > 0.8,
                            outpost_spec: spec,
                        });
                    }
                }

                if outputs[1] > 0.5 {
                    acc.push(InteractionCommand::UpdateReputation {
                        target_idx: i,
                        delta: 0.01,
                    });
                }

                if let Some(new_color) = social::start_tribal_split_components(
                    phys,
                    met,
                    intel,
                    decision.nearby_count as f32 / 10.0,
                    ctx.config,
                    &mut local_rng,
                ) {
                    acc.push(InteractionCommand::TribalSplit {
                        target_idx: i,
                        new_color,
                    });
                }

                if !met.has_metamorphosed
                    && (ctx.tick - met.birth_tick)
                        > (ctx.config.metabolism.maturity_age as f32
                            * intel.genotype.maturity_gene
                            * ctx.config.metabolism.metamorphosis_trigger_maturity)
                            as u64
                {
                    acc.push(InteractionCommand::Metamorphosis { target_idx: i });
                }

                acc
            },
        )
        .reduce(Vec::new, |mut a, b| {
            a.extend(b);
            a
        });

    interaction_commands.extend(all_cmds_flat);

    interaction_commands.sort_by_key(|cmd| match cmd {
        InteractionCommand::Kill { attacker_idx, .. } => *attacker_idx,
        InteractionCommand::EatFood { attacker_idx, .. } => *attacker_idx,
        InteractionCommand::Birth { parent_idx, .. } => *parent_idx,
        InteractionCommand::Bond { target_idx, .. } => *target_idx,
        InteractionCommand::BondBreak { target_idx, .. } => *target_idx,
        InteractionCommand::TransferEnergy { target_idx, .. } => *target_idx,
        InteractionCommand::Dig { attacker_idx, .. } => *attacker_idx,
        InteractionCommand::Build { attacker_idx, .. } => *attacker_idx,
        InteractionCommand::TribalSplit { target_idx, .. } => *target_idx,
        InteractionCommand::Metamorphosis { target_idx, .. } => *target_idx,
        _ => 0,
    });
}

pub fn calculate_actions_parallel(
    ctx: &SystemContext,
    env: &mut Environment,
    entity_id_map: &HashMap<uuid::Uuid, usize>,
    entity_data: &mut [(hecs::Entity, EntityComponents)],
    decision_buffer: &mut [EntityDecision],
) -> Vec<(action::ActionOutput, f64)> {
    entity_data
        .par_iter_mut()
        .zip(decision_buffer.par_iter_mut())
        .map(
            |((_handle, (identity, pos, velocity, phys, met, intel, _health)), decision)| {
                let EntityDecision {
                    outputs,
                    grn_speed_mod,
                    ..
                } = std::mem::take(decision);
                intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

                let mut output = action::ActionOutput::default();

                let eff_max_speed = phys.max_speed * grn_speed_mod;

                action::action_system_components_with_modifiers(
                    &identity.id,
                    pos,
                    velocity,
                    phys,
                    eff_max_speed,
                    met,
                    intel,
                    _health,
                    outputs,
                    &mut action::ActionContext {
                        env,
                        config: ctx.config,
                        terrain: ctx.terrain,
                        influence: ctx.influence,
                        snapshots: ctx.snapshots,
                        entity_id_map,
                        spatial_hash: ctx.spatial_hash,
                        pressure: ctx.pressure,
                        width: ctx.config.world.width,
                        height: ctx.config.world.height,
                    },
                    &mut output,
                );
                let drain = output.oxygen_drain;
                (output, drain)
            },
        )
        .collect()
}

pub fn apply_actions_sequential(
    all_outputs: Vec<(action::ActionOutput, f64)>,
    pheromones: &mut PheromoneGrid,
    sound: &mut SoundGrid,
    pressure: &mut PressureGrid,
    env: &mut Environment,
) -> Vec<(uuid::Uuid, f32)> {
    let mut overmind_broadcasts = Vec::new();
    let mut total_oxygen_drain = 0.0;

    for (output, drain) in all_outputs {
        total_oxygen_drain += drain;
        if let Some(b) = output.overmind_broadcast {
            overmind_broadcasts.push(b);
        }

        // Deterministic sequential deposits
        for p in output.pheromones {
            pheromones.deposit(p.x, p.y, p.ptype, p.amount);
        }
        for s in output.sounds {
            sound.deposit(s.x, s.y, s.amount);
        }
        for pr in output.pressure {
            pressure.deposit(pr.x, pr.y, pr.ptype, pr.amount);
        }
    }

    env.consume_oxygen(total_oxygen_drain);
    overmind_broadcasts
}
