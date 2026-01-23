//! Action system - handles entity movement, velocity updates, and game mode effects.

use crate::model::config::{AppConfig, GameMode};
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::terrain::{TerrainGrid, TerrainType};

/// Apply neural network outputs to entity movement and calculate energy costs.
pub fn action_system(
    entity: &mut Entity,
    outputs: [f32; 5],
    env: &Environment,
    config: &AppConfig,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    // 1. Calculate phenotypic modifiers
    let speed_cap = entity.physics.max_speed;
    let sensing_radius = entity.physics.sensing_range;
    let stomach_penalty = (entity.metabolism.max_energy - 200.0) / 1000.0;

    // 2. Speed and Aggression
    let speed_mult = (1.0 + (outputs[2] as f64 + 1.0) / 2.0) * speed_cap;
    let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
    entity.intel.last_aggression = (outputs[3] + 1.0) / 2.0;
    entity.intel.last_share_intent = (outputs[4] + 1.0) / 2.0;

    // 3. Inertia/Responsiveness based on stomach size
    let inertia = (0.8 - stomach_penalty).clamp(0.4, 0.85);
    entity.physics.vx = entity.physics.vx * inertia + (outputs[0] as f64) * (1.0 - inertia);
    entity.physics.vy = entity.physics.vy * inertia + (outputs[1] as f64) * (1.0 - inertia);

    // 4. Calculate Costs
    let metabolism_mult = env.metabolism_multiplier();
    // Move cost scales with speed capability
    let mut move_cost = config.metabolism.base_move_cost * metabolism_mult * speed_mult;
    if predation_mode {
        move_cost *= 2.0;
    }

    // Idle cost scales with sensing capability
    let sensing_cost_mod = 1.0 + (sensing_radius - 5.0).max(0.0) * 0.1;
    let idle_cost = config.metabolism.base_idle_cost * metabolism_mult * sensing_cost_mod;

    entity.metabolism.energy -= move_cost + idle_cost;
    handle_movement(entity, speed_mult, terrain, width, height);
}

/// Handle entity movement with terrain collision and wall bouncing.
pub fn handle_movement(
    entity: &mut Entity,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    let terrain_speed_mod = terrain.movement_modifier(entity.physics.x, entity.physics.y);
    let next_x = entity.physics.x + entity.physics.vx * speed * terrain_speed_mod;
    let next_y = entity.physics.y + entity.physics.vy * speed * terrain_speed_mod;
    let next_terrain = terrain.get(next_x, next_y);
    if next_terrain.terrain_type == TerrainType::Wall {
        entity.physics.vx = -entity.physics.vx;
        entity.physics.vy = -entity.physics.vy;
        return;
    }
    entity.physics.x = next_x;
    entity.physics.y = next_y;
    let width_f = f64::from(width);
    let height_f = f64::from(height);
    if entity.physics.x <= 0.0 {
        entity.physics.x = 0.0;
        entity.physics.vx = -entity.physics.vx;
    } else if entity.physics.x >= width_f {
        entity.physics.x = width_f - 0.1;
        entity.physics.vx = -entity.physics.vx;
    }
    if entity.physics.y <= 0.0 {
        entity.physics.y = 0.0;
        entity.physics.vy = -entity.physics.vy;
    } else if entity.physics.y >= height_f {
        entity.physics.y = height_f - 0.1;
        entity.physics.vy = -entity.physics.vy;
    }
}

/// Apply Battle Royale shrinking zone damage.
pub fn handle_game_modes(
    entities: &mut [Entity],
    config: &AppConfig,
    tick: u64,
    width: u16,
    height: u16,
) {
    if config.game_mode == GameMode::BattleRoyale {
        let width_f = f64::from(width);
        let height_f = f64::from(height);
        let shrink_amount = (tick as f64 / 100.0).min(width_f / 2.0 - 5.0);
        for e in entities {
            if e.physics.x < shrink_amount
                || e.physics.x > width_f - shrink_amount
                || e.physics.y < shrink_amount
                || e.physics.y > height_f - shrink_amount
            {
                e.metabolism.energy -= 10.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::config::AppConfig;
    use crate::model::state::entity::Entity;
    use crate::model::state::environment::Environment;
    use crate::model::state::terrain::TerrainGrid;

    #[test]
    fn test_handle_movement_boundary_bounce() {
        let terrain = TerrainGrid::generate(10, 10, 42);
        let mut entity = Entity::new(0.5, 0.5, 0);
        entity.physics.vx = -1.0;
        entity.physics.vy = -1.0;

        handle_movement(&mut entity, 1.0, &terrain, 10, 10);

        assert!(
            entity.physics.vx > 0.0,
            "Velocity X should be reversed at left boundary"
        );
        assert!(
            entity.physics.vy > 0.0,
            "Velocity Y should be reversed at top boundary"
        );
    }

    #[test]
    fn test_handle_movement_wall_collision() {
        let mut terrain = TerrainGrid::generate(10, 10, 42);
        terrain.set_cell_type(5, 5, TerrainType::Wall);

        let mut entity = Entity::new(4.5, 4.5, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 1.0;

        handle_movement(&mut entity, 1.0, &terrain, 10, 10);

        assert!(
            entity.physics.vx < 0.0,
            "Velocity X should be reversed on wall hit"
        );
        assert!(
            entity.physics.vy < 0.0,
            "Velocity Y should be reversed on wall hit"
        );
        assert_eq!(entity.physics.x, 4.5, "Entity should not move into wall");
    }

    #[test]
    fn test_handle_movement_right_boundary() {
        let terrain = TerrainGrid::generate(10, 10, 42);
        let mut entity = Entity::new(9.5, 5.0, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 0.0;

        handle_movement(&mut entity, 1.0, &terrain, 10, 10);

        assert!(
            entity.physics.vx < 0.0,
            "Velocity should be reversed at right boundary"
        );
    }

    #[test]
    fn test_handle_movement_bottom_boundary() {
        let terrain = TerrainGrid::generate(10, 10, 42);
        let mut entity = Entity::new(5.0, 9.5, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 1.0;

        handle_movement(&mut entity, 1.0, &terrain, 10, 10);

        assert!(
            entity.physics.vy < 0.0,
            "Velocity should be reversed at bottom boundary"
        );
    }

    #[test]
    fn test_action_system_energy_consumption() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.energy = 100.0;
        let initial_energy = entity.metabolism.energy;

        let outputs = [0.0, 0.0, 0.0, 0.0, 0.0]; // Neutral outputs
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);

        action_system(&mut entity, outputs, &env, &config, &terrain, 20, 20);

        assert!(
            entity.metabolism.energy < initial_energy,
            "Energy should decrease after action"
        );
    }

    #[test]
    fn test_action_system_predation_mode_higher_cost() {
        let mut entity_normal = Entity::new(5.0, 5.0, 0);
        let mut entity_predator = Entity::new(5.0, 5.0, 0);
        entity_normal.metabolism.energy = 100.0;
        entity_predator.metabolism.energy = 100.0;

        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0]; // Low aggression
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0]; // High aggression
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);

        action_system(
            &mut entity_normal,
            normal_outputs,
            &env,
            &config,
            &terrain,
            20,
            20,
        );
        action_system(
            &mut entity_predator,
            predator_outputs,
            &env,
            &config,
            &terrain,
            20,
            20,
        );

        assert!(
            entity_predator.metabolism.energy < entity_normal.metabolism.energy,
            "Predation mode should cost more energy"
        );
    }

    #[test]
    fn test_action_system_velocity_update() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 0.0;

        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0]; // Move right-up
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        // Ensure the test area is clear of walls to prevent velocity reversal
        terrain.set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Plains);
        terrain.set_cell_type(6, 5, crate::model::state::terrain::TerrainType::Plains);
        terrain.set_cell_type(5, 4, crate::model::state::terrain::TerrainType::Plains);

        action_system(&mut entity, outputs, &env, &config, &terrain, 20, 20);

        assert!(entity.physics.vx > 0.0, "Velocity X should be positive");
        assert!(entity.physics.vy < 0.0, "Velocity Y should be negative");
    }

    #[test]
    fn test_handle_game_modes_battle_royale_damage() {
        let config = AppConfig {
            game_mode: GameMode::BattleRoyale,
            ..Default::default()
        };

        // Use width=20 so shrink_amount can be > 0
        // At tick 100, shrink_amount = min(1.0, 20/2 - 5) = min(1.0, 5.0) = 1.0
        let mut entities = vec![
            Entity::new(0.5, 10.0, 0),  // In the danger zone (x < 1.0)
            Entity::new(10.0, 10.0, 0), // Center (safe)
        ];
        entities[0].metabolism.energy = 100.0;
        entities[1].metabolism.energy = 100.0;

        handle_game_modes(&mut entities, &config, 100, 20, 20);

        assert!(
            entities[0].metabolism.energy < 100.0,
            "Entity in danger zone should take damage"
        );
        assert_eq!(
            entities[1].metabolism.energy, 100.0,
            "Entity in center should be safe"
        );
    }

    #[test]
    fn test_handle_game_modes_standard_no_damage() {
        let config = AppConfig {
            game_mode: GameMode::Standard,
            ..Default::default()
        };

        let mut entities = vec![Entity::new(1.0, 1.0, 0)];
        entities[0].metabolism.energy = 100.0;

        handle_game_modes(&mut entities, &config, 1000, 10, 10);

        assert_eq!(
            entities[0].metabolism.energy, 100.0,
            "Standard mode should not damage entities"
        );
    }
}
