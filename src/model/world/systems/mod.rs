pub mod action_parallel;
pub mod commands;
pub mod perception;

use crate::model::environment::Environment;
use crate::model::world::{EntityComponents, EntityDecision, SystemContext};
use primordium_core::interaction::InteractionCommand;
use rayon::prelude::*;
use std::collections::HashMap;

pub use action_parallel::{apply_actions_sequential, calculate_actions_parallel};
pub use commands::generate_commands_for_entity;
pub use perception::perceive_one_entity;

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
        .enumerate()
        .for_each(
            |(_i, ((_handle, (identity, pos, _vel, phys, met, intel, health)), decision))| {
                let input = perception::EntityPerceptionInput {
                    identity,
                    pos,
                    phys,
                    met,
                    intel,
                    health,
                };
                *decision = perception::perceive_one_entity(input, ctx, env, id_map);
            },
        );

    let all_cmds_flat: Vec<InteractionCommand> = entity_data
        .par_iter_mut()
        .enumerate()
        .fold(
            Vec::new,
            |mut acc, (i, (_handle, (identity, pos, _vel, phys, met, intel, health)))| {
                let decision = &decision_buffer[i];
                let input = commands::EntityCommandInput {
                    i,
                    identity,
                    pos,
                    phys,
                    met,
                    intel,
                    health,
                    decision,
                };
                let cmds = commands::generate_commands_for_entity(
                    input, ctx, env, biomass_c, id_map, pop_len,
                );
                acc.extend(cmds);
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
