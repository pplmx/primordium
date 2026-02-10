use crate::model::environment::Environment;
use crate::model::world::{EntityComponents, EntityDecision, SystemContext};
use primordium_core::pheromone::PheromoneGrid;
use primordium_core::pressure::PressureGrid;
use primordium_core::sound::SoundGrid;
use primordium_core::systems::action;
use rayon::prelude::*;
use std::collections::HashMap;

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

                let mut action_entity = action::ActionEntity {
                    id: &identity.id,
                    position: pos,
                    velocity,
                    physics: phys,
                    metabolism: met,
                    intel,
                    health: _health,
                };
                action::action_system_components_with_modifiers(
                    &mut action_entity,
                    eff_max_speed,
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
