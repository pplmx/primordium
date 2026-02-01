//! Ecological system - handles food spawning and consumption.

use crate::model::environment::Environment;
use crate::model::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::TerrainGrid;
use primordium_data::{Entity, Food};
use rand::Rng;
use std::collections::HashSet;

/// Context for feeding operations.
pub struct FeedingContext<'a> {
    pub food: &'a [Food],
    pub food_hash: &'a SpatialHash,
    pub eaten_indices: &'a mut HashSet<usize>,
    pub terrain: &'a mut TerrainGrid,
    pub pheromones: &'a mut PheromoneGrid,
    pub config: &'a crate::model::config::AppConfig,
    pub lineage_consumption: &'a mut Vec<(uuid::Uuid, f64)>,
}

pub fn spawn_food_ecs(
    world: &mut hecs::World,
    env: &Environment,
    terrain: &TerrainGrid,
    config: &crate::model::config::AppConfig,
    width: u16,
    height: u16,
    rng: &mut impl Rng,
) {
    let food_spawn_mult = env.food_spawn_multiplier();
    let base_spawn_chance =
        config.ecosystem.base_spawn_chance as f64 * food_spawn_mult * env.light_level() as f64;

    let mut food_count = 0;
    for _ in world.query::<&Food>().iter() {
        food_count += 1;
    }

    if food_count < config.world.max_food {
        for _ in 0..3 {
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            let terrain_mod = terrain.food_spawn_modifier(f64::from(x), f64::from(y));
            if terrain_mod > 0.0 && rng.gen::<f64>() < base_spawn_chance * terrain_mod {
                let terrain_type = terrain.get_cell(x, y).terrain_type;
                let nutrient_type = match terrain_type {
                    crate::model::terrain::TerrainType::Mountain
                    | crate::model::terrain::TerrainType::River => rng.gen_range(0.6..1.0),
                    _ => rng.gen_range(0.0..0.4),
                };
                let new_food = Food::new(x, y, nutrient_type);
                world.spawn((
                    new_food,
                    crate::model::state::Position {
                        x: x as f64,
                        y: y as f64,
                    },
                    crate::model::state::MetabolicNiche(nutrient_type),
                ));
                break;
            }
        }
    }
}

/// Spawn new food items based on environment and terrain.
pub fn spawn_food(
    food: &mut Vec<Food>,
    env: &Environment,
    terrain: &TerrainGrid,
    config: &crate::model::config::AppConfig,
    width: u16,
    height: u16,
    rng: &mut impl Rng,
) {
    let food_spawn_mult = env.food_spawn_multiplier();
    let base_spawn_chance =
        config.ecosystem.base_spawn_chance as f64 * food_spawn_mult * env.light_level() as f64;
    if food.len() < config.world.max_food {
        for _ in 0..3 {
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            let terrain_mod = terrain.food_spawn_modifier(f64::from(x), f64::from(y));
            if terrain_mod > 0.0 && rng.gen::<f64>() < base_spawn_chance * terrain_mod {
                // Typed food: Mountain/River favors Blue, Plains favor Green
                let terrain_type = terrain.get_cell(x, y).terrain_type;
                let nutrient_type = match terrain_type {
                    crate::model::terrain::TerrainType::Mountain
                    | crate::model::terrain::TerrainType::River => {
                        rng.gen_range(0.6..1.0) // Favor Blue
                    }
                    _ => rng.gen_range(0.0..0.4), // Favor Green
                };
                food.push(Food::new(x, y, nutrient_type));
                break;
            }
        }
    }
}

/// Optimized feeding handler using spatial hashing.
pub fn handle_feeding_optimized(idx: usize, entities: &mut [Entity], ctx: &mut FeedingContext) {
    let sensing_radius = entities[idx].physics.sensing_range;
    let mut nearby_food = Vec::new();
    ctx.food_hash.query_into(
        entities[idx].physics.x,
        entities[idx].physics.y,
        1.5f64.max(sensing_radius / 4.0),
        &mut nearby_food,
    );

    let mut eaten_idx = None;
    for &f_idx in &nearby_food {
        if !ctx.eaten_indices.contains(&f_idx) {
            let f = &ctx.food[f_idx];
            let dx = f64::from(f.x) - entities[idx].physics.x;
            let dy = f64::from(f.y) - entities[idx].physics.y;
            if (dx * dx + dy * dy).sqrt() < 1.5 {
                eaten_idx = Some(f_idx);
                break;
            }
        }
    }

    if let Some(f_idx) = eaten_idx {
        let f = &ctx.food[f_idx];

        // NEW: Trophic Continuum - Plants only provide energy based on Herbivore potential
        let trophic_efficiency = 1.0 - entities[idx].metabolism.trophic_potential as f64;

        // If trophic efficiency is near zero, skip eating to allow specialist logic
        if trophic_efficiency < 0.1 {
            return;
        }

        // Niche match efficiency
        let niche_match =
            1.0 - (entities[idx].intel.genotype.metabolic_niche - f.nutrient_type).abs() as f64;
        let efficiency = (niche_match * ctx.config.ecosystem.nutrient_niche_multiplier as f64)
            .clamp(0.2, 1.2)
            * trophic_efficiency;

        let energy_gain = ctx.config.metabolism.food_value * efficiency;
        entities[idx].metabolism.energy = (entities[idx].metabolism.energy + energy_gain)
            .min(entities[idx].metabolism.max_energy);

        ctx.terrain.deplete(
            entities[idx].physics.x,
            entities[idx].physics.y,
            ctx.config.ecosystem.soil_depletion_unit,
        );
        ctx.pheromones.deposit(
            entities[idx].physics.x,
            entities[idx].physics.y,
            PheromoneType::Food,
            0.3,
        );
        ctx.eaten_indices.insert(f_idx);
        ctx.lineage_consumption
            .push((entities[idx].metabolism.lineage_id, energy_gain));
    }
}

pub fn sense_nearest_food_ecs_decomposed(
    position: &primordium_data::Position,
    sensing_range: f64,
    world: &hecs::World,
    food_hash: &SpatialHash,
    food_handles: &[hecs::Entity],
) -> (f64, f64, f32) {
    let mut dx_food = 0.0;
    let mut dy_food = 0.0;
    let mut f_type = 0.5;
    let mut min_dist_sq = f64::MAX;

    food_hash.query_callback(position.x, position.y, sensing_range, |f_idx| {
        let handle = food_handles[f_idx];
        if let Ok(f) = world.get::<&Food>(handle) {
            let dx = f64::from(f.x) - position.x;
            let dy = f64::from(f.y) - position.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
                dx_food = dx;
                dy_food = dy;
                f_type = f.nutrient_type;
            }
        }
    });

    if min_dist_sq == f64::MAX {
        (0.0, 0.0, 0.5)
    } else {
        (dx_food, dy_food, f_type)
    }
}

/// Sense the nearest food within a radius (using components).
pub fn sense_nearest_food_components(
    physics: &primordium_data::Physics,
    food: &[Food],
    food_hash: &SpatialHash,
) -> (f64, f64, f32) {
    let mut dx_food = 0.0;
    let mut dy_food = 0.0;
    let mut f_type = 0.5;
    let mut min_dist_sq = f64::MAX;

    food_hash.query_callback(physics.x, physics.y, 20.0, |f_idx| {
        let f = &food[f_idx];
        let dx = f64::from(f.x) - physics.x;
        let dy = f64::from(f.y) - physics.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            dx_food = dx;
            dy_food = dy;
            f_type = f.nutrient_type;
        }
    });

    if min_dist_sq == f64::MAX {
        (0.0, 0.0, 0.5)
    } else {
        (dx_food, dy_food, f_type)
    }
}

/// Sense the nearest food within a radius.
pub fn sense_nearest_food(
    entity: &Entity,
    food: &[Food],
    food_hash: &SpatialHash,
) -> (f64, f64, f32) {
    sense_nearest_food_components(&entity.physics, food, food_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sense_nearest_food_empty() {
        let entity = crate::model::lifecycle::create_entity(5.0, 5.0, 0);
        let food: Vec<Food> = vec![];
        let food_hash = SpatialHash::new(5.0, 100, 100);
        let (dx, dy, _) = sense_nearest_food(&entity, &food, &food_hash);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn test_spawn_food_respects_max() {
        let mut food = vec![Food::new(1, 1, 0.0); 100];
        let env = Environment::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let mut rng = rand::thread_rng();
        let initial_count = food.len();
        let config = crate::model::config::AppConfig::default();
        spawn_food(&mut food, &env, &terrain, &config, 20, 20, &mut rng);
        assert_eq!(food.len(), initial_count);
    }
}
