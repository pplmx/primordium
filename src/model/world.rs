use crate::model::entity::Entity;
use crate::model::environment::Environment;
use crate::model::food::Food;
use rand::Rng;

pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
}

impl World {
    pub fn new(width: u16, height: u16, initial_population: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut entities = Vec::with_capacity(initial_population);

        for _ in 0..initial_population {
            // Spawn slightly inside to avoid immediate collision
            let x = rng.gen_range(2.0..f64::from(width - 2));
            let y = rng.gen_range(2.0..f64::from(height - 2));
            entities.push(Entity::new(x, y));
        }

        // Initial food population
        let mut food = Vec::new();
        for _ in 0..30 {
            let x = rng.gen_range(1..width - 1);
            let y = rng.gen_range(1..height - 1);
            food.push(Food::new(x, y));
        }

        Self {
            width,
            height,
            entities,
            food,
            tick: 0,
        }
    }

    pub fn update(&mut self, env: &Environment) {
        self.tick += 1;
        let mut rng = rand::thread_rng();

        let width_f = f64::from(self.width);
        let height_f = f64::from(self.height);

        // Multipliers from Hardware Coupling
        let metabolism_mult = env.metabolism_multiplier();
        let food_spawn_mult = env.food_spawn_multiplier();

        // 1. Food Spawning
        // Base rate: 0.5 items/second @ 60 FPS => 0.0083 chance per tick
        let spawn_chance = 0.0083 * food_spawn_mult;
        if self.food.len() < 50 && rng.gen::<f64>() < spawn_chance {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(1..self.height - 1);
            self.food.push(Food::new(x, y));
        }

        // 2. Entity Logic
        let mut new_babies = Vec::new();

        for entity in self.entities.iter_mut() {
            // Move cost + Idle cost coupled to CPU usage (Metabolism)
            let move_cost = 1.0 * metabolism_mult;
            let idle_cost = 0.5 * metabolism_mult;
            entity.energy -= move_cost + idle_cost;

            // Skip dead updates
            if entity.energy <= 0.0 {
                continue;
            }

            // Apply velocity
            entity.x += entity.vx;
            entity.y += entity.vy;

            // Boundary collision (Bounce)
            if entity.x <= 0.0 {
                entity.x = 0.0;
                entity.vx = -entity.vx;
            } else if entity.x >= width_f {
                entity.x = width_f - 0.1;
                entity.vx = -entity.vx;
            }
            if entity.y <= 0.0 {
                entity.y = 0.0;
                entity.vy = -entity.vy;
            } else if entity.y >= height_f {
                entity.y = height_f - 0.1;
                entity.vy = -entity.vy;
            }

            // Eating Logic
            let mut eaten_idx = None;
            for (f_idx, f) in self.food.iter().enumerate() {
                let dx = entity.x - f64::from(f.x);
                let dy = entity.y - f64::from(f.y);
                if (dx * dx + dy * dy) < 1.5 {
                    eaten_idx = Some(f_idx);
                    break;
                }
            }

            if let Some(f_idx) = eaten_idx {
                entity.energy += self.food[f_idx].value;
                if entity.energy > entity.max_energy {
                    entity.energy = entity.max_energy;
                }
                self.food.swap_remove(f_idx);
            }

            // Reproduction
            if entity.energy > 150.0 {
                new_babies.push(entity.reproduce());
            }
        }

        // Remove dead
        self.entities.retain(|e| e.energy > 0.0);

        // Add babies
        self.entities.append(&mut new_babies);
    }
}
