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

pub struct ActionEntity<'a> {
    pub id: &'a uuid::Uuid,
    pub position: &'a mut primordium_data::Position,
    pub velocity: &'a mut primordium_data::Velocity,
    pub physics: &'a Physics,
    pub metabolism: &'a mut Metabolism,
    pub intel: &'a mut Intel,
    pub health: &'a mut Health,
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

pub struct TerraformContext<'a> {
    pub position: &'a mut primordium_data::Position,
    pub velocity: &'a mut primordium_data::Velocity,
    pub physics: &'a Physics,
    pub intel: &'a mut Intel,
    pub terrain: &'a TerrainGrid,
    pub pressure: &'a crate::pressure::PressureGrid,
    pub cell: &'a crate::terrain::TerrainCell,
    pub output: &'a mut ActionOutput,
}

fn handle_terraforming(ctx: &mut TerraformContext) {
    if let Some(spec) = ctx.intel.specialization {
        if spec == Specialization::Engineer {
            let is_near_river = ctx.terrain.has_neighbor_type(
                ctx.position.x as u16,
                ctx.position.y as u16,
                crate::terrain::TerrainType::River,
            );

            let (tx, ty, max_press) = ctx.pressure.find_highest_in_range(
                ctx.position.x,
                ctx.position.y,
                ctx.physics.sensing_range,
            );

            if max_press > 0.2 {
                let dx = tx - ctx.position.x;
                let dy = ty - ctx.position.y;
                let dist = (dx * dx + dy * dy).sqrt().max(1.0);
                let attr_force = if is_near_river { 0.3 } else { 0.15 };
                ctx.velocity.vx += (dx / dist) * attr_force;
                ctx.velocity.vy += (dy / dist) * attr_force;
            }

            if is_near_river
                && matches!(
                    ctx.cell.terrain_type,
                    primordium_data::TerrainType::Plains | primordium_data::TerrainType::Desert
                )
            {
                ctx.output.pressure.push(crate::pressure::PressureDeposit {
                    x: ctx.position.x,
                    y: ctx.position.y,
                    ptype: crate::pressure::PressureType::DigDemand,
                    amount: 0.8,
                });
            }
        }
    }
}

pub fn action_system_components_with_modifiers(
    entity: &mut ActionEntity,
    eff_max_speed: f64,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    let speed_mult = (1.0 + f64::midpoint(f64::from(outputs[2]), 1.0)) * eff_max_speed;
    let predation_mode = f64::midpoint(f64::from(outputs[3]), 1.0) > 0.5;

    entity.intel.last_aggression = f32::midpoint(outputs[3], 1.0);
    entity.intel.last_share_intent = f32::midpoint(outputs[4], 1.0);
    entity.intel.last_signal = outputs[5];
    entity.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

    let stomach_penalty = (entity.metabolism.max_energy - 200.0).max(0.0) / 1000.0;
    let inertia = (0.8 + stomach_penalty).clamp(0.4, 0.95);
    entity.velocity.vx = entity.velocity.vx * inertia + f64::from(outputs[0]) * (1.0 - inertia);
    entity.velocity.vy = entity.velocity.vy * inertia + f64::from(outputs[1]) * (1.0 - inertia);

    let metabolism_mult = ctx.env.metabolism_multiplier();

    let activity_drain = (speed_mult - 1.0).max(0.0) * 0.01;

    let cell = ctx.terrain.get(entity.position.x, entity.position.y);
    let local_cooling = cell.local_cooling;

    let effective_metabolism_mult = if metabolism_mult > 1.0 {
        1.0 + (metabolism_mult - 1.0) * (1.0 - f64::from(local_cooling) * 0.8).max(0.0)
    } else {
        metabolism_mult
    };

    let total_cost = calculate_metabolic_cost(MetabolicCostInput {
        intel: entity.intel,
        metabolism: entity.metabolism,
        ctx,
        speed_mult,
        predation_mode,
        signal_strength: outputs[5].abs(),
        activity_drain,
        cell,
        effective_metabolism_mult,
        x: entity.position.x,
        y: entity.position.y,
    });

    entity.metabolism.energy -= total_cost;

    apply_social_forces(
        &mut BondContext {
            id: entity.id,
            position: entity.position,
            velocity: entity.velocity,
            physics: entity.physics,
            intel: entity.intel,
            metabolism: entity.metabolism,
        },
        ctx,
    );

    handle_emissions(entity.position, outputs, entity.intel, output);

    handle_terraforming(&mut TerraformContext {
        position: entity.position,
        velocity: entity.velocity,
        physics: entity.physics,
        intel: entity.intel,
        terrain: ctx.terrain,
        pressure: ctx.pressure,
        cell,
        output,
    });

    if outputs[11] > 0.8 && entity.intel.rank > 0.8 {
        output.overmind_broadcast = Some((*entity.id, outputs[11]));
    }

    handle_repulsion(entity.position, entity.velocity, entity.id, ctx);

    handle_movement_components(MovementContext {
        position: entity.position,
        velocity: entity.velocity,
        speed: speed_mult,
        terrain: ctx.terrain,
        width: ctx.width,
        height: ctx.height,
    });
    output.oxygen_drain = activity_drain;
}

struct MetabolicCostInput<'a, 'b> {
    intel: &'a Intel,
    metabolism: &'a Metabolism,
    ctx: &'b ActionContext<'b>,
    speed_mult: f64,
    predation_mode: bool,
    signal_strength: f32,
    activity_drain: f64,
    cell: &'a crate::terrain::TerrainCell,
    effective_metabolism_mult: f64,
    x: f64,
    y: f64,
}

fn calculate_metabolic_cost<'a, 'b>(input: MetabolicCostInput<'a, 'b>) -> f64 {
    let oxygen_factor = (input.ctx.env.oxygen_level / 21.0).max(0.1);
    let aerobic_boost = oxygen_factor.sqrt();

    let mut move_cost = input.ctx.config.metabolism.base_move_cost
        * input.effective_metabolism_mult
        * input.speed_mult
        / aerobic_boost;

    if input.predation_mode {
        move_cost *= 2.0;
    }

    let signal_cost =
        f64::from(input.signal_strength) * input.ctx.config.social.sharing_fraction * 2.0;
    let brain_maintenance = (input.intel.genotype.brain.nodes.len() as f64
        * input.ctx.config.brain.hidden_node_cost)
        + (input.intel.genotype.brain.connections.len() as f64
            * input.ctx.config.brain.connection_cost);

    let mut base_idle = input.ctx.config.metabolism.base_idle_cost;

    if input
        .intel
        .ancestral_traits
        .contains(&primordium_data::AncestralTrait::HardenedMetabolism)
    {
        base_idle *= 0.8;
    }

    if matches!(input.cell.terrain_type, primordium_data::TerrainType::Nest) {
        base_idle *= 1.0 - f64::from(input.ctx.config.ecosystem.corpse_fertility_mult);
    }

    let mut idle_cost = (base_idle + brain_maintenance) * input.effective_metabolism_mult;

    if input.intel.bonded_to.is_some() {
        move_cost *= 0.9;
        idle_cost *= 0.9;
    }

    if input.ctx.env.oxygen_level < 8.0 {
        idle_cost += 1.0;
    }

    let (dom_l, intensity) = input.ctx.influence.get_influence(input.x, input.y);
    if let Some(lid) = dom_l {
        if lid != input.metabolism.lineage_id {
            idle_cost += f64::from(intensity) * 0.1;
        }
    }

    // Carbon stress increases metabolic cost in high-carbon environments
    let carbon_stress = input.ctx.env.carbon_stress_factor();
    if carbon_stress > 0.0 {
        idle_cost *= 1.0 + carbon_stress * 2.0;
    }

    move_cost + signal_cost + idle_cost + input.activity_drain
}

pub struct BondContext<'a> {
    pub id: &'a uuid::Uuid,
    pub position: &'a primordium_data::Position,
    pub velocity: &'a mut primordium_data::Velocity,
    pub physics: &'a Physics,
    pub metabolism: &'a Metabolism,
    pub intel: &'a Intel,
}

fn apply_social_forces(ctx: &mut BondContext, action_ctx: &ActionContext) {
    if let Some(partner_id) = ctx.intel.bonded_to {
        if let Some(partner) = action_ctx
            .entity_id_map
            .get(&partner_id)
            .map(|&idx| &action_ctx.snapshots[idx])
        {
            let dx = partner.x - ctx.position.x;
            let dy = partner.y - ctx.position.y;
            let dist = (dx * dx + dy * dy).sqrt();

            let k = action_ctx.config.brain.coupling_spring_constant;
            let rest_length = 2.0;

            if dist > rest_length {
                let force = (dist - rest_length) * k;
                let fx = (dx / dist) * force;
                let fy = (dy / dist) * force;

                ctx.velocity.vx += fx;
                ctx.velocity.vy += fy;
            }
        }
    } else {
        let mut best_alpha_pos = None;
        let mut max_rank = ctx.intel.rank;

        action_ctx.spatial_hash.query_callback(
            ctx.position.x,
            ctx.position.y,
            ctx.physics.sensing_range,
            |idx| {
                let s = &action_ctx.snapshots[idx];
                if s.id != *ctx.id && s.lineage_id == ctx.metabolism.lineage_id {
                    let dx = s.x - ctx.position.x;
                    let dy = s.y - ctx.position.y;
                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq < ctx.physics.sensing_range.powi(2) && s.rank > max_rank + 0.1 {
                        max_rank = s.rank;
                        best_alpha_pos = Some((s.x, s.y));
                    }
                }
            },
        );

        if let Some((ax, ay)) = best_alpha_pos {
            let dx = ax - ctx.position.x;
            let dy = ay - ctx.position.y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            ctx.velocity.vx = ctx.velocity.vx * 0.7 + (dx / dist) * 0.3;
            ctx.velocity.vy = ctx.velocity.vy * 0.7 + (dy / dist) * 0.3;
        }
    }
}

fn handle_emissions(
    position: &primordium_data::Position,
    outputs: [f32; 12],
    _intel: &Intel,
    output: &mut ActionOutput,
) {
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
}

pub fn action_system_components(
    entity: &mut ActionEntity,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    action_system_components_with_modifiers(entity, entity.physics.max_speed, outputs, ctx, output);
}

pub fn handle_movement(
    entity: &mut Entity,
    speed: f64,
    terrain: &TerrainGrid,
    width: u16,
    height: u16,
) {
    handle_movement_components(MovementContext {
        position: &mut entity.position,
        velocity: &mut entity.velocity,
        speed,
        terrain,
        width,
        height,
    });
}

pub struct MovementContext<'a> {
    pub position: &'a mut primordium_data::Position,
    pub velocity: &'a mut primordium_data::Velocity,
    pub speed: f64,
    pub terrain: &'a TerrainGrid,
    pub width: u16,
    pub height: u16,
}

fn handle_repulsion(
    position: &primordium_data::Position,
    velocity: &mut primordium_data::Velocity,
    id: &uuid::Uuid,
    ctx: &ActionContext,
) {
    if ctx.config.world.repulsion_force <= 0.0 {
        return;
    }

    let radius = 0.5; // Repulsion radius
    let force_mult = ctx.config.world.repulsion_force;

    ctx.spatial_hash
        .query_callback(position.x, position.y, radius, |idx| {
            let neighbor = &ctx.snapshots[idx];
            if neighbor.id != *id {
                let dx = position.x - neighbor.x;
                let dy = position.y - neighbor.y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < radius * radius && dist_sq > 0.0001 {
                    let dist = dist_sq.sqrt();
                    let push = (radius - dist) / radius; // 0.0 to 1.0
                    let force = push * force_mult;

                    velocity.vx += (dx / dist) * force;
                    velocity.vy += (dy / dist) * force;
                } else if dist_sq <= 0.0001 {
                    // Exact overlap - push in random direction based on ID hash or pseudo-random
                    // To be deterministic, use ID byte logic
                    let hash = id.as_bytes()[0] as f64;
                    let angle = hash * 0.1;
                    velocity.vx += angle.cos() * force_mult * 0.5;
                    velocity.vy += angle.sin() * force_mult * 0.5;
                }
            }
        });
}

pub fn handle_movement_components(ctx: MovementContext) {
    let next_x = ctx.position.x + ctx.velocity.vx * ctx.speed;
    let next_y = ctx.position.y + ctx.velocity.vy * ctx.speed;

    if ctx.terrain.get(next_x, next_y).terrain_type == primordium_data::TerrainType::Wall {
        ctx.velocity.vx *= -0.5;
        ctx.velocity.vy *= -0.5;
    } else {
        ctx.position.x = next_x;
        ctx.position.y = next_y;
    }

    if ctx.position.x < 0.0 {
        ctx.position.x = 0.0;
        ctx.velocity.vx *= -1.0;
    } else if ctx.position.x >= ctx.width as f64 {
        ctx.position.x = ctx.width as f64 - 0.1;
        ctx.velocity.vx *= -1.0;
    }

    if ctx.position.y < 0.0 {
        ctx.position.y = 0.0;
        ctx.velocity.vy *= -1.0;
    } else if ctx.position.y >= ctx.height as f64 {
        ctx.position.y = ctx.height as f64 - 0.1;
        ctx.velocity.vy *= -1.0;
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

pub fn action_system(
    entity: &mut Entity,
    outputs: [f32; 12],
    ctx: &mut ActionContext,
    output: &mut ActionOutput,
) {
    let mut action_entity = ActionEntity {
        id: &entity.identity.id,
        position: &mut entity.position,
        velocity: &mut entity.velocity,
        physics: &entity.physics,
        metabolism: &mut entity.metabolism,
        intel: &mut entity.intel,
        health: &mut entity.health,
    };
    action_system_components(&mut action_entity, outputs, ctx, output);
}
