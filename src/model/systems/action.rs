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
    pub snapshots: &'a [crate::model::world::InternalEntitySnapshot],
    pub entity_id_map: &'a std::collections::HashMap<uuid::Uuid, usize>,
    pub spatial_hash: &'a crate::model::quadtree::SpatialHash,
    pub pressure: &'a crate::model::state::pressure::PressureGrid,
    pub width: u16,
    pub height: u16,
}

pub struct ActionResult {
    pub pheromones: Vec<crate::model::state::pheromone::PheromoneDeposit>,
    pub sounds: Vec<crate::model::state::sound::SoundDeposit>,
    pub pressure: Vec<crate::model::state::pressure::PressureDeposit>,
    pub oxygen_drain: f64,
    pub overmind_broadcast: Option<(uuid::Uuid, f32)>,
}

/// Process brain outputs and apply movement and metabolic costs.
pub fn action_system(
    entity: &mut Entity,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
) -> ActionResult {
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
    let oxygen_factor = (ctx.env.oxygen_level / 21.0).max(0.1);
    let aerobic_boost = oxygen_factor.sqrt();

    // Speed-dependent oxygen consumption
    let activity_drain = (speed_mult - 1.0).max(0.0) * 0.01;

    let mut move_cost =
        ctx.config.metabolism.base_move_cost * metabolism_mult * speed_mult / aerobic_boost;
    if predation_mode {
        move_cost *= 2.0;
    }

    let signal_cost = outputs[5].abs() as f64 * ctx.config.social.sharing_fraction * 2.0; // Scaled signal cost
    let brain_maintenance = (entity.intel.genotype.brain.nodes.len() as f64
        * ctx.config.brain.hidden_node_cost)
        + (entity.intel.genotype.brain.connections.len() as f64 * ctx.config.brain.connection_cost);

    // Nest Metabolic Benefit
    let mut base_idle = ctx.config.metabolism.base_idle_cost;

    // Phase 61: Ancestral Traits (Hardened Metabolism)
    if entity
        .intel
        .ancestral_traits
        .contains(&crate::model::state::lineage_registry::AncestralTrait::HardenedMetabolism)
    {
        base_idle *= 0.8;
    }

    let terrain_type = ctx
        .terrain
        .get(entity.physics.x, entity.physics.y)
        .terrain_type;
    if matches!(
        terrain_type,
        crate::model::state::terrain::TerrainType::Nest
    ) {
        base_idle *= 1.0 - ctx.config.ecosystem.corpse_fertility_mult as f64; // Using as proxy for nest bonus
    }

    let mut idle_cost = (base_idle + brain_maintenance) * metabolism_mult;

    // Symbiosis Efficiency Bonus
    if entity.intel.bonded_to.is_some() {
        move_cost *= 0.9;
        idle_cost *= 0.9;
    }

    // Hypoxia Stress
    if ctx.env.oxygen_level < 8.0 {
        idle_cost += 1.0;
    }

    // Kinematic Coupling (Spring Force)
    if let Some(partner_id) = entity.intel.bonded_to {
        if let Some(partner) = ctx
            .entity_id_map
            .get(&partner_id)
            .map(|&idx| &ctx.snapshots[idx])
        {
            let dx = partner.x - entity.physics.x;
            let dy = partner.y - entity.physics.y;
            let dist = (dx * dx + dy * dy).sqrt();

            let k = ctx.config.brain.coupling_spring_constant;
            let rest_length = 2.0;

            if dist > rest_length {
                let force = (dist - rest_length) * k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                entity.physics.vx += fx;
                entity.physics.vy += fy;
            }
        }
    }

    // Leadership Vector (Follow Alphas)
    if entity.intel.bonded_to.is_none() {
        let mut best_alpha_pos = None;
        let mut max_rank = entity.intel.rank;

        let nearby = ctx.spatial_hash.query(
            entity.physics.x,
            entity.physics.y,
            entity.physics.sensing_range,
        );

        for idx in nearby {
            let s = &ctx.snapshots[idx];
            if s.id != entity.id && s.lineage_id == entity.metabolism.lineage_id {
                let dx = s.x - entity.physics.x;
                let dy = s.y - entity.physics.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < entity.physics.sensing_range.powi(2) && s.rank > max_rank + 0.1 {
                    max_rank = s.rank;
                    best_alpha_pos = Some((s.x, s.y));
                }
            }
        }

        if let Some((ax, ay)) = best_alpha_pos {
            let dx = ax - entity.physics.x;
            let dy = ay - entity.physics.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            entity.physics.vx += (dx / dist) * ctx.config.brain.alpha_following_force;
            entity.physics.vy += (dy / dist) * ctx.config.brain.alpha_following_force;
        }
    }

    entity.metabolism.energy -= move_cost + idle_cost + signal_cost;

    let mut pheromones = Vec::new();
    if outputs[6] > 0.5 {
        pheromones.push(crate::model::state::pheromone::PheromoneDeposit {
            x: entity.physics.x,
            y: entity.physics.y,
            ptype: crate::model::state::pheromone::PheromoneType::SignalA,
            amount: 0.5,
        });
    }
    if outputs[7] > 0.5 {
        pheromones.push(crate::model::state::pheromone::PheromoneDeposit {
            x: entity.physics.x,
            y: entity.physics.y,
            ptype: crate::model::state::pheromone::PheromoneType::SignalB,
            amount: 0.5,
        });
    }

    let mut sounds = Vec::new();
    if entity.intel.last_vocalization > 0.1 {
        sounds.push(crate::model::state::sound::SoundDeposit {
            x: entity.physics.x,
            y: entity.physics.y,
            amount: entity.intel.last_vocalization,
        });
    }

    let mut pressure = Vec::new();
    if outputs[9] > 0.5 {
        pressure.push(crate::model::state::pressure::PressureDeposit {
            x: entity.physics.x,
            y: entity.physics.y,
            ptype: crate::model::state::pressure::PressureType::DigDemand,
            amount: outputs[9],
        });
    }
    if outputs[10] > 0.5 {
        pressure.push(crate::model::state::pressure::PressureDeposit {
            x: entity.physics.x,
            y: entity.physics.y,
            ptype: crate::model::state::pressure::PressureType::BuildDemand,
            amount: outputs[10],
        });
    }

    // Phase 60: Biological Irrigation (Engineer caste logic)
    if entity.intel.specialization == Some(crate::model::state::entity::Specialization::Engineer) {
        let is_near_river = ctx.terrain.has_neighbor_type(
            entity.physics.x as u16,
            entity.physics.y as u16,
            crate::model::state::terrain::TerrainType::River,
        );
        let cell = ctx.terrain.get(entity.physics.x, entity.physics.y);
        if is_near_river
            && matches!(
                cell.terrain_type,
                crate::model::state::terrain::TerrainType::Plains
                    | crate::model::state::terrain::TerrainType::Desert
            )
        {
            // Engineers near rivers deposit Dig demand to extend canals
            pressure.push(crate::model::state::pressure::PressureDeposit {
                x: entity.physics.x,
                y: entity.physics.y,
                ptype: crate::model::state::pressure::PressureType::DigDemand,
                amount: 0.8,
            });
        }
    }

    let mut overmind_broadcast = None;
    if outputs[11] > 0.5 && entity.intel.rank > 0.8 {
        overmind_broadcast = Some((entity.metabolism.lineage_id, outputs[11]));
    }

    handle_movement(entity, speed_mult, ctx.terrain, ctx.width, ctx.height);
    ActionResult {
        pheromones,
        sounds,
        pressure,
        oxygen_drain: activity_drain,
        overmind_broadcast,
    }
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
        let outputs = [0.0; 12];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
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
        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx_n = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity_normal, normal_outputs, &mut ctx_n);
        let mut ctx_p = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
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
        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        terrain.set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Plains);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity, outputs, &mut ctx);
        assert!(entity.physics.vx > 0.0);
        assert!(entity.physics.vy < 0.0);
    }
}
