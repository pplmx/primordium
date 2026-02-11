use crate::model::environment::Environment;
use crate::model::lifecycle;
use crate::model::world::{EntityDecision, SystemContext};
use primordium_core::interaction::InteractionCommand;
use primordium_core::systems::social;
use primordium_core::systems::social::ReproductionContext;
use primordium_data::{Food, Position, Specialization};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

pub struct EntityCommandInput<'a> {
    pub i: usize,
    pub identity: &'a primordium_data::Identity,
    pub pos: &'a primordium_data::Position,
    pub phys: &'a primordium_data::Physics,
    pub met: &'a primordium_data::Metabolism,
    pub intel: &'a primordium_data::Intel,
    pub health: &'a primordium_data::Health,
    pub decision: &'a EntityDecision,
}

pub struct BondContext<'a, R: rand::Rng> {
    pub i: usize,
    pub pos: &'a primordium_data::Position,
    pub met: &'a primordium_data::Metabolism,
    pub intel: &'a primordium_data::Intel,
    pub decision: &'a EntityDecision,
    pub ctx: &'a SystemContext<'a>,
    pub env: &'a Environment,
    pub pop_len: usize,
    pub id_map: &'a HashMap<uuid::Uuid, usize>,
    pub rng: &'a mut R,
}

pub struct ReproductionParams<'a, R: rand::Rng> {
    pub i: usize,
    pub pos: &'a primordium_data::Position,
    pub met: &'a primordium_data::Metabolism,
    pub intel: &'a primordium_data::Intel,
    pub decision: &'a EntityDecision,
    pub ctx: &'a SystemContext<'a>,
    pub env: &'a Environment,
    pub pop_len: usize,
    pub rng: &'a mut R,
}

pub struct PredationContext<'a> {
    pub i: usize,
    pub pos: &'a primordium_data::Position,
    pub phys: &'a primordium_data::Physics,
    pub met: &'a primordium_data::Metabolism,
    pub intel: &'a primordium_data::Intel,
    pub health: &'a primordium_data::Health,
    pub decision: &'a EntityDecision,
    pub ctx: &'a SystemContext<'a>,
    pub biomass_c: f64,
}

pub fn generate_eat_cmds(
    i: usize,
    pos: &primordium_data::Position,
    met: &primordium_data::Metabolism,
    intel: &primordium_data::Intel,
    decision: &EntityDecision,
    ctx: &SystemContext,
) -> Vec<InteractionCommand> {
    let mut acc = Vec::new();
    if let Some((f_idx, dx_f, dy_f, _)) = decision.sensed_food {
        if dx_f.abs() < 1.5 && dy_f.abs() < 1.5 {
            let food_handle = ctx.food_handles[f_idx];
            let mut energy_gain = 0.0;
            if let Ok(food_data) = ctx.ecs.get::<&Food>(food_handle) {
                let trophic_eff = 1.0 - met.trophic_potential as f64;
                if trophic_eff > 0.1 {
                    let niche_eff =
                        1.0 - (intel.genotype.metabolic_niche - food_data.nutrient_type).abs();
                    energy_gain = ctx.config.metabolism.food_value * niche_eff as f64 * trophic_eff;
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
    acc
}

pub fn generate_bond_cmds<R: rand::Rng>(input: BondContext<R>) -> Vec<InteractionCommand> {
    let mut acc = Vec::new();
    let outputs = input.decision.outputs;
    if outputs[8] > input.ctx.config.social.sharing_threshold {
        if let Some(p_id) = social::handle_symbiosis_components(
            input.i,
            input.ctx.snapshots,
            outputs,
            input.ctx.spatial_hash,
            input.ctx.config,
        ) {
            if let Some(&p_idx) = input.id_map.get(&p_id) {
                let partner_snap = &input.ctx.snapshots[p_idx];
                if let Some(partner_genotype) = partner_snap.genotype.as_ref() {
                    if input.met.energy > input.ctx.config.metabolism.reproduction_threshold
                        && partner_snap.energy > input.ctx.config.metabolism.reproduction_threshold
                    {
                        acc.push(InteractionCommand::Bond {
                            target_idx: input.i,
                            partner_id: p_id,
                        });

                        let mut repro_ctx = ReproductionContext {
                            tick: input.ctx.tick,
                            config: input.ctx.config,
                            population: input.pop_len,
                            traits: input.ctx.registry.get_traits(&input.met.lineage_id),
                            is_radiation_storm: input.env.is_radiation_storm(),
                            rng: input.rng,
                            ancestral_genotype: input
                                .ctx
                                .registry
                                .lineages
                                .get(&input.met.lineage_id)
                                .and_then(|r| r.max_fitness_genotype.as_ref()),
                        };

                        let mut modified_genotype = (*input.intel.genotype).clone();
                        modified_genotype.reproductive_investment = (modified_genotype
                            .reproductive_investment
                            * input.decision.grn_repro_mod)
                            .clamp(0.1, 0.9);

                        let (baby, dist) = social::reproduce_sexual_parallel_components_decomposed(
                            &social::ParentData {
                                pos: input.pos,
                                energy: input.met.energy,
                                generation: input.met.generation,
                                genotype: &modified_genotype,
                            },
                            &social::ParentData {
                                pos: &Position {
                                    x: partner_snap.x,
                                    y: partner_snap.y,
                                },
                                energy: partner_snap.energy,
                                generation: partner_snap.generation,
                                genotype: partner_genotype,
                            },
                            &mut repro_ctx,
                        );
                        acc.push(InteractionCommand::Birth {
                            parent_idx: input.i,
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
            let self_energy = input.met.energy;
            let partner_energy = input.ctx.snapshots[input.i].energy;
            if self_energy > partner_energy + 2.0 {
                let diff = self_energy - partner_energy;
                let amount = diff * input.ctx.config.social.sharing_fraction;
                if amount > 0.1 {
                    if let Some(p_id) = social::handle_symbiosis_components(
                        input.i,
                        input.ctx.snapshots,
                        outputs,
                        input.ctx.spatial_hash,
                        input.ctx.config,
                    ) {
                        if let Some(&p_idx) = input.id_map.get(&p_id) {
                            acc.push(InteractionCommand::TransferEnergy {
                                target_idx: p_idx,
                                amount,
                            });
                            acc.push(InteractionCommand::TransferEnergy {
                                target_idx: input.i,
                                amount: -amount,
                            });
                        }
                    }
                }
            }
        }
    }

    if let Some(p_id) = input.intel.bonded_to {
        if outputs[8] < 0.2 {
            acc.push(InteractionCommand::BondBreak {
                target_idx: input.i,
            });
        } else if let Some(&p_idx) = input.id_map.get(&p_id) {
            let partner_snap = &input.ctx.snapshots[p_idx];
            let self_energy = input.met.energy;
            let partner_energy = partner_snap.energy;
            if self_energy > partner_energy + 2.0 {
                let diff = self_energy - partner_energy;
                let amount = diff * input.ctx.config.social.sharing_fraction;
                if amount > 0.1 {
                    acc.push(InteractionCommand::TransferEnergy {
                        target_idx: p_idx,
                        amount,
                    });
                    acc.push(InteractionCommand::TransferEnergy {
                        target_idx: input.i,
                        amount: -amount,
                    });
                }
            }
        }
    }
    acc
}

pub fn generate_predation_cmds(input: PredationContext) -> Vec<InteractionCommand> {
    let mut acc = Vec::new();
    let outputs = input.decision.outputs;
    if outputs[4] > input.ctx.config.social.aggression_threshold
        && input.met.energy > input.met.max_energy * 0.7
    {
        input
            .ctx
            .spatial_hash
            .query_callback(input.pos.x, input.pos.y, 3.0, |t_idx| {
                let target_snap = &input.ctx.snapshots[t_idx];
                if input.i != t_idx {
                    let color_dist = (input.phys.r as i32 - target_snap.r as i32).abs()
                        + (input.phys.g as i32 - target_snap.g as i32).abs()
                        + (input.phys.b as i32 - target_snap.b as i32).abs();

                    if color_dist < input.ctx.config.social.tribe_color_threshold
                        && target_snap.energy
                            < target_snap.max_energy
                                * input.ctx.config.social.energy_sharing_low_threshold as f64
                    {
                        acc.push(InteractionCommand::TransferEnergy {
                            target_idx: t_idx,
                            amount: input.met.energy * input.ctx.config.social.sharing_fraction,
                        });
                        acc.push(InteractionCommand::TransferEnergy {
                            target_idx: input.i,
                            amount: -(input.met.energy * input.ctx.config.social.sharing_fraction),
                        });
                    }
                }
            });
    }

    if outputs[3] > 0.5 {
        input
            .ctx
            .spatial_hash
            .query_callback(input.pos.x, input.pos.y, 1.5, |t_idx| {
                if input.i != t_idx {
                    let target_snap = &input.ctx.snapshots[t_idx];
                    let color_dist = (input.phys.r as i32 - target_snap.r as i32).abs()
                        + (input.phys.g as i32 - target_snap.g as i32).abs()
                        + (input.phys.b as i32 - target_snap.b as i32).abs();

                    if target_snap.lineage_id == input.met.lineage_id {
                        return;
                    }

                    if color_dist >= input.ctx.config.social.tribe_color_threshold {
                        let mut multiplier = 1.0;
                        let attacker_status = lifecycle::calculate_status(
                            input.met,
                            input.health,
                            input.intel,
                            input.ctx.config.brain.activation_threshold,
                            input.ctx.tick,
                            input.ctx.config.metabolism.maturity_age,
                        );
                        if attacker_status == primordium_data::EntityStatus::Soldier
                            || input.intel.specialization == Some(Specialization::Soldier)
                        {
                            multiplier *= input.ctx.config.social.soldier_damage_mult;
                        }

                        let allies = input.ctx.spatial_hash.get_lineage_density(
                            target_snap.x,
                            target_snap.y,
                            target_snap.lineage_id,
                        ) as f64;

                        let defense_mult = (1.0
                            - allies * input.ctx.config.social.defense_per_ally_reduction)
                            .max(input.ctx.config.social.min_defense_multiplier);
                        let success_chance = (multiplier * defense_mult).min(1.0) as f32;

                        let competition_mult = (1.0
                            - (input.biomass_c
                                / input.ctx.config.ecosystem.predation_competition_scale))
                            .max(input.ctx.config.ecosystem.predation_min_efficiency);

                        let energy_gain = target_snap.energy
                            * input.ctx.config.ecosystem.predation_energy_gain_fraction
                            * multiplier
                            * defense_mult
                            * competition_mult;

                        acc.push(InteractionCommand::Kill {
                            target_idx: t_idx,
                            attacker_idx: input.i,
                            attacker_lineage: input.met.lineage_id,
                            cause: "Predation".to_string(),
                            precalculated_energy_gain: energy_gain,
                            success_chance,
                        });
                    }
                }
            });
    }
    acc
}

pub fn generate_reproduction_cmds<R: rand::Rng>(
    input: ReproductionParams<R>,
) -> Vec<InteractionCommand> {
    let mut acc = Vec::new();
    if lifecycle::is_mature_components(
        input.met,
        input.intel,
        input.ctx.tick,
        input.ctx.config.metabolism.maturity_age,
    ) && input.met.energy > input.ctx.config.metabolism.reproduction_threshold
    {
        let mut repro_ctx = ReproductionContext {
            tick: input.ctx.tick,
            config: input.ctx.config,
            population: input.pop_len,
            traits: input.ctx.registry.get_traits(&input.met.lineage_id),
            is_radiation_storm: input.env.is_radiation_storm(),
            rng: input.rng,
            ancestral_genotype: input
                .ctx
                .registry
                .lineages
                .get(&input.met.lineage_id)
                .and_then(|r| r.max_fitness_genotype.as_ref()),
        };

        let mut modified_genotype = (*input.intel.genotype).clone();
        modified_genotype.reproductive_investment = (modified_genotype.reproductive_investment
            * input.decision.grn_repro_mod)
            .clamp(0.1, 0.9);

        let (baby, dist) = social::reproduce_asexual_parallel_components_decomposed(
            social::AsexualReproductionContext {
                pos: input.pos,
                energy: input.met.energy,
                generation: input.met.generation,
                genotype: &modified_genotype,
                specialization: input.intel.specialization,
                ctx: &mut repro_ctx,
            },
        );
        acc.push(InteractionCommand::Birth {
            parent_idx: input.i,
            baby: Box::new(baby),
            genetic_distance: dist,
        });
    }
    acc
}

pub fn generate_commands_for_entity(
    input: EntityCommandInput,
    ctx: &SystemContext,
    env: &Environment,
    biomass_c: f64,
    id_map: &HashMap<uuid::Uuid, usize>,
    pop_len: usize,
) -> Vec<InteractionCommand> {
    let EntityCommandInput {
        i,
        identity,
        pos,
        phys,
        met,
        intel,
        health,
        decision,
    } = input;
    let mut acc = Vec::new();
    let u = identity.id.as_u128();
    let seed = ctx
        .world_seed
        .wrapping_add(ctx.tick)
        .wrapping_mul(0x517CC1B727220A95)
        ^ (u >> 64) as u64
        ^ (u as u64);
    let mut local_rng = ChaCha8Rng::seed_from_u64(seed);

    acc.extend(generate_eat_cmds(i, pos, met, intel, decision, ctx));
    acc.extend(generate_bond_cmds(BondContext {
        i,
        pos,
        met,
        intel,
        decision,
        ctx,
        env,
        pop_len,
        id_map,
        rng: &mut local_rng,
    }));
    acc.extend(generate_predation_cmds(PredationContext {
        i,
        pos,
        phys,
        met,
        intel,
        health,
        decision,
        ctx,
        biomass_c,
    }));
    acc.extend(generate_reproduction_cmds(ReproductionParams {
        i,
        pos,
        met,
        intel,
        decision,
        ctx,
        env,
        pop_len,
        rng: &mut local_rng,
    }));

    let outputs = decision.outputs;
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

    if !met.has_metamorphosed {
        let age = ctx.tick.saturating_sub(met.birth_tick);
        let threshold = (ctx.config.metabolism.maturity_age as f32
            * intel.genotype.maturity_gene
            * ctx.config.metabolism.metamorphosis_trigger_maturity) as u64;

        if age > threshold {
            acc.push(InteractionCommand::Metamorphosis { target_idx: i });
        }
    }

    acc
}
