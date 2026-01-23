use crate::model::config::AppConfig;
use crate::model::history::{HallOfFame, HistoryLogger, LiveEvent, PopulationStats};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::lineage_registry::LineageRegistry;
use crate::model::state::pheromone::PheromoneGrid;
use crate::model::state::terrain::TerrainGrid;
use crate::model::systems::{action, biological, ecological, environment, intel, social, stats};
use chrono::Utc;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A lightweight snapshot of an entity's state for read-only access during update cycles.
///
/// This struct captures essential entity attributes at the start of each tick,
/// allowing systems to query entity state without holding mutable borrows.
#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Unique identifier of the entity.
    pub id: uuid::Uuid,
    /// NEW: Lineage identifier.
    pub lineage_id: uuid::Uuid,
    /// X coordinate in world space.
    pub x: f64,
    /// Y coordinate in world space.
    pub y: f64,
    /// Current energy level.
    pub energy: f64,
    /// Tick at which the entity was born.
    pub birth_tick: u64,
    /// Number of offspring produced.
    pub offspring_count: u32,
    /// Red color component (0-255) for tribe identification.
    pub r: u8,
    /// Green color component (0-255) for tribe identification.
    pub g: u8,
    /// Blue color component (0-255) for tribe identification.
    pub b: u8,
}

/// The simulation world containing all entities, resources, and environmental state.
///
/// `World` is the central data structure of the simulation. It orchestrates:
/// - Entity lifecycle (birth, death, reproduction)
/// - Spatial indexing for efficient neighbor queries
/// - Environmental systems (terrain, pheromones, pathogens)
/// - Population statistics and history logging
///
/// # Example
/// ```ignore
/// let config = AppConfig::default();
/// let mut world = World::new(0, config)?;
/// let env = Environment::default();
/// world.update(&env)?;
/// ```
#[derive(Serialize, Deserialize)]
pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
    #[serde(skip, default = "HistoryLogger::new_dummy")]
    pub logger: HistoryLogger,
    pub config: AppConfig,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub spatial_hash: SpatialHash,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub food_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid,
    pub lineage_registry: LineageRegistry,
    pub active_pathogens: Vec<crate::model::state::pathogen::Pathogen>,

    // Reusable buffers to reduce allocation jitter
    #[serde(skip, default)]
    killed_ids: HashSet<uuid::Uuid>,
    #[serde(skip, default)]
    eaten_food_indices: HashSet<usize>,
    #[serde(skip, default)]
    new_babies: Vec<Entity>,
    #[serde(skip, default)]
    alive_entities: Vec<Entity>,
    #[serde(skip, default)]
    perception_buffer: Vec<[f32; 14]>,
    #[serde(skip, default)]
    decision_buffer: Vec<([f32; 8], [f32; 6])>,
    #[serde(skip, default)]
    energy_transfers: Vec<(usize, f64)>,
    #[serde(skip, default)]
    lineage_consumption: Vec<(uuid::Uuid, f64)>,
}

impl World {
    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        let mut rng = rand::thread_rng();
        let mut entities = Vec::with_capacity(initial_population);
        let logger = HistoryLogger::new()?;
        let mut lineage_registry = LineageRegistry::new();
        for _ in 0..initial_population {
            let e = Entity::new(
                rng.gen_range(1.0..config.world.width as f64 - 1.0),
                rng.gen_range(1.0..config.world.height as f64 - 1.0),
                0,
            );
            // Initial entities establish the founding lineages
            lineage_registry.record_birth(e.metabolism.lineage_id, 1, 0);
            entities.push(e);
        }

        let mut food = Vec::new();
        for _ in 0..config.world.initial_food {
            let x = rng.gen_range(1..config.world.width - 1);
            let y = rng.gen_range(1..config.world.height - 1);
            let n_type = rng.gen_range(0.0..1.0);
            food.push(Food::new(x, y, n_type));
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
            lineage_registry,
            active_pathogens: Vec::new(),
            killed_ids: HashSet::new(),
            eaten_food_indices: HashSet::new(),
            new_babies: Vec::new(),
            alive_entities: Vec::new(),
            perception_buffer: Vec::new(),
            decision_buffer: Vec::new(),
            energy_transfers: Vec::new(),
            lineage_consumption: Vec::new(),
        })
    }

    pub fn update(&mut self, env: &Environment) -> anyhow::Result<Vec<LiveEvent>> {
        let mut events = Vec::new();
        self.tick += 1;
        let mut rng = rand::thread_rng();

        action::handle_game_modes(
            &mut self.entities,
            &self.config,
            self.tick,
            self.width,
            self.height,
        );
        self.pheromones.decay();

        environment::handle_disasters(env, self.entities.len(), &mut self.terrain, &mut rng);
        self.terrain.update(self.pop_stats.biomass_h);

        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut rng);
        ecological::spawn_food(
            &mut self.food,
            env,
            &self.terrain,
            self.config.world.max_food,
            self.width,
            self.height,
            &mut rng,
        );

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
                lineage_id: e.metabolism.lineage_id,
                x: e.physics.x,
                y: e.physics.y,
                energy: e.metabolism.energy,
                birth_tick: e.metabolism.birth_tick,
                offspring_count: e.metabolism.offspring_count,
                r: e.physics.r,
                g: e.physics.g,
                b: e.physics.b,
            })
            .collect();

        let mut killed_ids = std::mem::take(&mut self.killed_ids);
        let mut eaten_food_indices = std::mem::take(&mut self.eaten_food_indices);
        let mut new_babies = std::mem::take(&mut self.new_babies);
        let mut alive_entities = std::mem::take(&mut self.alive_entities);
        let _perception_buffer = std::mem::take(&mut self.perception_buffer);
        let _decision_buffer = std::mem::take(&mut self.decision_buffer);
        let mut energy_transfers = std::mem::take(&mut self.energy_transfers);

        killed_ids.clear();
        eaten_food_indices.clear();
        new_babies.clear();
        alive_entities.clear();
        energy_transfers.clear();

        let mut perception_buffer: Vec<[f32; 14]> = Vec::with_capacity(current_entities.len());
        let mut decision_buffer: Vec<([f32; 8], [f32; 6])> =
            Vec::with_capacity(current_entities.len());

        current_entities
            .par_iter()
            .enumerate()
            .map(|(i, e)| {
                let (dx_f, dy_f, f_type) =
                    ecological::sense_nearest_food(e, &self.food, &self.food_hash);
                let sensing_radius = e.physics.sensing_range;
                let nearby_indices =
                    self.spatial_hash
                        .query(e.physics.x, e.physics.y, sensing_radius);
                let (pheromone_food, _, pheromone_a, pheromone_b) =
                    self.pheromones
                        .sense_all(e.physics.x, e.physics.y, sensing_radius / 2.0);
                let tribe_count = nearby_indices
                    .iter()
                    .filter(|&&n_idx| {
                        n_idx != i && social::are_same_tribe(e, &current_entities[n_idx])
                    })
                    .count();
                let mut kin_dx = 0.0;
                let mut kin_dy = 0.0;
                let mut kin_count = 0;
                for &n_idx in &nearby_indices {
                    if n_idx != i
                        && current_entities[n_idx].metabolism.lineage_id == e.metabolism.lineage_id
                    {
                        kin_dx += current_entities[n_idx].physics.x - e.physics.x;
                        kin_dy += current_entities[n_idx].physics.y - e.physics.y;
                        kin_count += 1;
                    }
                }
                let kin_vec_x = if kin_count > 0 {
                    (kin_dx / kin_count as f64).clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                let kin_vec_y = if kin_count > 0 {
                    (kin_dy / kin_count as f64).clamp(-1.0, 1.0)
                } else {
                    0.0
                };

                // Wall sensing
                let mut wall_proximity = 0.0;
                if e.physics.x < 5.0
                    || e.physics.x > (self.width - 5) as f64
                    || e.physics.y < 5.0
                    || e.physics.y > (self.height - 5) as f64
                {
                    wall_proximity = 1.0;
                }

                [
                    (dx_f / (sensing_radius * 4.0)).clamp(-1.0, 1.0) as f32,
                    (dy_f / (sensing_radius * 4.0)).clamp(-1.0, 1.0) as f32,
                    (e.metabolism.energy / e.metabolism.max_energy) as f32,
                    (nearby_indices.len().saturating_sub(1) as f32 / 10.0).min(1.0),
                    pheromone_food,
                    (tribe_count as f32 / 5.0).min(1.0),
                    kin_vec_x as f32,
                    kin_vec_y as f32,
                    pheromone_a,
                    pheromone_b,
                    wall_proximity as f32,
                    (e.metabolism.birth_tick as f32 / self.tick.max(1) as f32).min(1.0),
                    f_type,
                    e.metabolism.trophic_potential, // 14th Input: Trophic Potential
                ]
            })
            .collect_into_vec(&mut perception_buffer);

        current_entities
            .par_iter()
            .zip(perception_buffer.par_iter())
            .map(|(e, inputs)| {
                intel::brain_forward(&e.intel.genotype.brain, *inputs, e.intel.last_hidden)
            })
            .collect_into_vec(&mut decision_buffer);

        for i in 0..current_entities.len() {
            if killed_ids.contains(&current_entities[i].id) {
                continue;
            }
            let (outputs, next_hidden) = decision_buffer[i];
            current_entities[i].intel.last_hidden = next_hidden;

            let mut ctx = action::ActionContext {
                env,
                config: &self.config,
                terrain: &self.terrain,
                pheromones: &mut self.pheromones,
                width: self.width,
                height: self.height,
            };
            action::action_system(&mut current_entities[i], outputs, &mut ctx);

            // NEW: Phase 30 - Herding Reward
            // Use perceived kin vector from perception_buffer[i][6/7]
            let kin_vx = perception_buffer[i][6] as f64;
            let kin_vy = perception_buffer[i][7] as f64;
            if kin_vx != 0.0 || kin_vy != 0.0 {
                let dot_product = current_entities[i].physics.vx * kin_vx
                    + current_entities[i].physics.vy * kin_vy;
                if dot_product > 0.5 {
                    current_entities[i].metabolism.energy += 0.05; // Cooperation bonus
                }
            }

            if current_entities[i].metabolism.energy <= 0.0 {
                biological::handle_death(
                    i,
                    &current_entities,
                    self.tick,
                    "starvation",
                    &mut self.pop_stats,
                    &mut events,
                    &mut self.logger,
                );
                continue;
            }

            biological::biological_system(&mut current_entities[i]);
            biological::handle_infection(
                i,
                &mut current_entities,
                &killed_ids,
                &self.active_pathogens,
                &self.spatial_hash,
                &mut rng,
            );

            let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
            if predation_mode {
                let mut ctx = social::PredationContext {
                    snapshots: &entity_snapshots,
                    killed_ids: &mut killed_ids,
                    events: &mut events,
                    config: &self.config,
                    spatial_hash: &self.spatial_hash,
                    pheromones: &mut self.pheromones,
                    pop_stats: &mut self.pop_stats,
                    logger: &mut self.logger,
                    tick: self.tick,
                    energy_transfers: &mut energy_transfers,
                    lineage_consumption: &mut self.lineage_consumption,
                };
                social::handle_predation(i, &mut current_entities, &mut ctx);
            }

            // Energy Sharing
            let mut share_ctx = social::PredationContext {
                snapshots: &entity_snapshots,
                killed_ids: &mut killed_ids,
                events: &mut events,
                config: &self.config,
                spatial_hash: &self.spatial_hash,
                pheromones: &mut self.pheromones,
                pop_stats: &mut self.pop_stats,
                logger: &mut self.logger,
                tick: self.tick,
                energy_transfers: &mut energy_transfers,
                lineage_consumption: &mut self.lineage_consumption,
            };
            social::handle_sharing(i, &mut current_entities, &mut share_ctx);

            let mut feed_ctx = ecological::FeedingContext {
                food: &self.food,
                food_hash: &self.food_hash,
                eaten_indices: &mut eaten_food_indices,
                terrain: &mut self.terrain,
                pheromones: &mut self.pheromones,
                food_value: self.config.metabolism.food_value,
                lineage_consumption: &mut self.lineage_consumption,
            };
            ecological::handle_feeding_optimized(i, &mut current_entities, &mut feed_ctx);

            if let Some(baby) = social::handle_reproduction(
                i,
                &mut current_entities,
                &killed_ids,
                &self.spatial_hash,
                &self.config,
                self.tick,
            ) {
                // Register birth
                self.lineage_registry.record_birth(
                    baby.metabolism.lineage_id,
                    baby.metabolism.generation,
                    self.tick,
                );

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

        // Apply lineage consumption
        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();

        // Apply energy transfers
        for (target_idx, amount) in &energy_transfers {
            if *target_idx < current_entities.len()
                && !killed_ids.contains(&current_entities[*target_idx].id)
            {
                current_entities[*target_idx].metabolism.energy =
                    (current_entities[*target_idx].metabolism.energy + amount)
                        .min(current_entities[*target_idx].metabolism.max_energy);
            }
        }

        for e in current_entities {
            if killed_ids.contains(&e.id) {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
                social::archive_if_legend(&e, self.tick, &self.logger);
                continue;
            }
            if e.metabolism.energy <= 0.0 {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
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
        self.energy_transfers = energy_transfers;

        social::handle_extinction(&self.entities, self.tick, &mut events, &mut self.logger);
        stats::update_stats(
            self.tick,
            &self.entities,
            self.food.len(),
            &mut self.pop_stats,
            &mut self.hall_of_fame,
        );
        if self.tick % 1000 == 0 {
            let _ = self.lineage_registry.save("logs/lineages.json");
        }

        Ok(events)
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
            .set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Wall);

        let mut entity = Entity::new(4.5, 4.5, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 1.0;

        // Move towards the wall
        // Speed 1.0, terrain mod 1.0 (Plains at 4,4)
        // next_x = 4.5 + 1.0 = 5.5 (Wall)
        action::handle_movement(&mut entity, 1.0, &world.terrain, world.width, world.height);

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
