use crate::model::brain::Brain;
use crate::model::config::AppConfig;
use crate::model::entity::Entity;
use crate::model::environment::Environment;
use crate::model::food::Food;
use crate::model::history::{HistoryLogger, Legend, LiveEvent, PopulationStats};
use crate::model::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::quadtree::SpatialHash;
use crate::model::terrain::TerrainGrid;
use chrono::Utc;
use rand::Rng;
use std::collections::HashSet;

pub struct HallOfFame {
    pub top_living: Vec<(f64, Entity)>,
}

impl HallOfFame {
    pub fn new() -> Self {
        Self {
            top_living: Vec::with_capacity(3),
        }
    }
    pub fn update(&mut self, entities: &[Entity], tick: u64) {
        let mut scores: Vec<(f64, Entity)> = entities
            .iter()
            .map(|e| {
                let age = tick - e.birth_tick;
                let score =
                    (age as f64 * 0.5) + (e.offspring_count as f64 * 10.0) + (e.peak_energy * 0.2);
                (score, e.clone())
            })
            .collect();
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        self.top_living = scores.into_iter().take(3).collect();
    }
}

pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
    pub logger: HistoryLogger,
    pub config: AppConfig,
    pub spatial_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid, // NEW: Pheromone layer
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
        // Generate terrain with tick-based seed for variety
        let terrain = TerrainGrid::generate(
            config.world.width,
            config.world.height,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(42),
        );

        let pheromones = PheromoneGrid::new(config.world.width, config.world.height);

        Ok(Self {
            width: config.world.width,
            height: config.world.height,
            entities,
            food,
            tick: 0,
            logger,
            config,
            spatial_hash: SpatialHash::new(5.0),
            pop_stats: PopulationStats::new(),
            hall_of_fame: HallOfFame::new(),
            terrain,
            pheromones,
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

        // Game Mode Logic
        match self.config.game_mode {
            crate::model::config::GameMode::BattleRoyale => {
                // Shrinking border: Reduce safe area by 0.1 every 10 ticks (example)
                let shrink_amount = (self.tick as f64 / 100.0).min(width_f / 2.0 - 5.0);
                // Entities outside this range take massive damage
                for e in &mut self.entities {
                    if e.x < shrink_amount || e.x > width_f - shrink_amount || e.y < shrink_amount || e.y > height_f - shrink_amount {
                        e.energy -= 10.0; // The fog hurts
                    }
                }
            },
            _ => {}
        }

        self.spatial_hash.clear();
        for (i, e) in self.entities.iter().enumerate() {
            self.spatial_hash.insert(e.x, e.y, i);
        }

        // Decay pheromones each tick
        self.pheromones.decay();

        // Terrain-aware food spawning
        let base_spawn_chance = 0.0083 * food_spawn_mult;
        if self.food.len() < self.config.world.max_food {
            // Try multiple times with terrain weighting
            for _ in 0..3 {
                let x = rng.gen_range(1..self.width - 1);
                let y = rng.gen_range(1..self.height - 1);
                let terrain_mod = self.terrain.food_spawn_modifier(f64::from(x), f64::from(y));
                if terrain_mod > 0.0 && rng.gen::<f64>() < base_spawn_chance * terrain_mod {
                    self.food.push(Food::new(x, y));
                    break;
                }
            }
        }

        let mut new_babies = Vec::new();
        let mut alive_entities = Vec::new();
        let mut killed_ids = HashSet::new();
        let mut current_entities = std::mem::take(&mut self.entities);
        let entity_snapshots: Vec<(uuid::Uuid, f64, f64, f64, u64, u32)> = current_entities
            .iter()
            .map(|e| (e.id, e.x, e.y, e.energy, e.birth_tick, e.offspring_count))
            .collect();

        for i in 0..current_entities.len() {
            if killed_ids.contains(&current_entities[i].id) {
                continue;
            }
            let (dx_f, dy_f) = self.sense_nearest_food(&current_entities[i]);

            // Count same-tribe entities nearby
            let nearby_indices = self.spatial_hash.query(current_entities[i].x, current_entities[i].y, 5.0);
            let tribe_count = nearby_indices.iter().filter(|&&idx| {
                idx != i && current_entities[i].same_tribe(&current_entities[idx])
            }).count();

            // Sense pheromones
            let (pheromone_food, _pheromone_danger) = self.pheromones.sense(
                current_entities[i].x, current_entities[i].y, 3.0
            );

            let inputs = [
                (dx_f / 20.0).clamp(-1.0, 1.0) as f32,
                (dy_f / 20.0).clamp(-1.0, 1.0) as f32,
                (current_entities[i].energy / current_entities[i].max_energy) as f32,
                (nearby_indices.len().saturating_sub(1) as f32 / 10.0).min(1.0),
                pheromone_food,                      // NEW: Pheromone food input
                (tribe_count as f32 / 5.0).min(1.0), // NEW: Tribe density input
            ];
            let outputs = current_entities[i].brain.forward(inputs);
            let speed = 1.0 + (outputs[2] as f64 + 1.0) / 2.0;
            let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
            current_entities[i].last_aggression = (outputs[3] + 1.0) / 2.0;
            current_entities[i].last_share_intent = (outputs[4] + 1.0) / 2.0; // NEW: Share intent
            current_entities[i].vx = current_entities[i].vx * 0.8 + (outputs[0] as f64) * 0.2;
            current_entities[i].vy = current_entities[i].vy * 0.8 + (outputs[1] as f64) * 0.2;
            let mut move_cost = self.config.metabolism.base_move_cost * metabolism_mult * speed;
            if predation_mode {
                move_cost *= 2.0;
            }
            current_entities[i].energy -=
                move_cost + self.config.metabolism.base_idle_cost * metabolism_mult;

            if current_entities[i].energy <= 0.0 {
                let age = self.tick - current_entities[i].birth_tick;
                self.pop_stats.record_death(age);
                let ev = LiveEvent::Death {
                    id: current_entities[i].id,
                    age,
                    offspring: current_entities[i].offspring_count,
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                    cause: "starvation".to_string(),
                };
                let _ = self.logger.log_event(ev.clone());
                events.push(ev);
                self.archive_if_legend(&current_entities[i]);
                continue;
            }
            // Apply terrain movement modifier
            let terrain_speed_mod = self.terrain.movement_modifier(
                current_entities[i].x,
                current_entities[i].y,
            );
            current_entities[i].x += current_entities[i].vx * speed * terrain_speed_mod;
            current_entities[i].y += current_entities[i].vy * speed * terrain_speed_mod;
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

            // Predation with tribe protection
            if predation_mode {
                let territorial_bonus = current_entities[i].territorial_aggression();
                for t_idx in
                    self.spatial_hash
                        .query(current_entities[i].x, current_entities[i].y, 1.5)
                {
                    let (v_id, _, _, v_e, v_b, v_o) = entity_snapshots[t_idx];
                    // Don't attack same-tribe members
                    // Cooperate Mode: No attacks at all
                    let can_attack = match self.config.game_mode {
                        crate::model::config::GameMode::Cooperative => false,
                        _ => true,
                    };

                    if can_attack
                        && v_id != current_entities[i].id
                        && !killed_ids.contains(&v_id)
                        && v_e < current_entities[i].energy * territorial_bonus
                        && !current_entities[i].same_tribe(&current_entities[t_idx])
                    {
                        current_entities[i].energy += v_e * 0.8;
                        if current_entities[i].energy > current_entities[i].max_energy {
                            current_entities[i].energy = current_entities[i].max_energy;
                        }
                        killed_ids.insert(v_id);
                        // Deposit danger pheromone at kill site
                        self.pheromones.deposit(current_entities[i].x, current_entities[i].y,
                                                PheromoneType::Danger, 0.5);
                        let v_age = self.tick - v_b;
                        self.pop_stats.record_death(v_age);
                        let ev = LiveEvent::Death {
                            id: v_id,
                            age: v_age,
                            offspring: v_o,
                            tick: self.tick,
                            timestamp: Utc::now().to_rfc3339(),
                            cause: "predation".to_string(),
                        };
                        let _ = self.logger.log_event(ev.clone());
                        events.push(ev);
                    }
                }
            }
            let mut eaten_idx = None;
            for (f_idx, f) in self.food.iter().enumerate() {
                if (current_entities[i].x - f64::from(f.x)).powi(2)
                    + (current_entities[i].y - f64::from(f.y)).powi(2)
                    < 2.25
                {
                    eaten_idx = Some(f_idx);
                    break;
                }
            }
            if let Some(f_idx) = eaten_idx {
                current_entities[i].energy += self.config.metabolism.food_value;
                if current_entities[i].energy > current_entities[i].max_energy {
                    current_entities[i].energy = current_entities[i].max_energy;
                }
                // Deposit food pheromone when eating
                self.pheromones.deposit(current_entities[i].x, current_entities[i].y,
                                        PheromoneType::Food, 0.3);
                self.food.swap_remove(f_idx);
            }

            if current_entities[i].energy > self.config.metabolism.reproduction_threshold {
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
                    let mut cb = Brain::crossover(
                        &current_entities[i].brain,
                        &current_entities[m_idx].brain,
                    );
                    cb.mutate_with_config(&self.config.evolution);
                    let c = current_entities[i].reproduce_with_mate(self.tick, cb);
                    current_entities[i].energy -= 50.0;
                    c
                } else {
                    current_entities[i].reproduce(self.tick, &self.config.evolution)
                };
                let ev = LiveEvent::Birth {
                    id: baby.id,
                    parent_id: Some(current_entities[i].id),
                    gen: baby.generation,
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = self.logger.log_event(ev.clone());
                events.push(ev);
                new_babies.push(baby);
            }
        }
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
            let ev = LiveEvent::Extinction {
                population: 0,
                tick: self.tick,
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(ev.clone());
            events.push(ev);
        }
        if self.tick % 60 == 0 {
            self.hall_of_fame.update(&self.entities, self.tick);
            let top_fitness = self
                .hall_of_fame
                .top_living
                .first()
                .map(|(s, _)| *s)
                .unwrap_or(0.0);
            self.pop_stats.update_snapshot(&self.entities, top_fitness);
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
