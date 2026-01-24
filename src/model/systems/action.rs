//! Action system - handles movement, energy costs, and pheromone deposits.

use crate::model::config::AppConfig;
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::terrain::TerrainGrid;

/// Context for action operations, reducing parameter count.
pub struct ActionContext<'a> {
    pub env: &'a Environment,
    pub config: &'a AppConfig,
    pub terrain: &'a TerrainGrid,
    pub pheromones: &'a mut crate::model::state::pheromone::PheromoneGrid,
    pub width: u16,
    pub height: u16,
}

/// Process brain outputs and apply movement and metabolic costs.
pub fn action_system(entity: &mut Entity, outputs: [f32; 9], ctx: &mut ActionContext) {
    let speed_cap = entity.physics.max_speed;
    let speed_mult = (1.0 + (outputs[2] as f64 + 1.0) / 2.0) * speed_cap;
    let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;

    entity.intel.last_aggression = (outputs[3] + 1.0) / 2.0;
    entity.intel.last_share_intent = (outputs[4] + 1.0) / 2.0;
    entity.intel.last_signal = outputs[5];
    entity.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

    let stomach_penalty = (entity.metabolism.max_energy - 200.0).max(0.0) / 1000.0;
    let inertia = (0.8 + stomach_penalty).clamp(0.4, 0.95);
    entity.physics.vx = entity.physics.vx * inertia + (outputs[0] as f64) * (1.0 - inertia);
    entity.physics.vy = entity.physics.vy * inertia + (outputs[1] as f64) * (1.0 - inertia);

    let metabolism_mult = ctx.env.metabolism_multiplier();
    let mut move_cost = ctx.config.metabolism.base_move_cost * metabolism_mult * speed_mult;
    if predation_mode {
        move_cost *= 2.0;
    }

    let signal_cost = outputs[5].abs() as f64 * 0.1;
    let brain_maintenance = (entity.intel.genotype.brain.nodes.len() as f64 * 0.02)
        + (entity.intel.genotype.brain.connections.len() as f64 * 0.005);
    let idle_cost = (ctx.config.metabolism.base_idle_cost + brain_maintenance) * metabolism_mult;

    entity.metabolism.energy -= move_cost + idle_cost + signal_cost;

    if outputs[6] > 0.5 {
        ctx.pheromones.deposit(
            entity.physics.x,
            entity.physics.y,
            crate::model::state::pheromone::PheromoneType::SignalA,
            0.5,
        );
    }
    if outputs[7] > 0.5 {
        ctx.pheromones.deposit(
            entity.physics.x,
            entity.physics.y,
            crate::model::state::pheromone::PheromoneType::SignalB,
            0.5,
        );
    }

    handle_movement(entity, speed_mult, ctx.terrain, ctx.width, ctx.height);
}

pub fn handle_movement(
    entity: &mut Entity,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    let terrain_mod = terrain.movement_modifier(entity.physics.x, entity.physics.y);
    let effective_speed = speed * terrain_mod;
    let next_x = entity.physics.x + entity.physics.vx * effective_speed;
    let next_y = entity.physics.y + entity.physics.vy * effective_speed;

    let cell_type = terrain.get_cell(next_x as u16, next_y as u16).terrain_type;
    if matches!(cell_type, crate::model::state::terrain::TerrainType::Wall) {
        entity.physics.vx *= -1.0;
        entity.physics.vy *= -1.0;
    } else {
        entity.physics.x = next_x;
        entity.physics.y = next_y;
    }

    if entity.physics.x < 0.0 {
        entity.physics.x = 0.0;
        entity.physics.vx *= -1.0;
    } else if entity.physics.x >= f64::from(width) {
        entity.physics.x = f64::from(width) - 1.0;
        entity.physics.vx *= -1.0;
    }
    if entity.physics.y < 0.0 {
        entity.physics.y = 0.0;
        entity.physics.vy *= -1.0;
    } else if entity.physics.y >= f64::from(height) {
        entity.physics.y = f64::from(height) - 1.0;
        entity.physics.vy *= -1.0;
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
    use crate::model::state::environment::Environment;
    #[test]
    fn test_action_system_energy_consumption() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.energy = 100.0;
        let initial_energy = entity.metabolism.energy;
        let outputs = [0.0; 9];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let mut pheromones = crate::model::state::pheromone::PheromoneGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            pheromones: &mut pheromones,
            width: 20,
            height: 20,
        };
        action_system(&mut entity, outputs, &mut ctx);
        assert!(entity.metabolism.energy < initial_energy);
    }
    #[test]
    fn test_action_system_predation_mode_higher_cost() {
        let mut entity_normal = Entity::new(5.0, 5.0, 0);
        let mut entity_predator = Entity::new(5.0, 5.0, 0);
        entity_normal.metabolism.energy = 100.0;
        entity_predator.metabolism.energy = 100.0;
        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let mut pheromones = crate::model::state::pheromone::PheromoneGrid::new(20, 20);
        let mut ctx_n = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            pheromones: &mut pheromones,
            width: 20,
            height: 20,
        };
        action_system(&mut entity_normal, normal_outputs, &mut ctx_n);
        let mut ctx_p = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            pheromones: &mut pheromones,
            width: 20,
            height: 20,
        };
        action_system(&mut entity_predator, predator_outputs, &mut ctx_p);
        assert!(entity_predator.metabolism.energy < entity_normal.metabolism.energy);
    }
    #[test]
    fn test_action_system_velocity_update() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 0.0;
        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        let mut pheromones = crate::model::state::pheromone::PheromoneGrid::new(20, 20);
        terrain.set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Plains);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            pheromones: &mut pheromones,
            width: 20,
            height: 20,
        };
        action_system(&mut entity, outputs, &mut ctx);
        assert!(entity.physics.vx > 0.0);
        assert!(entity.physics.vy < 0.0);
    }
}
