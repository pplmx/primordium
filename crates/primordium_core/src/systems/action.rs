use crate::config::AppConfig;
use crate::environment::Environment;
use crate::terrain::TerrainGrid;
use primordium_data::{Entity, Health, Intel, Metabolism, Physics, Specialization};
use std::collections::HashMap;

pub struct ActionContext<'a> {
    pub env: &'a Environment,
    pub config: &'a AppConfig,
    pub terrain: &'a TerrainGrid,
    pub influence: &'a crate::influence::InfluenceGrid,
    pub snapshots: &'a [crate::snapshot::InternalEntitySnapshot],
    pub entity_id_map: &'a HashMap<uuid::Uuid, usize>,
    pub spatial_hash: &'a crate::spatial_hash::SpatialHash,
    pub pressure: &'a crate::pressure::PressureGrid,
    pub width: u16,
    pub height: u16,
}

pub struct ActionOutput {
    pub pheromones: Vec<crate::pheromone::PheromoneDeposit>,
    pub sounds: Vec<crate::sound::SoundDeposit>,
    pub pressure: Vec<crate::pressure::PressureDeposit>,
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

#[allow(clippy::too_many_arguments)]
pub fn action_system_components_with_modifiers(
    id: &uuid::Uuid,
    position: &mut primordium_data::Position,
    velocity: &mut primordium_data::Velocity,
    physics: &Physics,
    eff_max_speed: f64,
    metabolism: &mut Metabolism,
    intel: &mut Intel,
    _health: &mut Health,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    let speed_mult = (1.0 + f64::midpoint(f64::from(outputs[2]), 1.0)) * eff_max_speed;
    let predation_mode = f64::midpoint(f64::from(outputs[3]), 1.0) > 0.5;

    intel.last_aggression = f32::midpoint(outputs[3], 1.0);
    intel.last_share_intent = f32::midpoint(outputs[4], 1.0);
    intel.last_signal = outputs[5];
    intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

    let stomach_penalty = (metabolism.max_energy - 200.0).max(0.0) / 1000.0;
    let inertia = (0.8 + stomach_penalty).clamp(0.4, 0.95);
    velocity.vx = velocity.vx * inertia + f64::from(outputs[0]) * (1.0 - inertia);
    velocity.vy = velocity.vy * inertia + f64::from(outputs[1]) * (1.0 - inertia);

    let metabolism_mult = ctx.env.metabolism_multiplier();
    let oxygen_factor = (ctx.env.oxygen_level / 21.0).max(0.1);
    let aerobic_boost = oxygen_factor.sqrt();

    let activity_drain = (speed_mult - 1.0).max(0.0) * 0.01;

    let cell = ctx.terrain.get(position.x, position.y);
    let local_cooling = cell.local_cooling;

    let effective_metabolism_mult = if metabolism_mult > 1.0 {
        1.0 + (metabolism_mult - 1.0) * (1.0 - f64::from(local_cooling) * 0.8).max(0.0)
    } else {
        metabolism_mult
    };

    let mut move_cost =
        ctx.config.metabolism.base_move_cost * effective_metabolism_mult * speed_mult
            / aerobic_boost;

    if predation_mode {
        move_cost *= 2.0;
    }

    let signal_cost = f64::from(outputs[5].abs()) * ctx.config.social.sharing_fraction * 2.0;
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

    if matches!(cell.terrain_type, primordium_data::TerrainType::Nest) {
        base_idle *= 1.0 - f64::from(ctx.config.ecosystem.corpse_fertility_mult);
    }

    let mut idle_cost = (base_idle + brain_maintenance) * effective_metabolism_mult;

    if intel.bonded_to.is_some() {
        move_cost *= 0.9;
        idle_cost *= 0.9;
    }

    if ctx.env.oxygen_level < 8.0 {
        idle_cost += 1.0;
    }

    let (dom_l, intensity) = ctx.influence.get_influence(position.x, position.y);
    if let Some(lid) = dom_l {
        if lid != metabolism.lineage_id {
            idle_cost += f64::from(intensity) * 0.1;
        }
    }

    let total_cost = move_cost + signal_cost + idle_cost + activity_drain;

    metabolism.energy -= total_cost;

    if let Some(partner_id) = intel.bonded_to {
        if let Some(partner) = ctx
            .entity_id_map
            .get(&partner_id)
            .map(|&idx| &ctx.snapshots[idx])
        {
            let dx = partner.x - position.x;
            let dy = partner.y - position.y;
            let dist = (dx * dx + dy * dy).sqrt();

            let k = ctx.config.brain.coupling_spring_constant;
            let rest_length = 2.0;

            if dist > rest_length {
                let force = (dist - rest_length) * k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                velocity.vx += fx;
                velocity.vy += fy;
            }
        }
    }

    if intel.bonded_to.is_none() {
        let mut best_alpha_pos = None;
        let mut max_rank = intel.rank;

        ctx.spatial_hash
            .query_callback(position.x, position.y, physics.sensing_range, |idx| {
                let s = &ctx.snapshots[idx];
                if s.id != *id && s.lineage_id == metabolism.lineage_id {
                    let dx = s.x - position.x;
                    let dy = s.y - position.y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < physics.sensing_range.powi(2) && s.rank > max_rank + 0.1 {
                        max_rank = s.rank;
                        best_alpha_pos = Some((s.x, s.y));
                    }
                }
            });

        if let Some((ax, ay)) = best_alpha_pos {
            let dx = ax - position.x;
            let dy = ay - position.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            velocity.vx = velocity.vx * 0.7 + (dx / dist) * 0.3;
            velocity.vy = velocity.vy * 0.7 + (dy / dist) * 0.3;
        }
    }

    if outputs[6].abs() > 0.1 {
        output.sounds.push(crate::sound::SoundDeposit {
            x: position.x,
            y: position.y,
            amount: outputs[6].abs(),
        });
        output.pheromones.push(crate::pheromone::PheromoneDeposit {
            x: position.x,
            y: position.y,
            ptype: crate::pheromone::PheromoneType::SignalA,
            amount: outputs[6].abs(),
        });
    }

    if outputs[7].abs() > 0.1 {
        output.sounds.push(crate::sound::SoundDeposit {
            x: position.x,
            y: position.y,
            amount: outputs[7].abs(),
        });
        output.pheromones.push(crate::pheromone::PheromoneDeposit {
            x: position.x,
            y: position.y,
            ptype: crate::pheromone::PheromoneType::SignalB,
            amount: outputs[7].abs(),
        });
    }

    if outputs[9] > 0.5 {
        output.pressure.push(crate::pressure::PressureDeposit {
            x: position.x,
            y: position.y,
            ptype: crate::pressure::PressureType::DigDemand,
            amount: outputs[9],
        });
    }
    if outputs[10] > 0.5 {
        output.pressure.push(crate::pressure::PressureDeposit {
            x: position.x,
            y: position.y,
            ptype: crate::pressure::PressureType::BuildDemand,
            amount: outputs[10],
        });
    }

    if let Some(spec) = intel.specialization {
        if spec == Specialization::Engineer {
            let is_near_river = ctx.terrain.has_neighbor_type(
                position.x as u16,
                position.y as u16,
                crate::terrain::TerrainType::River,
            );

            let (tx, ty, max_press) =
                ctx.pressure
                    .find_highest_in_range(position.x, position.y, physics.sensing_range);

            if max_press > 0.2 {
                let dx = tx - position.x;
                let dy = ty - position.y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let attr_force = if is_near_river { 0.3 } else { 0.15 };
                velocity.vx += (dx / dist) * attr_force;
                velocity.vy += (dy / dist) * attr_force;
            }

            if is_near_river
                && matches!(
                    cell.terrain_type,
                    primordium_data::TerrainType::Plains | primordium_data::TerrainType::Desert
                )
            {
                output.pressure.push(crate::pressure::PressureDeposit {
                    x: position.x,
                    y: position.y,
                    ptype: crate::pressure::PressureType::DigDemand,
                    amount: 0.8,
                });
            }
        }
    }

    if outputs[11] > 0.8 && intel.rank > 0.8 {
        output.overmind_broadcast = Some((*id, outputs[11]));
    }

    handle_movement_components(
        position,
        velocity,
        physics,
        speed_mult,
        ctx.terrain,
        ctx.width,
        ctx.height,
    );
    output.oxygen_drain = activity_drain;
}

#[allow(clippy::too_many_arguments)]
pub fn action_system_components(
    id: &uuid::Uuid,
    position: &mut primordium_data::Position,
    velocity: &mut primordium_data::Velocity,
    physics: &Physics,
    metabolism: &mut Metabolism,
    intel: &mut Intel,
    health: &mut Health,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    action_system_components_with_modifiers(
        id,
        position,
        velocity,
        physics,
        physics.max_speed,
        metabolism,
        intel,
        health,
        outputs,
        ctx,
        output,
    );
}

pub fn handle_movement(
    entity: &mut Entity,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    handle_movement_components(
        &mut entity.position,
        &mut entity.velocity,
        &entity.physics,
        speed,
        terrain,
        width,
        height,
    );
}

pub fn handle_movement_components(
    position: &mut primordium_data::Position,
    velocity: &mut primordium_data::Velocity,
    _physics: &Physics,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    let next_x = position.x + velocity.vx * speed;
    let next_y = position.y + velocity.vy * speed;

    if terrain.get(next_x, next_y).terrain_type == primordium_data::TerrainType::Wall {
        velocity.vx *= -0.5;
        velocity.vy *= -0.5;
    } else {
        position.x = next_x;
        position.y = next_y;
    }

    if position.x < 0.0 {
        position.x = 0.0;
        velocity.vx *= -1.0;
    } else if position.x >= width as f64 {
        position.x = width as f64 - 0.1;
        velocity.vx *= -1.0;
    }

    if position.y < 0.0 {
        position.y = 0.0;
        velocity.vy *= -1.0;
    } else if position.y >= height as f64 {
        position.y = height as f64 - 0.1;
        velocity.vy *= -1.0;
    }
}

pub fn handle_game_modes_ecs(
    world: &mut hecs::World,
    config: &AppConfig,
    tick: u64,
    width: u16,
    height: u16,
) {
    use crate::config::GameMode;
    if config.game_mode == GameMode::BattleRoyale {
        let shrink_speed = 0.01;
        let shrink_amount = (tick as f32 * shrink_speed).min(f32::from(width) / 2.0 - 5.0);
        let danger_radius_x = f32::from(width) / 2.0 - shrink_amount;
        let danger_radius_y = f32::from(height) / 2.0 - shrink_amount;
        let center_x = f32::from(width) / 2.0;
        let center_y = f32::from(height) / 2.0;

        for (_handle, (pos, met)) in
            world.query_mut::<(&primordium_data::Position, &mut Metabolism)>()
        {
            let dx = (pos.x as f32 - center_x).abs();
            let dy = (pos.y as f32 - center_y).abs();
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
    use crate::config::GameMode;
    if config.game_mode == GameMode::BattleRoyale {
        let shrink_speed = 0.01;
        let shrink_amount = (tick as f32 * shrink_speed).min(f32::from(width) / 2.0 - 5.0);
        let danger_radius_x = f32::from(width) / 2.0 - shrink_amount;
        let danger_radius_y = f32::from(height) / 2.0 - shrink_amount;
        let center_x = f32::from(width) / 2.0;
        let center_y = f32::from(height) / 2.0;

        for e in entities {
            let dx = (e.position.x as f32 - center_x).abs();
            let dy = (e.position.y as f32 - center_y).abs();
            if dx > danger_radius_x || dy > danger_radius_y {
                e.metabolism.energy -= 5.0;
            }
        }
    }
}

pub fn action_system(
    entity: &mut Entity,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    action_system_components(
        &entity.identity.id,
        &mut entity.position,
        &mut entity.velocity,
        &entity.physics,
        &mut entity.metabolism,
        &mut entity.intel,
        &mut entity.health,
        outputs,
        ctx,
        output,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::environment::Environment;
    use crate::terrain::TerrainGrid;

    #[test]
    fn test_action_system_energy_consumption() {
        let mut entity = crate::lifecycle::create_entity(5.0, 5.0, 0);
        entity.metabolism.energy = 100.0;
        let initial_energy = entity.metabolism.energy;
        let outputs = [0.0; 12];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::pressure::PressureGrid::new(20, 20);
        let influence = crate::influence::InfluenceGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            influence: &influence,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::spatial_hash::SpatialHash::new(5.0, 20, 20),
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
        let mut entity_normal = crate::lifecycle::create_entity(5.0, 5.0, 0);
        let mut entity_predator = crate::lifecycle::create_entity(5.0, 5.0, 0);
        entity_normal.metabolism.energy = 100.0;
        entity_predator.metabolism.energy = 100.0;
        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::pressure::PressureGrid::new(20, 20);
        let influence = crate::influence::InfluenceGrid::new(20, 20);
        let mut ctx_n = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            influence: &influence,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::spatial_hash::SpatialHash::new(5.0, 20, 20),
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
            influence: &influence,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::spatial_hash::SpatialHash::new(5.0, 20, 20),
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
        let mut entity = crate::lifecycle::create_entity(5.0, 5.0, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 0.0;
        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        terrain.set_cell_type(5, 5, crate::terrain::TerrainType::Plains);
        let pressure_grid = crate::pressure::PressureGrid::new(20, 20);
        let influence = crate::influence::InfluenceGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            influence: &influence,
            snapshots: &[],
            entity_id_map: &HashMap::new(),
            spatial_hash: &crate::spatial_hash::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        let mut out = ActionOutput::default();
        action_system(&mut entity, outputs, &mut ctx, &mut out);
        assert!(entity.velocity.vx > 0.0);
        assert!(entity.velocity.vy < 0.0);
    }
}
