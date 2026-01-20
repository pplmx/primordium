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
            let x = rng.gen_range(2.0..f64::from(width - 2));
            let y = rng.gen_range(2.0..f64::from(height - 2));
            entities.push(Entity::new(x, y));
        }

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

        let metabolism_mult = env.metabolism_multiplier();
        let food_spawn_mult = env.food_spawn_multiplier();

        // 1. Food Spawning
        let spawn_chance = 0.0083 * food_spawn_mult;
        if self.food.len() < 50 && rng.gen::<f64>() < spawn_chance {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(1..self.height - 1);
            self.food.push(Food::new(x, y));
        }

        // 2. Entity Logic
        let mut new_babies = Vec::new();

        // We need to clone entities or use indices to avoid borrow checker issues with crowding/senses
        // For 100 entities, cloning position/ID is cheap.
        let entity_positions: Vec<(f64, f64)> = self.entities.iter().map(|e| (e.x, e.y)).collect();

        for entity in self.entities.iter_mut() {
            // --- SENSORY INPUTS ---

            // A. Nearest Food
            let mut dx_food = 0.0;
            let mut dy_food = 0.0;
            let mut min_dist_sq = f64::MAX;

            for f in &self.food {
                let dx = f64::from(f.x) - entity.x;
                let dy = f64::from(f.y) - entity.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < min_dist_sq {
                    min_dist_sq = dist_sq;
                    dx_food = dx;
                    dy_food = dy;
                }
            }

            // Normalize dx/dy to [-1.0, 1.0] (cap at some reasonable distance, e.g., 20.0)
            let cap = 20.0;
            let dx_input = (dx_food / cap).clamp(-1.0, 1.0) as f32;
            let dy_input = (dy_food / cap).clamp(-1.0, 1.0) as f32;

            // B. Energy Ratio
            let energy_input = (entity.energy / entity.max_energy) as f32;

            // C. Crowding (neighbors within 5.0 radius / 10.0)
            let mut neighbors = 0;
            for (nx, ny) in &entity_positions {
                let dx = nx - entity.x;
                let dy = ny - entity.y;
                if dx * dx + dy * dy < 25.0 {
                    neighbors += 1;
                }
            }
            let crowding_input = (neighbors as f32 / 10.0).min(1.0);

            // --- BRAIN FORWARD ---
            let inputs = [dx_input, dy_input, energy_input, crowding_input];
            let outputs = entity.brain.forward(inputs);

            // --- OUTPUT INTERPRETATION ---
            // outputs[0]: Move intent X
            // outputs[1]: Move intent Y
            // outputs[2]: Speed boost

            let move_x = outputs[0] as f64;
            let move_y = outputs[1] as f64;
            let speed_boost = (outputs[2] as f64 + 1.0) / 2.0; // Map tanh [-1,1] to [0,1]

            let speed = 1.0 + speed_boost; // 1.0 to 2.0

            // Apply Intent to Velocity (Small friction/smoothing)
            entity.vx = entity.vx * 0.8 + move_x * 0.2;
            entity.vy = entity.vy * 0.8 + move_y * 0.2;

            // --- METABOLISM ---
            let move_cost = 1.0 * metabolism_mult * speed;
            let idle_cost = 0.5 * metabolism_mult;
            entity.energy -= move_cost + idle_cost;

            if entity.energy <= 0.0 {
                continue;
            }

            // Apply velocity * speed
            entity.x += entity.vx * speed;
            entity.y += entity.vy * speed;

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

        self.entities.retain(|e| e.energy > 0.0);
        self.entities.append(&mut new_babies);
    }
}
