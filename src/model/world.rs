use crate::model::brain::Brain;
use crate::model::config::AppConfig;
use crate::model::entity::Entity;
use crate::model::environment::Environment;
use crate::model::food::Food;
use crate::model::history::{HistoryLogger, Legend, LiveEvent};
use crate::model::quadtree::SpatialHash;
use chrono::Utc;
use rand::Rng;
use std::collections::HashSet;

pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
    pub logger: HistoryLogger,
    pub config: AppConfig,
    pub spatial_hash: SpatialHash,
}

impl World {
    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        let mut rng = rand::thread_rng();
        let mut entities = Vec::with_capacity(initial_population);
        let mut logger = HistoryLogger::new()?;

        for _ in 0..initial_population {
            let x = rng.gen_range(2.0..f64::from(config.world.width - 2));
            let y = rng.gen_range(2.0..f64::from(config.world.height - 2));
            let entity = Entity::new(x, y, 0);

            logger.log_event(LiveEvent::Birth {
                id: entity.id,
                parent_id: None,
                gen: entity.generation,
                tick: 0,
                timestamp: Utc::now().to_rfc3339(),
            })?;

            entities.push(entity);
        }

        let mut food = Vec::new();
        for _ in 0..config.world.initial_food {
            let x = rng.gen_range(1..config.world.width - 1);
            let y = rng.gen_range(1..config.world.height - 1);
            food.push(Food::new(x, y));
        }

        Ok(Self {
            width: config.world.width,
            height: config.world.height,
            entities,
            food,
            tick: 0,
            logger,
            config,
            spatial_hash: SpatialHash::new(5.0),
        })
    }

    pub fn update(&mut self, env: &Environment) -> anyhow::Result<Vec<LiveEvent>> {
        let mut events = Vec::new();
        self.tick += 1;
        let mut rng = rand::thread_rng();

        let width_f = f64::from(self.width);
        let height_f = f64::from(self.height);

        let metabolism_mult = env.metabolism_multiplier();
        let food_spawn_mult = env.food_spawn_multiplier();

        // 1. Rebuild Spatial Hash
        self.spatial_hash.clear();
        for (i, e) in self.entities.iter().enumerate() {
            self.spatial_hash.insert(e.x, e.y, i);
        }

        // 2. Food Spawning
        let spawn_chance = 0.0083 * food_spawn_mult;
        if self.food.len() < self.config.world.max_food && rng.gen::<f64>() < spawn_chance {
            let x = rng.gen_range(1..self.width - 1);
            let y = rng.gen_range(1..self.height - 1);
            self.food.push(Food::new(x, y));
        }

        // 3. Entity Logic
        let mut new_babies = Vec::new();
        let mut alive_entities = Vec::new();
        let mut killed_ids = HashSet::new();

        let mut current_entities = std::mem::take(&mut self.entities);

        // Snapshot for non-conflicting reads
        let entity_snapshots: Vec<(uuid::Uuid, f64, f64, f64, u64, u32)> = current_entities
            .iter()
            .map(|e| (e.id, e.x, e.y, e.energy, e.birth_tick, e.offspring_count))
            .collect();

        for i in 0..current_entities.len() {
            let entity_id = current_entities[i].id;
            if killed_ids.contains(&entity_id) {
                continue;
            }

            // --- SENSORY INPUTS ---
            let (dx_food, dy_food) = self.sense_nearest_food(&current_entities[i]);
            let cap = 20.0;
            let dx_input = (dx_food / cap).clamp(-1.0, 1.0) as f32;
            let dy_input = (dy_food / cap).clamp(-1.0, 1.0) as f32;
            let energy_input = (current_entities[i].energy / current_entities[i].max_energy) as f32;

            let neighbors_indices =
                self.spatial_hash
                    .query(current_entities[i].x, current_entities[i].y, 5.0);
            let neighbors = neighbors_indices.len().saturating_sub(1);
            let crowding_input = (neighbors as f32 / 10.0).min(1.0);

            // BRAIN FORWARD
            let inputs = [dx_input, dy_input, energy_input, crowding_input];
            let outputs = current_entities[i].brain.forward(inputs);

            let move_x = outputs[0] as f64;
            let move_y = outputs[1] as f64;
            let speed_boost = (outputs[2] as f64 + 1.0) / 2.0;
            let aggression = (outputs[3] as f64 + 1.0) / 2.0;

            let speed = 1.0 + speed_boost;
            let predation_mode = aggression > 0.5;

            current_entities[i].vx = current_entities[i].vx * 0.8 + move_x * 0.2;
            current_entities[i].vy = current_entities[i].vy * 0.8 + move_y * 0.2;

            // METABOLISM
            let mut move_cost = self.config.metabolism.base_move_cost * metabolism_mult * speed;
            let idle_cost = self.config.metabolism.base_idle_cost * metabolism_mult;
            if predation_mode {
                move_cost *= 2.0;
            }

            current_entities[i].energy -= move_cost + idle_cost;

            // DEATH CHECK
            if current_entities[i].energy <= 0.0 {
                let event = LiveEvent::Death {
                    id: current_entities[i].id,
                    age: self.tick - current_entities[i].birth_tick,
                    offspring: current_entities[i].offspring_count,
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                    cause: "starvation".to_string(),
                };
                self.logger.log_event(event.clone())?;
                events.push(event);
                self.archive_if_legend(&current_entities[i]);
                continue;
            }

            // MOVEMENT
            current_entities[i].x += current_entities[i].vx * speed;
            current_entities[i].y += current_entities[i].vy * speed;

            if current_entities[i].x <= 0.0 {
                current_entities[i].x = 0.0;
                current_entities[i].vx = -current_entities[i].vx;
            } else if current_entities[i].x >= width_f {
                current_entities[i].x = width_f - 0.1;
                current_entities[i].vx = -current_entities[i].vx;
            }
            if current_entities[i].y <= 0.0 {
                current_entities[i].y = 0.0;
                current_entities[i].vy = -current_entities[i].vy;
            } else if current_entities[i].y >= height_f {
                current_entities[i].y = height_f - 0.1;
                current_entities[i].vy = -current_entities[i].vy;
            }

            // PREDATION
            if predation_mode {
                let targets =
                    self.spatial_hash
                        .query(current_entities[i].x, current_entities[i].y, 1.5);
                for t_idx in targets {
                    let (v_id, _vx, _vy, v_energy, v_birth, v_offspring) = entity_snapshots[t_idx];
                    if v_id != current_entities[i].id && !killed_ids.contains(&v_id) {
                        if v_energy < current_entities[i].energy {
                            current_entities[i].energy += v_energy * 0.8;
                            if current_entities[i].energy > current_entities[i].max_energy {
                                current_entities[i].energy = current_entities[i].max_energy;
                            }
                            killed_ids.insert(v_id);

                            let event = LiveEvent::Death {
                                id: v_id,
                                age: self.tick - v_birth,
                                offspring: v_offspring,
                                tick: self.tick,
                                timestamp: Utc::now().to_rfc3339(),
                                cause: "predation".to_string(),
                            };
                            let _ = self.logger.log_event(event.clone());
                            events.push(event);
                        }
                    }
                }
            }

            // EATING FOOD
            let mut eaten_idx = None;
            for (f_idx, f) in self.food.iter().enumerate() {
                let dx = current_entities[i].x - f64::from(f.x);
                let dy = current_entities[i].y - f64::from(f.y);
                if (dx * dx + dy * dy) < 1.5 {
                    eaten_idx = Some(f_idx);
                    break;
                }
            }
            if let Some(f_idx) = eaten_idx {
                current_entities[i].energy += self.config.metabolism.food_value;
                if current_entities[i].energy > current_entities[i].max_energy {
                    current_entities[i].energy = current_entities[i].max_energy;
                }
                self.food.swap_remove(f_idx);
            }

            // REPRODUCTION & GENETIC SYNERGY (Sexual Crossover)
            if current_entities[i].energy > self.config.metabolism.reproduction_threshold {
                // Look for mate nearby
                let mate_indices =
                    self.spatial_hash
                        .query(current_entities[i].x, current_entities[i].y, 2.0);
                let mut mate_idx = None;
                for m_idx in mate_indices {
                    if m_idx != i
                        && !killed_ids.contains(&current_entities[m_idx].id)
                        && current_entities[m_idx].energy > 100.0
                    {
                        mate_idx = Some(m_idx);
                        break;
                    }
                }

                let baby = if let Some(m_idx) = mate_idx {
                    // Sexual Reproduction
                    let mate = &current_entities[m_idx];
                    let mut child_brain = Brain::crossover(&current_entities[i].brain, &mate.brain);
                    child_brain.mutate_with_config(&self.config.evolution);

                    let child = current_entities[i].reproduce_with_mate(self.tick, child_brain);
                    current_entities[i].energy -= 50.0;
                    // We don't deduct energy from mate to avoid killing it in the loop (mate might have already run its tick)
                    child
                } else {
                    // Asexual fallback
                    current_entities[i].reproduce(self.tick, &self.config.evolution)
                };

                let event = LiveEvent::Birth {
                    id: baby.id,
                    parent_id: Some(current_entities[i].id),
                    gen: baby.generation,
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = self.logger.log_event(event.clone());
                events.push(event);
                new_babies.push(baby);
            }
        }

        // Finalize alive entities and archive dead ones
        for e in current_entities {
            if killed_ids.contains(&e.id) {
                self.archive_if_legend(&e);
                continue;
            }
            if e.energy > 0.0 {
                alive_entities.push(e);
            }
        }

        self.entities = alive_entities;
        self.entities.append(&mut new_babies);

        if self.entities.is_empty() && self.tick > 0 {
            let event = LiveEvent::Extinction {
                population: 0,
                tick: self.tick,
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(event.clone());
            events.push(event);
        }

        Ok(events)
    }

    fn sense_nearest_food(&self, entity: &Entity) -> (f64, f64) {
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
        (dx_food, dy_food)
    }

    fn archive_if_legend(&self, entity: &Entity) {
        let lifespan = self.tick - entity.birth_tick;
        if lifespan > 1000 || entity.offspring_count > 10 || entity.peak_energy > 300.0 {
            let _ = self.logger.archive_legend(Legend {
                id: entity.id,
                parent_id: entity.parent_id,
                birth_tick: entity.birth_tick,
                death_tick: self.tick,
                lifespan,
                generation: entity.generation,
                offspring_count: entity.offspring_count,
                peak_energy: entity.peak_energy,
                birth_timestamp: "".to_string(),
                death_timestamp: Utc::now().to_rfc3339(),
                brain_dna: entity.brain.clone(),
                color_rgb: (entity.r, entity.g, entity.b),
            });
        }
    }
}
