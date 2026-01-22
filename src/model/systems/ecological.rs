//! Ecological system - handles food spawning, feeding, and environmental sensing.

use crate::model::entity::{Entity, EntityRole};
use crate::model::environment::Environment;
use crate::model::food::Food;
use crate::model::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::quadtree::SpatialHash;
use crate::model::terrain::TerrainGrid;
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

/// Handle herbivore feeding with spatial optimization.
#[allow(clippy::too_many_arguments)]
pub fn handle_feeding_optimized(
    idx: usize,
    entities: &mut [Entity],
    food: &[Food],
    food_hash: &SpatialHash,
    eaten_indices: &mut HashSet<usize>,
    terrain: &mut TerrainGrid,
    pheromones: &mut PheromoneGrid,
    food_value: f64,
) {
    if !matches!(entities[idx].metabolism.role, EntityRole::Herbivore) {
        return;
    }
    let mut eaten_idx = None;
    let nearby_food = food_hash.query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
    for f_idx in nearby_food {
        if eaten_indices.contains(&f_idx) {
            continue;
        }
        let f = &food[f_idx];
        if (entities[idx].physics.x - f64::from(f.x)).powi(2)
            + (entities[idx].physics.y - f64::from(f.y)).powi(2)
            < 2.25
        {
            eaten_idx = Some(f_idx);
            break;
        }
    }
    if let Some(f_idx) = eaten_idx {
        entities[idx].metabolism.energy =
            (entities[idx].metabolism.energy + food_value).min(entities[idx].metabolism.max_energy);
        terrain.deplete(entities[idx].physics.x, entities[idx].physics.y, 0.4);
        pheromones.deposit(
            entities[idx].physics.x,
            entities[idx].physics.y,
            PheromoneType::Food,
            0.3,
        );
        eaten_indices.insert(f_idx);
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
    use crate::model::environment::Environment;
    use crate::model::terrain::TerrainGrid;

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
}
