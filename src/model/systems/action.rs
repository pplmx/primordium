//! Action system - handles movement, energy costs, and pheromone deposits.

use crate::model::config::AppConfig;
use crate::model::environment::Environment;
use crate::model::terrain::TerrainGrid;
use primordium_data::{Entity, Intel, Metabolism, Physics};
use std::collections::HashMap;

/// Context for action operations, reducing parameter count.
pub struct ActionContext<'a> {
    pub env: &'a Environment,
    pub config: &'a AppConfig,
    pub terrain: &'a TerrainGrid,
    pub snapshots: &'a [crate::model::world::InternalEntitySnapshot],
    pub entity_id_map: &'a HashMap<uuid::Uuid, usize>,
    pub spatial_hash: &'a crate::model::spatial_hash::SpatialHash,
    pub pressure: &'a crate::model::pressure::PressureGrid,
    pub width: u16,
    pub height: u16,
}

pub struct ActionOutput {
    pub pheromones: Vec<crate::model::pheromone::PheromoneDeposit>,
    pub sounds: Vec<crate::model::sound::SoundDeposit>,
    pub pressure: Vec<crate::model::pressure::PressureDeposit>,
    pub oxygen_drain: f64,
    pub overmind_broadcast: Option<(uuid::Uuid, f32)>,
}

impl Default for ActionOutput {
    fn default() -> Self {
        Self {
            pheromones: Vec::with_capacity(2),
            sounds: Vec::with_capacity(1),
            pressure: Vec::with_capacity(2),
            oxygen_drain: 0.0,
            overmind_broadcast: None,
        }
    }
}

pub fn action_system_components(
    id: &uuid::Uuid,
    physics: &mut Physics,
    metabolism: &mut Metabolism,
    intel: &mut Intel,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    let speed_cap = physics.max_speed;
    let speed_mult = (1.0 + (outputs[2] as f64 + 1.0) / 2.0) * speed_cap;
    let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;

    intel.last_aggression = (outputs[3] + 1.0) / 2.0;
    intel.last_share_intent = (outputs[4] + 1.0) / 2.0;
    intel.last_signal = outputs[5];
    intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

    let stomach_penalty = (metabolism.max_energy - 200.0).max(0.0) / 1000.0;
    let inertia = (0.8 + stomach_penalty).clamp(0.4, 0.95);
    physics.vx = physics.vx * inertia + (outputs[0] as f64) * (1.0 - inertia);
    physics.vy = physics.vy * inertia + (outputs[1] as f64) * (1.0 - inertia);

    let metabolism_mult = ctx.env.metabolism_multiplier();
    let oxygen_factor = (ctx.env.oxygen_level / 21.0).max(0.1);
    let aerobic_boost = oxygen_factor.sqrt();

    let activity_drain = (speed_mult - 1.0).max(0.0) * 0.01;

    let cell = ctx.terrain.get(physics.x, physics.y);
    let local_cooling = cell.local_cooling;

    let effective_metabolism_mult = if metabolism_mult > 1.0 {
        1.0 + (metabolism_mult - 1.0) * (1.0 - local_cooling as f64 * 0.8).max(0.0)
    } else {
        metabolism_mult
    };

    let mut move_cost =
        ctx.config.metabolism.base_move_cost * effective_metabolism_mult * speed_mult
            / aerobic_boost;

    if predation_mode {
        move_cost *= 2.0;
    }

    let signal_cost = outputs[5].abs() as f64 * ctx.config.social.sharing_fraction * 2.0;
    let brain_maintenance = (intel.genotype.brain.nodes.len() as f64
        * ctx.config.brain.hidden_node_cost)
        + (intel.genotype.brain.connections.len() as f64 * ctx.config.brain.connection_cost);

    let mut base_idle = ctx.config.metabolism.base_idle_cost;

    if intel
        .ancestral_traits
        .contains(&primordium_data::AncestralTrait::HardenedMetabolism)
    {
        base_idle *= 0.8;
    }

    if matches!(cell.terrain_type, crate::model::terrain::TerrainType::Nest) {
        base_idle *= 1.0 - ctx.config.ecosystem.corpse_fertility_mult as f64;
    }

    let mut idle_cost = (base_idle + brain_maintenance) * effective_metabolism_mult;

    if intel.bonded_to.is_some() {
        move_cost *= 0.9;
        idle_cost *= 0.9;
    }

    if ctx.env.oxygen_level < 8.0 {
        idle_cost += 1.0;
    }

    if let Some(partner_id) = intel.bonded_to {
        if let Some(partner) = ctx
            .entity_id_map
            .get(&partner_id)
            .map(|&idx| &ctx.snapshots[idx])
        {
            let dx = partner.x - physics.x;
            let dy = partner.y - physics.y;
            let dist = (dx * dx + dy * dy).sqrt();

            let k = ctx.config.brain.coupling_spring_constant;
            let rest_length = 2.0;

            if dist > rest_length {
                let force = (dist - rest_length) * k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                physics.vx += fx;
                physics.vy += fy;
            }
        }
    }

    if intel.bonded_to.is_none() {
        let mut best_alpha_pos = None;
        let mut max_rank = intel.rank;

        ctx.spatial_hash
            .query_callback(physics.x, physics.y, physics.sensing_range, |idx| {
                let s = &ctx.snapshots[idx];
                if s.id != *id && s.lineage_id == metabolism.lineage_id {
                    let dx = s.x - physics.x;
                    let dy = s.y - physics.y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < physics.sensing_range.powi(2) && s.rank > max_rank + 0.1 {
                        max_rank = s.rank;
                        best_alpha_pos = Some((s.x, s.y));
                    }
                }
            });

        if let Some((ax, ay)) = best_alpha_pos {
            let dx = ax - physics.x;
            let dy = ay - physics.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            physics.vx += (dx / dist) * ctx.config.brain.alpha_following_force;
            physics.vy += (dy / dist) * ctx.config.brain.alpha_following_force;
        }
    }

    metabolism.energy -= move_cost + idle_cost + signal_cost;

    if outputs[6] > 0.5 {
        output
            .pheromones
            .push(crate::model::pheromone::PheromoneDeposit {
                x: physics.x,
                y: physics.y,
                ptype: crate::model::pheromone::PheromoneType::SignalA,
                amount: 0.5,
            });
    }
    if outputs[7] > 0.5 {
        output
            .pheromones
            .push(crate::model::pheromone::PheromoneDeposit {
                x: physics.x,
                y: physics.y,
                ptype: crate::model::pheromone::PheromoneType::SignalB,
                amount: 0.5,
            });
    }

    if intel.last_vocalization > 0.1 {
        output.sounds.push(crate::model::sound::SoundDeposit {
            x: physics.x,
            y: physics.y,
            amount: intel.last_vocalization,
        });
    }

    if outputs[9] > 0.5 {
        output
            .pressure
            .push(crate::model::pressure::PressureDeposit {
                x: physics.x,
                y: physics.y,
                ptype: crate::model::pressure::PressureType::DigDemand,
                amount: outputs[9],
            });
    }
    if outputs[10] > 0.5 {
        output
            .pressure
            .push(crate::model::pressure::PressureDeposit {
                x: physics.x,
                y: physics.y,
                ptype: crate::model::pressure::PressureType::BuildDemand,
                amount: outputs[10],
            });
    }

    if let Some(spec) = intel.specialization {
        if spec == primordium_data::Specialization::Engineer {
            let is_near_river = ctx.terrain.has_neighbor_type(
                physics.x as u16,
                physics.y as u16,
                crate::model::terrain::TerrainType::River,
            );
            if is_near_river
                && matches!(
                    cell.terrain_type,
                    crate::model::terrain::TerrainType::Plains
                        | crate::model::terrain::TerrainType::Desert
                )
            {
                println!(
                    "DEBUG: Entity at ({}, {}) depositing DigDemand",
                    physics.x, physics.y
                );
                output
                    .pressure
                    .push(crate::model::pressure::PressureDeposit {
                        x: physics.x,
                        y: physics.y,
                        ptype: crate::model::pressure::PressureType::DigDemand,
                        amount: 0.8,
                    });
            }
        }
    }

    if outputs[11] > 0.5 && intel.rank > 0.8 {
        output.overmind_broadcast = Some((metabolism.lineage_id, outputs[11]));
    }

    handle_movement_components(physics, speed_mult, ctx.terrain, ctx.width, ctx.height);
    output.oxygen_drain = activity_drain;
}

pub fn action_system(
    entity: &mut Entity,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    action_system_components(
        &entity.identity.id,
        &mut entity.physics,
        &mut entity.metabolism,
        &mut entity.intel,
        outputs,
        ctx,
        output,
    );
}

pub fn handle_movement_components(
    physics: &mut Physics,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    let terrain_mod = terrain.movement_modifier(physics.x, physics.y);
    let effective_speed = speed * terrain_mod;
    let next_x = physics.x + physics.vx * effective_speed;
    let next_y = physics.y + physics.vy * effective_speed;

    let cell_type = terrain.get_cell(next_x as u16, next_y as u16).terrain_type;
    if matches!(cell_type, crate::model::terrain::TerrainType::Wall) {
        physics.vx *= -1.0;
        physics.vy *= -1.0;
    } else {
        physics.x = next_x;
        physics.y = next_y;
    }

    if physics.x < 0.0 {
        physics.x = 0.0;
        physics.vx *= -1.0;
    } else if physics.x >= f64::from(width) {
        physics.x = f64::from(width) - 1.0;
        physics.vx *= -1.0;
    }
    if physics.y < 0.0 {
        physics.y = 0.0;
        physics.vy *= -1.0;
    } else if physics.y >= f64::from(height) {
        physics.y = f64::from(height) - 1.0;
        physics.vy *= -1.0;
    }
}

pub fn handle_movement(
    entity: &mut Entity,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    handle_movement_components(&mut entity.physics, speed, terrain, width, height);
}

pub fn handle_game_modes_ecs(
    world: &mut hecs::World,
    config: &AppConfig,
    tick: u64,
    width: u16,
    height: u16,
) {
    use crate::model::config::GameMode;
    if config.game_mode == GameMode::BattleRoyale {
        let shrink_speed = 0.01;
        let shrink_amount = (tick as f32 * shrink_speed).min(f32::from(width) / 2.0 - 5.0);
        let danger_radius_x = f32::from(width) / 2.0 - shrink_amount;
        let danger_radius_y = f32::from(height) / 2.0 - shrink_amount;
        let center_x = f32::from(width) / 2.0;
        let center_y = f32::from(height) / 2.0;

        for (_handle, (phys, met)) in world.query_mut::<(&Physics, &mut Metabolism)>() {
            let dx = (phys.x as f32 - center_x).abs();
            let dy = (phys.y as f32 - center_y).abs();
            if dx > danger_radius_x || dy > danger_radius_y {
                met.energy -= 5.0;
            }
        }
    }
}

pub fn handle_game_modes(
    entities: &mut [Entity],
    config: &AppConfig,
    tick: u64,
    width: u16,
    height: u16,
) {
    use crate::model::config::GameMode;
    if config.game_mode == GameMode::BattleRoyale {
        let shrink_speed = 0.01;
        let shrink_amount = (tick as f32 * shrink_speed).min(f32::from(width) / 2.0 - 5.0);
        let danger_radius_x = f32::from(width) / 2.0 - shrink_amount;
        let danger_radius_y = f32::from(height) / 2.0 - shrink_amount;
        let center_x = f32::from(width) / 2.0;
        let center_y = f32::from(height) / 2.0;

        for e in entities {
            let dx = (e.physics.x as f32 - center_x).abs();
            let dy = (e.physics.y as f32 - center_y).abs();
            if dx > danger_radius_x || dy > danger_radius_y {
                e.metabolism.energy -= 5.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::environment::Environment;
    #[test]
    fn test_action_system_energy_consumption() {
        let mut entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        entity.metabolism.energy = 100.0;
        let initial_energy = entity.metabolism.energy;
        let outputs = [0.0; 12];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::model::spatial_hash::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        let mut out = ActionOutput::default();
        action_system(&mut entity, outputs, &mut ctx, &mut out);
        assert!(entity.metabolism.energy < initial_energy);
    }
    #[test]
    fn test_action_system_predation_mode_higher_cost() {
        let mut entity_normal = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        let mut entity_predator = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        entity_normal.metabolism.energy = 100.0;
        entity_predator.metabolism.energy = 100.0;
        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::pressure::PressureGrid::new(20, 20);
        let mut ctx_n = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::model::spatial_hash::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        let mut out_n = ActionOutput::default();
        action_system(&mut entity_normal, normal_outputs, &mut ctx_n, &mut out_n);
        let mut ctx_p = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::model::spatial_hash::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        let mut out_p = ActionOutput::default();
        action_system(
            &mut entity_predator,
            predator_outputs,
            &mut ctx_p,
            &mut out_p,
        );
        assert!(entity_predator.metabolism.energy < entity_normal.metabolism.energy);
    }
    #[test]
    fn test_action_system_velocity_update() {
        let mut entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 0.0;
        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        terrain.set_cell_type(5, 5, crate::model::terrain::TerrainType::Plains);
        let pressure_grid = crate::model::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::model::spatial_hash::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        let mut out = ActionOutput::default();
        action_system(&mut entity, outputs, &mut ctx, &mut out);
        assert!(entity.physics.vx > 0.0);
        assert!(entity.physics.vy < 0.0);
    }
}
