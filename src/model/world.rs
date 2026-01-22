use crate::model::brain::Brain;
use crate::model::config::AppConfig;
use crate::model::entity::Entity;
use crate::model::environment::Environment;
use crate::model::food::Food;
use crate::model::history::{HistoryLogger, LiveEvent, PopulationStats};
use crate::model::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::quadtree::SpatialHash;
use crate::model::systems::{action, biological, ecological, social};
use crate::model::terrain::TerrainGrid;
use chrono::Utc;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashSet;

pub struct EntitySnapshot {
    pub id: uuid::Uuid,
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub birth_tick: u64,
    pub offspring_count: u32,
}

pub struct HallOfFame {
    pub top_living: Vec<(f64, Entity)>,
}

impl Default for HallOfFame {
    fn default() -> Self {
        Self::new()
    }
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
                let age = tick - e.metabolism.birth_tick;
                let score = (age as f64 * 0.5)
                    + (e.metabolism.offspring_count as f64 * 10.0)
                    + (e.metabolism.peak_energy * 0.2);
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
    pub food_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid,
    pub active_pathogens: Vec<crate::model::pathogen::Pathogen>,

    // Reusable buffers to reduce allocation jitter
    killed_ids: HashSet<uuid::Uuid>,
    eaten_food_indices: HashSet<usize>,
    new_babies: Vec<Entity>,
    alive_entities: Vec<Entity>,
    perception_buffer: Vec<[f32; 6]>,
    decision_buffer: Vec<([f32; 5], [f32; 6])>,
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
                gen: entity.metabolism.generation,
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
            food_hash: SpatialHash::new(5.0),
            pop_stats: PopulationStats::new(),
            hall_of_fame: HallOfFame::new(),
            terrain,
            pheromones,
            active_pathogens: Vec::new(),
            killed_ids: HashSet::new(),
            eaten_food_indices: HashSet::new(),
            new_babies: Vec::new(),
            alive_entities: Vec::new(),
            perception_buffer: Vec::new(),
            decision_buffer: Vec::new(),
        })
    }

    pub fn update(&mut self, env: &Environment) -> anyhow::Result<Vec<LiveEvent>> {
        let mut events = Vec::new();
        self.tick += 1;
        let mut rng = rand::thread_rng();

        self.handle_game_modes();
        self.pheromones.decay();

        // Trigger Dust Bowl disaster
        if env.is_heat_wave() && self.entities.len() > 300 && rng.gen_bool(0.01) {
            self.terrain.trigger_dust_bowl(500);
        }
        self.terrain.update();

        self.handle_pathogen_emergence(&mut rng);
        self.spawn_food(env, &mut rng);

        self.food_hash.clear();
        for (i, f) in self.food.iter().enumerate() {
            self.food_hash.insert(f64::from(f.x), f64::from(f.y), i);
        }

        let mut current_entities = std::mem::take(&mut self.entities);
        self.spatial_hash.clear();
        for (i, e) in current_entities.iter().enumerate() {
            self.spatial_hash.insert(e.physics.x, e.physics.y, i);
        }

        let entity_snapshots: Vec<EntitySnapshot> = current_entities
            .iter()
            .map(|e| EntitySnapshot {
                id: e.id,
                x: e.physics.x,
                y: e.physics.y,
                energy: e.metabolism.energy,
                birth_tick: e.metabolism.birth_tick,
                offspring_count: e.metabolism.offspring_count,
            })
            .collect();

        let mut killed_ids = std::mem::take(&mut self.killed_ids);
        let mut eaten_food_indices = std::mem::take(&mut self.eaten_food_indices);
        let mut new_babies = std::mem::take(&mut self.new_babies);
        let mut alive_entities = std::mem::take(&mut self.alive_entities);
        let mut perception_buffer = std::mem::take(&mut self.perception_buffer);
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);

        killed_ids.clear();
        eaten_food_indices.clear();
        new_babies.clear();
        alive_entities.clear();

        current_entities
            .par_iter()
            .enumerate()
            .map(|(i, e)| {
                let (dx_f, dy_f) = self.sense_nearest_food(e);
                let nearby_indices = self.spatial_hash.query(e.physics.x, e.physics.y, 5.0);
                let (pheromone_food, _) = self.pheromones.sense(e.physics.x, e.physics.y, 3.0);
                let tribe_count = nearby_indices
                    .iter()
                    .filter(|&&n_idx| n_idx != i && e.same_tribe(&current_entities[n_idx]))
                    .count();
                [
                    (dx_f / 20.0).clamp(-1.0, 1.0) as f32,
                    (dy_f / 20.0).clamp(-1.0, 1.0) as f32,
                    (e.metabolism.energy / e.metabolism.max_energy) as f32,
                    (nearby_indices.len().saturating_sub(1) as f32 / 10.0).min(1.0),
                    pheromone_food,
                    (tribe_count as f32 / 5.0).min(1.0),
                ]
            })
            .collect_into_vec(&mut perception_buffer);

        current_entities
            .par_iter()
            .zip(perception_buffer.par_iter())
            .map(|(e, inputs)| e.intel.brain.forward(*inputs, e.intel.last_hidden))
            .collect_into_vec(&mut decision_buffer);

        for i in 0..current_entities.len() {
            if killed_ids.contains(&current_entities[i].id) {
                continue;
            }
            let (outputs, next_hidden) = decision_buffer[i];
            current_entities[i].intel.last_hidden = next_hidden;
            self.action_system(&mut current_entities[i], outputs, env);
            if current_entities[i].metabolism.energy <= 0.0 {
                self.handle_death(i, &mut current_entities, &mut events, "starvation");
                continue;
            }
            self.biological_system(&mut current_entities[i], &mut rng);
            self.handle_infection(i, &mut current_entities, &killed_ids, &mut rng);
            let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
            if predation_mode {
                self.handle_predation(
                    i,
                    &mut current_entities,
                    &entity_snapshots,
                    &mut killed_ids,
                    &mut events,
                );
            }
            self.handle_feeding_optimized(i, &mut current_entities, &mut eaten_food_indices);
            if let Some(baby) = self.handle_reproduction(i, &mut current_entities, &killed_ids) {
                let ev = LiveEvent::Birth {
                    id: baby.id,
                    parent_id: Some(current_entities[i].id),
                    gen: baby.metabolism.generation,
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
            if e.metabolism.energy > 0.0 {
                alive_entities.push(e);
            }
        }
        self.entities.append(&mut alive_entities);
        self.entities.append(&mut new_babies);

        if !eaten_food_indices.is_empty() {
            let mut i = 0;
            self.food.retain(|_| {
                let keep = !eaten_food_indices.contains(&i);
                i += 1;
                keep
            });
        }

        self.killed_ids = killed_ids;
        self.eaten_food_indices = eaten_food_indices;
        self.new_babies = new_babies;
        self.alive_entities = alive_entities;
        self.perception_buffer = perception_buffer;
        self.decision_buffer = decision_buffer;

        self.handle_extinction(&mut events);
        self.update_stats();
        Ok(events)
    }

    fn action_system(&mut self, entity: &mut Entity, outputs: [f32; 5], env: &Environment) {
        let speed_mult = 1.0 + (outputs[2] as f64 + 1.0) / 2.0;
        let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
        entity.intel.last_aggression = (outputs[3] + 1.0) / 2.0;
        entity.intel.last_share_intent = (outputs[4] + 1.0) / 2.0;
        entity.physics.vx = entity.physics.vx * 0.8 + (outputs[0] as f64) * 0.2;
        entity.physics.vy = entity.physics.vy * 0.8 + (outputs[1] as f64) * 0.2;
        let metabolism_mult = env.metabolism_multiplier();
        let mut move_cost = self.config.metabolism.base_move_cost * metabolism_mult * speed_mult;
        if predation_mode {
            move_cost *= 2.0;
        }
        entity.metabolism.energy -=
            move_cost + self.config.metabolism.base_idle_cost * metabolism_mult;
        self.handle_movement(entity, speed_mult);
    }

    fn biological_system(&mut self, entity: &mut Entity, _rng: &mut impl Rng) {
        biological::biological_system(entity);
    }

    fn handle_game_modes(&mut self) {
        action::handle_game_modes(
            &mut self.entities,
            &self.config,
            self.tick,
            self.width,
            self.height,
        );
    }

    fn handle_pathogen_emergence(&mut self, rng: &mut impl Rng) {
        biological::handle_pathogen_emergence(&mut self.active_pathogens, rng);
    }

    fn spawn_food(&mut self, env: &Environment, rng: &mut impl Rng) {
        ecological::spawn_food(
            &mut self.food,
            env,
            &self.terrain,
            self.config.world.max_food,
            self.width,
            self.height,
            rng,
        );
    }

    fn handle_death(
        &mut self,
        idx: usize,
        entities: &mut [Entity],
        events: &mut Vec<LiveEvent>,
        cause: &str,
    ) {
        let age = self.tick - entities[idx].metabolism.birth_tick;
        self.pop_stats.record_death(age);
        let ev = LiveEvent::Death {
            id: entities[idx].id,
            age,
            offspring: entities[idx].metabolism.offspring_count,
            tick: self.tick,
            timestamp: Utc::now().to_rfc3339(),
            cause: cause.to_string(),
        };
        let _ = self.logger.log_event(ev.clone());
        events.push(ev);
        self.archive_if_legend(&entities[idx]);
    }

    fn handle_movement(&self, entity: &mut Entity, speed: f64) {
        action::handle_movement(entity, speed, &self.terrain, self.width, self.height);
    }

    fn handle_infection(
        &self,
        idx: usize,
        entities: &mut [Entity],
        killed_ids: &HashSet<uuid::Uuid>,
        rng: &mut impl Rng,
    ) {
        entities[idx].process_infection();
        for p in &self.active_pathogens {
            if rng.gen_bool(0.005) {
                entities[idx].try_infect(p);
            }
        }
        if let Some(p) = entities[idx].health.pathogen.clone() {
            for n_idx in
                self.spatial_hash
                    .query(entities[idx].physics.x, entities[idx].physics.y, 2.0)
            {
                if n_idx != idx
                    && !killed_ids.contains(&entities[n_idx].id)
                    && entities[n_idx].try_infect(&p)
                {}
            }
        }
    }

    fn handle_predation(
        &mut self,
        idx: usize,
        entities: &mut [Entity],
        snapshots: &[EntitySnapshot],
        killed_ids: &mut HashSet<uuid::Uuid>,
        events: &mut Vec<LiveEvent>,
    ) {
        let territorial_bonus = entities[idx].territorial_aggression();
        let targets =
            self.spatial_hash
                .query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
        for t_idx in targets {
            let v_id = snapshots[t_idx].id;
            let v_e = snapshots[t_idx].energy;
            let v_b = snapshots[t_idx].birth_tick;
            let v_o = snapshots[t_idx].offspring_count;
            let can_attack = !matches!(
                self.config.game_mode,
                crate::model::config::GameMode::Cooperative
            );
            if can_attack
                && v_id != entities[idx].id
                && !killed_ids.contains(&v_id)
                && v_e < entities[idx].metabolism.energy * territorial_bonus
                && !entities[idx].same_tribe(&entities[t_idx])
            {
                let gain_mult = match entities[idx].metabolism.role {
                    crate::model::entity::EntityRole::Carnivore => 1.2,
                    crate::model::entity::EntityRole::Herbivore => 0.2,
                };
                entities[idx].metabolism.energy = (entities[idx].metabolism.energy
                    + v_e * gain_mult)
                    .min(entities[idx].metabolism.max_energy);
                killed_ids.insert(v_id);
                self.pheromones.deposit(
                    entities[idx].physics.x,
                    entities[idx].physics.y,
                    PheromoneType::Danger,
                    0.5,
                );
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

    fn handle_feeding_optimized(
        &mut self,
        idx: usize,
        entities: &mut [Entity],
        eaten_indices: &mut HashSet<usize>,
    ) {
        if !matches!(
            entities[idx].metabolism.role,
            crate::model::entity::EntityRole::Herbivore
        ) {
            return;
        }
        let mut eaten_idx = None;
        let nearby_food =
            self.food_hash
                .query(entities[idx].physics.x, entities[idx].physics.y, 1.5);
        for f_idx in nearby_food {
            if eaten_indices.contains(&f_idx) {
                continue;
            }
            let f = &self.food[f_idx];
            if (entities[idx].physics.x - f64::from(f.x)).powi(2)
                + (entities[idx].physics.y - f64::from(f.y)).powi(2)
                < 2.25
            {
                eaten_idx = Some(f_idx);
                break;
            }
        }
        if let Some(f_idx) = eaten_idx {
            entities[idx].metabolism.energy = (entities[idx].metabolism.energy
                + self.config.metabolism.food_value)
                .min(entities[idx].metabolism.max_energy);
            self.terrain
                .deplete(entities[idx].physics.x, entities[idx].physics.y, 0.4);
            self.pheromones.deposit(
                entities[idx].physics.x,
                entities[idx].physics.y,
                PheromoneType::Food,
                0.3,
            );
            eaten_indices.insert(f_idx);
        }
    }

    fn handle_reproduction(
        &mut self,
        idx: usize,
        entities: &mut [Entity],
        killed_ids: &HashSet<uuid::Uuid>,
    ) -> Option<Entity> {
        if !entities[idx].is_mature(self.tick, self.config.metabolism.maturity_age)
            || entities[idx].metabolism.energy <= self.config.metabolism.reproduction_threshold
        {
            return None;
        }
        let mate_indices =
            self.spatial_hash
                .query(entities[idx].physics.x, entities[idx].physics.y, 2.0);
        let mut mate_idx = None;
        for m_idx in mate_indices {
            if m_idx != idx
                && !killed_ids.contains(&entities[m_idx].id)
                && entities[m_idx].metabolism.energy > 100.0
            {
                mate_idx = Some(m_idx);
                break;
            }
        }
        if let Some(m_idx) = mate_idx {
            let mut cb = Brain::crossover(&entities[idx].intel.brain, &entities[m_idx].intel.brain);
            cb.mutate_with_config(&self.config.evolution);
            let child = entities[idx].reproduce_with_mate(
                self.tick,
                cb,
                self.config.evolution.speciation_rate,
            );
            entities[idx].metabolism.energy -= 50.0;
            Some(child)
        } else {
            Some(entities[idx].reproduce(self.tick, &self.config.evolution))
        }
    }

    fn handle_extinction(&mut self, events: &mut Vec<LiveEvent>) {
        if self.entities.is_empty() && self.tick > 0 {
            let ev = LiveEvent::Extinction {
                population: 0,
                tick: self.tick,
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(ev.clone());
            events.push(ev);
        }
    }

    fn update_stats(&mut self) {
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
    }

    fn sense_nearest_food(&self, entity: &Entity) -> (f64, f64) {
        ecological::sense_nearest_food(entity, &self.food, &self.food_hash)
    }

    fn archive_if_legend(&self, entity: &Entity) {
        social::archive_if_legend(entity, self.tick, &self.logger);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::config::AppConfig;

    #[test]
    fn test_handle_movement_wall_bounce() {
        let mut config = AppConfig::default();
        config.world.width = 10;
        config.world.height = 10;
        let mut world = World::new(0, config).unwrap();

        // Place a wall at (5, 5)
        world
            .terrain
            .set_cell_type(5, 5, crate::model::terrain::TerrainType::Wall);

        let mut entity = Entity::new(4.5, 4.5, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 1.0;

        // Move towards the wall
        // Speed 1.0, terrain mod 1.0 (Plains at 4,4)
        // next_x = 4.5 + 1.0 = 5.5 (Wall)
        world.handle_movement(&mut entity, 1.0);

        assert!(
            entity.physics.vx < 0.0,
            "Velocity X should be reversed, got {}",
            entity.physics.vx
        );
        assert!(
            entity.physics.vy < 0.0,
            "Velocity Y should be reversed, got {}",
            entity.physics.vy
        );
        assert_eq!(
            entity.physics.x, 4.5,
            "Entity should not have moved into the wall"
        );
    }
}
