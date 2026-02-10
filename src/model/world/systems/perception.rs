use crate::model::brain::BrainLogic;
use crate::model::environment::Environment;
use crate::model::world::{EntityDecision, SystemContext};
use primordium_core::systems::{ecological, intel};
use std::collections::HashMap;

pub struct EntityPerceptionInput<'a> {
    pub identity: &'a primordium_data::Identity,
    pub pos: &'a primordium_data::Position,
    pub phys: &'a primordium_data::Physics,
    pub met: &'a primordium_data::Metabolism,
    pub intel: &'a mut primordium_data::Intel,
    pub health: &'a primordium_data::Health,
}

pub fn perceive_one_entity(
    input: EntityPerceptionInput,
    ctx: &SystemContext,
    env: &Environment,
    id_map: &HashMap<uuid::Uuid, usize>,
) -> EntityDecision {
    let EntityPerceptionInput {
        identity: _identity,
        pos,
        phys,
        met,
        intel,
        health,
    } = input;
    let nearby_kin =
        ctx.spatial_hash
            .count_nearby_kin_fast(pos.x, pos.y, phys.sensing_range, met.lineage_id);

    let (speed_mod, sensing_mod, repro_mod) = intel::apply_grn_rules(intel::GrnContext {
        genotype: &intel.genotype,
        metabolism: met,
        oxygen_level: env.oxygen_level,
        carbon_level: env.carbon_level,
        nearby_kin,
        tick: ctx.tick,
    });

    let eff_sensing_range = phys.sensing_range * sensing_mod;

    let (best_idx_f, dx_f, dy_f, f_type) =
        ecological::sense_nearest_food_data(pos, eff_sensing_range, ctx.food_hash, ctx.food_data);
    let sensed_food = best_idx_f.map(|idx| (idx, dx_f, dy_f, f_type));
    let nearby_count = ctx
        .spatial_hash
        .count_nearby(pos.x, pos.y, eff_sensing_range);
    let (ph_f, tribe_d, sa, sb) = ctx
        .pheromones
        .sense_all(pos.x, pos.y, eff_sensing_range / 2.0);
    let (kx, ky) = ctx
        .spatial_hash
        .sense_kin(pos.x, pos.y, eff_sensing_range, met.lineage_id);
    let wall_dist = ctx.terrain.sense_wall(pos.x, pos.y, 5.0);
    let age_ratio = (ctx.tick - met.birth_tick) as f32 / 2000.0;
    let sound_sense = ctx.sound.sense(pos.x, pos.y, eff_sensing_range);
    let mut partner_energy = 0.0;
    if let Some(p_id) = intel.bonded_to {
        if let Some(&p_idx) = id_map.get(&p_id) {
            partner_energy = (ctx.snapshots[p_idx].energy / met.max_energy.max(1.0)) as f32;
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
    EntityDecision {
        outputs,
        nearby_count,
        grn_speed_mod: speed_mod,
        grn_sensing_mod: sensing_mod,
        grn_repro_mod: repro_mod,
        sensed_food,
    }
}
