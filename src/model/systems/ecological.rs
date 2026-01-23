//! Ecological system - handles food spawning, feeding, and environmental sensing.

use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::{Entity, EntityRole};
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::state::terrain::TerrainGrid;
use rand::Rng;
use std::collections::HashSet;

/// Spawn food in the world based on environment and terrain.
pub fn spawn_food(
    food: &mut Vec<Food>,
    env: &Environment,
    terrain: &TerrainGrid,
    max_food: usize,
    width: u16,
    height: u16,
    rng: &mut impl Rng,
) {
    let food_spawn_mult = env.food_spawn_multiplier();
    let base_spawn_chance = 0.0083 * food_spawn_mult * env.light_level() as f64;
    if food.len() < max_food {
        for _ in 0..3 {
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            let terrain_mod = terrain.food_spawn_modifier(f64::from(x), f64::from(y));
            if terrain_mod > 0.0 && rng.gen::<f64>() < base_spawn_chance * terrain_mod {
                food.push(Food::new(x, y));
                break;
            }
        }
    }
}

/// Context for feeding operations, reducing parameter count.
pub struct FeedingContext<'a> {
    pub food: &'a [Food],
    pub food_hash: &'a SpatialHash,
    pub eaten_indices: &'a mut HashSet<usize>,
    pub terrain: &'a mut TerrainGrid,
    pub pheromones: &'a mut PheromoneGrid,
    pub food_value: f64,
    pub lineage_consumption: &'a mut Vec<(uuid::Uuid, f64)>, // NEW
}

/// Handle herbivore feeding with spatial optimization.
pub fn handle_feeding_optimized(idx: usize, entities: &mut [Entity], ctx: &mut FeedingContext) {
    if !matches!(entities[idx].metabolism.role, EntityRole::Herbivore) {
        return;
    }
    let mut eaten_idx = None;
    let nearby_food = ctx
        .food_hash
        .query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
    for f_idx in nearby_food {
        if ctx.eaten_indices.contains(&f_idx) {
            continue;
        }
        let f = &ctx.food[f_idx];
        if (entities[idx].physics.x - f64::from(f.x)).powi(2)
            + (entities[idx].physics.y - f64::from(f.y)).powi(2)
            < 2.25
        {
            eaten_idx = Some(f_idx);
            break;
        }
    }
    if let Some(f_idx) = eaten_idx {
        entities[idx].metabolism.energy = (entities[idx].metabolism.energy + ctx.food_value)
            .min(entities[idx].metabolism.max_energy);
        ctx.terrain
            .deplete(entities[idx].physics.x, entities[idx].physics.y, 0.4);
        ctx.pheromones.deposit(
            entities[idx].physics.x,
            entities[idx].physics.y,
            PheromoneType::Food,
            0.3,
        );
        ctx.eaten_indices.insert(f_idx);
        ctx.lineage_consumption
            .push((entities[idx].metabolism.lineage_id, ctx.food_value));
    }
}

/// Sense the nearest food within a radius.
pub fn sense_nearest_food(entity: &Entity, food: &[Food], food_hash: &SpatialHash) -> (f64, f64) {
    let mut dx_food = 0.0;
    let mut dy_food = 0.0;
    let mut min_dist_sq = f64::MAX;
    let nearby_food = food_hash.query(entity.physics.x, entity.physics.y, 20.0);
    if nearby_food.is_empty() {
        return (0.0, 0.0);
    }
    for &f_idx in &nearby_food {
        let f = &food[f_idx];
        let dx = f64::from(f.x) - entity.physics.x;
        let dy = f64::from(f.y) - entity.physics.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            dx_food = dx;
            dy_food = dy;
        }
    }
    (dx_food, dy_food)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::state::environment::Environment;
    use crate::model::state::terrain::TerrainGrid;

    #[test]
    fn test_sense_nearest_food_empty() {
        let entity = Entity::new(5.0, 5.0, 0);
        let food: Vec<Food> = vec![];
        let food_hash = SpatialHash::new(5.0);

        let (dx, dy) = sense_nearest_food(&entity, &food, &food_hash);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn test_spawn_food_respects_max() {
        let mut food = vec![Food::new(1, 1); 100];
        let env = Environment::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let mut rng = rand::thread_rng();

        let initial_count = food.len();
        spawn_food(&mut food, &env, &terrain, 100, 20, 20, &mut rng);

        // Should not exceed max_food
        assert!(food.len() <= 100, "Food count should not exceed max");
        assert_eq!(
            food.len(),
            initial_count,
            "No food should spawn when at max"
        );
    }

    #[test]
    fn test_feeding_context_usage() {
        let mut entities = vec![Entity::new(5.0, 5.0, 0)];
        entities[0].metabolism.role = EntityRole::Herbivore;
        entities[0].metabolism.energy = 50.0;

        let food = vec![Food::new(5, 5)];
        let mut food_hash = SpatialHash::new(5.0);
        food_hash.insert(5.0, 5.0, 0);

        let mut eaten_indices = HashSet::new();
        let mut terrain = TerrainGrid::generate(10, 10, 42);
        let mut pheromones = PheromoneGrid::new(10, 10);

        let mut lineage_consumption = Vec::new();

        let mut ctx = FeedingContext {
            food: &food,
            food_hash: &food_hash,
            eaten_indices: &mut eaten_indices,
            terrain: &mut terrain,
            pheromones: &mut pheromones,
            food_value: 50.0,
            lineage_consumption: &mut lineage_consumption,
        };

        let initial_energy = entities[0].metabolism.energy;
        handle_feeding_optimized(0, &mut entities, &mut ctx);

        assert!(
            entities[0].metabolism.energy > initial_energy,
            "Entity should have gained energy"
        );
        assert!(
            ctx.eaten_indices.contains(&0),
            "Food should be marked eaten"
        );
    }
}
