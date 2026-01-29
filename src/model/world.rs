use crate::model::config::AppConfig;
use crate::model::environment::Environment;
use crate::model::food::Food;
use crate::model::history::{
    FossilRegistry, HallOfFame, HistoryLogger, LiveEvent, PopulationStats,
};
use crate::model::interaction::InteractionCommand;
use crate::model::lineage_registry::LineageRegistry;
use crate::model::observer::WorldObserver;
use crate::model::pheromone::PheromoneGrid;
use crate::model::sound::SoundGrid;
use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::TerrainGrid;
use chrono::Utc;
use hecs;
use primordium_data::Entity;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::model::brain::BrainLogic;
use crate::model::lifecycle;
use crate::model::systems::{
    action, biological, civilization, ecological, environment, history, interaction, social, stats,
};
use primordium_data::{GeneType, MetabolicNiche, Position};
use social::ReproductionContext;

#[derive(Serialize, Deserialize, Clone)]
pub struct InternalEntitySnapshot {
    pub id: uuid::Uuid,
    pub lineage_id: uuid::Uuid,
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub birth_tick: u64,
    pub offspring_count: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub rank: f32,
    pub status: primordium_data::EntityStatus,
}

/// Result of a neural network forward pass for an entity.
#[derive(Default)]
pub struct EntityDecision {
    pub outputs: [f32; 12],
    pub next_hidden: [f32; 6],
    pub activations: primordium_data::Activations,
}

/// The main simulation universe containing all entities, terrain, and state.
#[derive(Serialize, Deserialize)]
pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
    #[serde(skip, default = "hecs::World::new")]
    pub ecs: hecs::World,
    #[serde(skip, default = "HistoryLogger::new_dummy")]
    pub logger: HistoryLogger,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub spatial_hash: SpatialHash,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub food_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid,
    pub sound: SoundGrid,
    pub pressure: crate::model::pressure::PressureGrid,
    pub social_grid: Vec<u8>,
    pub lineage_registry: LineageRegistry,
    pub fossil_registry: FossilRegistry,
    pub config: AppConfig,
    pub log_dir: String,
    pub active_pathogens: Vec<primordium_data::Pathogen>,
    #[serde(skip, default = "WorldObserver::new")]
    pub observer: WorldObserver,
    #[serde(skip, default)]
    pub best_legends: HashMap<uuid::Uuid, crate::model::history::Legend>,
    #[serde(skip, default = "ChaCha8Rng::from_entropy")]
    pub rng: ChaCha8Rng,
    #[serde(skip, default)]
    killed_ids: HashSet<uuid::Uuid>,
    #[serde(skip, default)]
    eaten_food_indices: HashSet<usize>,
    #[serde(skip, default)]
    new_babies: Vec<Entity>,
    #[serde(skip, default)]
    alive_entities: Vec<Entity>,
    #[serde(skip, default)]
    perception_buffer: Vec<[f32; 29]>,
    #[serde(skip, default)]
    decision_buffer: Vec<EntityDecision>,
    #[serde(skip, default)]
    lineage_consumption: Vec<(uuid::Uuid, f64)>,
    #[serde(skip, default)]
    entity_snapshots: Vec<InternalEntitySnapshot>,
    #[serde(skip)]
    cached_terrain: std::sync::Arc<TerrainGrid>,
    #[serde(skip)]
    cached_pheromones: std::sync::Arc<PheromoneGrid>,
    #[serde(skip)]
    cached_sound: std::sync::Arc<SoundGrid>,
    #[serde(skip)]
    cached_pressure: std::sync::Arc<crate::model::pressure::PressureGrid>,
    #[serde(skip)]
    cached_social_grid: std::sync::Arc<Vec<u8>>,
    #[serde(skip, default)]
    food_dirty: bool,
    #[serde(skip, default)]
    spatial_data_buffer: Vec<(f64, f64, uuid::Uuid)>,
    #[serde(skip, default)]
    food_positions_buffer: Vec<(f64, f64)>,
}

impl World {
    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        Self::new_at(initial_population, config, "logs")
    }
    pub fn new_at(
        initial_population: usize,
        config: AppConfig,
        log_dir: &str,
    ) -> anyhow::Result<Self> {
        let mut rng = if let Some(seed) = config.world.seed {
            ChaCha8Rng::seed_from_u64(seed)
        } else {
            ChaCha8Rng::from_entropy()
        };
        let mut entities = Vec::with_capacity(initial_population);
        let logger = HistoryLogger::new_at(log_dir)?;
        let mut lineage_registry = LineageRegistry::new();
        for _ in 0..initial_population {
            let e = lifecycle::create_entity_with_rng(
                rng.gen_range(1.0..config.world.width as f64 - 1.0),
                rng.gen_range(1.0..config.world.height as f64 - 1.0),
                0,
                &mut rng,
            );
            lineage_registry.record_birth(e.metabolism.lineage_id, 1, 0);
            entities.push(e);
        }
        let mut food = Vec::new();
        let mut ecs = hecs::World::new();
        for _ in 0..config.world.initial_food {
            let fx = rng.gen_range(1..config.world.width - 1);
            let fy = rng.gen_range(1..config.world.height - 1);
            let n_type = rng.gen_range(0.0..1.0);
            food.push(Food::new(fx, fy, n_type));
            ecs.spawn((
                Position {
                    x: fx as f64,
                    y: fy as f64,
                },
                MetabolicNiche(n_type),
            ));
        }
        let terrain = TerrainGrid::generate(config.world.width, config.world.height, 42);
        let pheromones = PheromoneGrid::new(config.world.width, config.world.height);
        let sound = SoundGrid::new(config.world.width, config.world.height);
        let pressure =
            crate::model::pressure::PressureGrid::new(config.world.width, config.world.height);
        let social_grid = vec![0; config.world.width as usize * config.world.height as usize];

        Ok(Self {
            width: config.world.width,
            height: config.world.height,
            entities,
            food,
            tick: 0,
            ecs,
            logger,
            spatial_hash: SpatialHash::new(5.0, config.world.width, config.world.height),
            food_hash: SpatialHash::new(5.0, config.world.width, config.world.height),
            pop_stats: PopulationStats::default(),
            hall_of_fame: HallOfFame {
                top_living: Vec::new(),
            },
            cached_terrain: std::sync::Arc::new(terrain.clone()),
            cached_pheromones: std::sync::Arc::new(pheromones.clone()),
            cached_sound: std::sync::Arc::new(sound.clone()),
            cached_pressure: std::sync::Arc::new(pressure.clone()),
            cached_social_grid: std::sync::Arc::new(social_grid.clone()),
            terrain,
            pheromones,
            sound,
            pressure,
            social_grid,
            lineage_registry,
            config,
            fossil_registry: FossilRegistry::default(),
            log_dir: log_dir.to_string(),
            active_pathogens: Vec::new(),
            observer: WorldObserver::new(),
            best_legends: HashMap::new(),
            rng,
            killed_ids: HashSet::new(),
            eaten_food_indices: HashSet::new(),
            new_babies: Vec::new(),
            alive_entities: Vec::new(),
            perception_buffer: Vec::new(),
            decision_buffer: Vec::new(),
            lineage_consumption: Vec::new(),
            entity_snapshots: Vec::new(),
            food_dirty: true,
            spatial_data_buffer: Vec::new(),
            food_positions_buffer: Vec::new(),
        })
    }

    pub fn load_persistent(&mut self) -> anyhow::Result<()> {
        self.lineage_registry =
            LineageRegistry::load(format!("{}/lineages.json", self.log_dir)).unwrap_or_default();
        self.fossil_registry =
            FossilRegistry::load(&format!("{}/fossils.json.gz", self.log_dir)).unwrap_or_default();
        Ok(())
    }

    pub fn apply_genetic_edit(&mut self, entity_id: uuid::Uuid, gene: GeneType, delta: f32) {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity_id) {
            match gene {
                GeneType::Trophic => {
                    e.intel.genotype.trophic_potential =
                        (e.intel.genotype.trophic_potential + delta).clamp(0.0, 1.0);
                    e.metabolism.trophic_potential = e.intel.genotype.trophic_potential;
                }
                GeneType::Sensing => {
                    e.intel.genotype.sensing_range =
                        (e.intel.genotype.sensing_range + delta as f64).clamp(3.0, 30.0);
                    e.physics.sensing_range = e.intel.genotype.sensing_range;
                }
                GeneType::Speed => {
                    e.intel.genotype.max_speed =
                        (e.intel.genotype.max_speed + delta as f64).clamp(0.1, 5.0);
                    e.physics.max_speed = e.intel.genotype.max_speed;
                }
                GeneType::ReproInvest => {
                    e.intel.genotype.reproductive_investment =
                        (e.intel.genotype.reproductive_investment + delta).clamp(0.1, 0.9);
                }
                GeneType::Maturity => {
                    e.intel.genotype.maturity_gene =
                        (e.intel.genotype.maturity_gene + delta).clamp(0.1, 5.0);
                }
                GeneType::MaxEnergy => {
                    e.intel.genotype.max_energy =
                        (e.intel.genotype.max_energy + delta as f64).clamp(50.0, 2000.0);
                    e.metabolism.max_energy = e.intel.genotype.max_energy;
                }
            }
        }
    }

    pub fn apply_trade(
        &mut self,
        env: &mut Environment,
        resource: crate::model::infra::network::TradeResource,
        amount: f32,
        incoming: bool,
    ) {
        use crate::model::infra::network::TradeResource;
        let sign = if incoming { 1.0 } else { -1.0 };
        match resource {
            TradeResource::Energy => {
                let count = (self.entities.len() / 10).max(1);
                let amount_per = (amount * sign) / count as f32;
                for e in self.entities.iter_mut().take(count) {
                    e.metabolism.energy = (e.metabolism.energy + amount_per as f64)
                        .clamp(0.0, e.metabolism.max_energy);
                }
            }
            TradeResource::Oxygen => {
                env.oxygen_level = (env.oxygen_level + (amount * sign) as f64).clamp(0.0, 50.0);
            }
            TradeResource::SoilFertility => {
                self.terrain.add_global_fertility(amount * sign);
            }
            TradeResource::Biomass => {
                if incoming {
                    let mut rng = rand::thread_rng();
                    for _ in 0..(amount as usize) {
                        let fx = rng.gen_range(1..self.width - 1);
                        let fy = rng.gen_range(1..self.height - 1);
                        self.food.push(Food::new(fx, fy, 0.0));
                    }
                } else {
                    self.food
                        .truncate(self.food.len().saturating_sub(amount as usize));
                }
                self.food_dirty = true;
            }
        }
    }

    pub fn apply_relief(&mut self, lineage_id: uuid::Uuid, amount: f32) {
        let members: Vec<_> = self
            .entities
            .iter_mut()
            .filter(|e| e.metabolism.lineage_id == lineage_id)
            .collect();

        if !members.is_empty() {
            let amount_per = amount as f64 / members.len() as f64;
            for e in members {
                e.metabolism.energy =
                    (e.metabolism.energy + amount_per).min(e.metabolism.max_energy);
            }
        }
    }

    pub fn clear_research_deltas(&mut self, entity_id: uuid::Uuid) {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity_id) {
            e.intel.genotype.brain.weight_deltas.clear();
        }
    }

    pub fn create_snapshot(
        &self,
        selected_id: Option<uuid::Uuid>,
    ) -> crate::model::snapshot::WorldSnapshot {
        use crate::model::snapshot::{EntitySnapshot, WorldSnapshot};
        let entities = self
            .entities
            .iter()
            .map(|e| EntitySnapshot {
                id: e.id,
                name: e.name(),
                x: e.physics.x,
                y: e.physics.y,
                r: e.physics.r,
                g: e.physics.g,
                b: e.physics.b,
                energy: e.metabolism.energy,
                max_energy: e.metabolism.max_energy,
                generation: e.metabolism.generation,
                age: self.tick - e.metabolism.birth_tick,
                offspring: e.metabolism.offspring_count,
                lineage_id: e.metabolism.lineage_id,
                rank: e.intel.rank,
                status: e.status(
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
                specialization: e.intel.specialization,
                last_vocalization: e.intel.last_vocalization,
                bonded_to: e.intel.bonded_to,
                trophic_potential: e.metabolism.trophic_potential,
                last_activations: e
                    .intel
                    .last_activations
                    .0
                    .iter()
                    .enumerate()
                    .filter(|(_, &v)| v.abs() > 0.001)
                    .map(|(k, v)| (k as i32, *v))
                    .collect(),
                last_inputs: e.intel.last_inputs,
                last_hidden: e.intel.last_hidden,
                weight_deltas: if Some(e.id) == selected_id {
                    e.intel.genotype.brain.weight_deltas.clone()
                } else {
                    HashMap::new()
                },
                genotype_hex: if Some(e.id) == selected_id {
                    Some(e.intel.genotype.to_hex())
                } else {
                    None
                },
            })
            .collect();

        WorldSnapshot {
            tick: self.tick,
            entities,
            food: self.food.clone(),
            stats: self.pop_stats.clone(),
            hall_of_fame: self.hall_of_fame.clone(),
            terrain: self.cached_terrain.clone(),
            pheromones: self.cached_pheromones.clone(),
            sound: self.cached_sound.clone(),
            pressure: self.cached_pressure.clone(),
            social_grid: self.cached_social_grid.clone(),
            width: self.width,
            height: self.height,
        }
    }

    pub fn update(&mut self, env: &mut Environment) -> anyhow::Result<Vec<LiveEvent>> {
        let mut events = Vec::new();
        self.tick += 1;

        let world_seed = self.config.world.seed.unwrap_or(0);

        self.update_environment_and_resources(env, world_seed);

        let tick = self.tick;
        let config = &self.config;
        self.entities.par_iter_mut().for_each(|e| {
            e.intel.rank = social::calculate_social_rank(e, tick, config);
        });

        let entity_id_map = self.prepare_spatial_hash();

        self.capture_entity_snapshots();

        self.learn_and_bond_check_parallel(&entity_id_map);

        let interaction_commands = self.perceive_and_decide(env, &entity_id_map, world_seed);

        let snapshots = std::mem::take(&mut self.entity_snapshots);
        self.execute_actions(env, &snapshots, &entity_id_map);
        self.entity_snapshots = snapshots;

        let mut interaction_events = self.execute_interactions(env, interaction_commands);
        events.append(&mut interaction_events);

        self.finalize_tick(env, &mut events);

        Ok(events)
    }

    fn update_environment_and_resources(&mut self, env: &mut Environment, world_seed: u64) {
        action::handle_game_modes(
            &mut self.entities,
            &self.config,
            self.tick,
            self.width,
            self.height,
        );

        if self.tick.is_multiple_of(50) {
            for val in &mut self.social_grid {
                *val = 0;
            }
        }

        self.pheromones.update();
        self.sound.update();
        self.pressure.update();
        self.lineage_registry.decay_memory(0.99);

        environment::handle_disasters(
            env,
            self.entities.len(),
            &mut self.terrain,
            &mut self.rng,
            &self.config,
        );

        let (_total_plant_biomass, total_sequestration) =
            self.terrain
                .update(self.pop_stats.biomass_h, self.tick, world_seed);

        // Phase 63: Ecosystem Dominance (Global Albedo Cooling)
        let total_owned_forests = self
            .terrain
            .cells
            .iter()
            .filter(|c| {
                c.terrain_type == crate::model::terrain::TerrainType::Forest && c.owner_id.is_some()
            })
            .count();
        if total_owned_forests > 100 {
            env.carbon_level = (env.carbon_level - 0.5).max(100.0);
        }

        env.sequestrate_carbon(total_sequestration * self.config.ecosystem.sequestration_rate);
        env.add_carbon(self.entities.len() as f64 * self.config.ecosystem.carbon_emission_rate);
        env.consume_oxygen(
            self.entities.len() as f64 * self.config.metabolism.oxygen_consumption_rate,
        );
        env.tick();

        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut self.rng);

        let old_food_len = self.food.len();
        ecological::spawn_food(
            &mut self.food,
            env,
            &self.terrain,
            &self.config,
            self.width,
            self.height,
            &mut self.rng,
        );

        if self.food.len() != old_food_len {
            self.food_dirty = true;
        }

        if self.food_dirty {
            let mut food_positions = std::mem::take(&mut self.food_positions_buffer);
            food_positions.clear();
            food_positions.extend(self.food.iter().map(|f| (f.x as f64, f.y as f64)));
            self.food_hash
                .build_parallel(&food_positions, self.width, self.height);
            self.sync_ecs_from_food();
            self.food_dirty = false;
            self.food_positions_buffer = food_positions;
        }
    }

    fn sync_ecs_from_food(&mut self) {
        self.ecs.clear();
        for f in &self.food {
            self.ecs.spawn((
                Position {
                    x: f.x as f64,
                    y: f.y as f64,
                },
                MetabolicNiche(f.nutrient_type),
            ));
        }
    }

    fn prepare_spatial_hash(&mut self) -> HashMap<uuid::Uuid, usize> {
        let mut spatial_data = std::mem::take(&mut self.spatial_data_buffer);
        spatial_data.clear();
        spatial_data.extend(
            self.entities
                .iter()
                .map(|e| (e.physics.x, e.physics.y, e.metabolism.lineage_id)),
        );

        self.spatial_hash
            .build_with_lineage(&spatial_data, self.width, self.height);

        let entity_id_map: HashMap<uuid::Uuid, usize> = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id, i))
            .collect();

        self.spatial_data_buffer = spatial_data;
        entity_id_map
    }

    fn capture_entity_snapshots(&mut self) {
        let mut entity_snapshots = std::mem::take(&mut self.entity_snapshots);
        self.entities
            .par_iter()
            .map(|e| InternalEntitySnapshot {
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
                rank: e.intel.rank,
                status: e.status(
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
            })
            .collect_into_vec(&mut entity_snapshots);

        self.entity_snapshots = entity_snapshots;
    }

    fn learn_and_bond_check_parallel(&mut self, entity_id_map: &HashMap<uuid::Uuid, usize>) {
        let config = &self.config;
        let snapshots = &self.entity_snapshots;

        self.entities.par_iter_mut().for_each(|e| {
            e.intel.genotype.brain.learn(
                e.intel.last_inputs,
                e.intel.last_hidden,
                ((e.metabolism.energy - e.metabolism.prev_energy)
                    / e.metabolism.max_energy.max(1.0)) as f32
                    * config.brain.learning_reinforcement,
            );
            e.metabolism.prev_energy = e.metabolism.energy;

            if let Some(p_id) = e.intel.bonded_to {
                if let Some(partner) = entity_id_map.get(&p_id).map(|&idx| &snapshots[idx]) {
                    let dx = partner.x - e.physics.x;
                    let dy = partner.y - e.physics.y;
                    if (dx * dx + dy * dy) > config.social.bond_break_dist.powi(2) {
                        e.intel.bonded_to = None;
                    }
                } else {
                    e.intel.bonded_to = None;
                }
            }
        });
    }

    fn perceive_and_decide(
        &mut self,
        env: &Environment,
        entity_id_map: &HashMap<uuid::Uuid, usize>,
        world_seed: u64,
    ) -> Vec<InteractionCommand> {
        let mut perception_buffer = std::mem::take(&mut self.perception_buffer);
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);
        let snapshots = &self.entity_snapshots;
        let food = &self.food;
        let food_hash = &self.food_hash;
        let spatial_hash = &self.spatial_hash;
        let pheromones = &self.pheromones;
        let sound = &self.sound;
        let pressure = &self.pressure;
        let terrain = &self.terrain;
        let tick = self.tick;

        self.entities
            .par_iter()
            .map(|e| {
                let (dx_f, dy_f, f_type) = ecological::sense_nearest_food(e, food, food_hash);
                let nearby_count =
                    spatial_hash.count_nearby(e.physics.x, e.physics.y, e.physics.sensing_range);
                let (ph_f, tribe_d, sa, sb) =
                    pheromones.sense_all(e.physics.x, e.physics.y, e.physics.sensing_range / 2.0);
                let (kx, ky) = spatial_hash.sense_kin(
                    e.physics.x,
                    e.physics.y,
                    e.physics.sensing_range,
                    e.metabolism.lineage_id,
                );
                let wall_dist = terrain.sense_wall(e.physics.x, e.physics.y, 5.0);
                let age_ratio = (tick - e.metabolism.birth_tick) as f32 / 2000.0;
                let sound_sense = sound.sense(e.physics.x, e.physics.y, e.physics.sensing_range);
                let mut partner_energy = 0.0;
                if let Some(p_id) = e.intel.bonded_to {
                    if let Some(&p_idx) = entity_id_map.get(&p_id) {
                        partner_energy =
                            (snapshots[p_idx].energy / e.metabolism.max_energy.max(1.0)) as f32;
                    }
                }
                let (b_press, d_press) =
                    pressure.sense(e.physics.x, e.physics.y, e.physics.sensing_range);
                let shared_goal = self
                    .lineage_registry
                    .get_memory_value(&e.metabolism.lineage_id, "goal");
                let shared_threat = self
                    .lineage_registry
                    .get_memory_value(&e.metabolism.lineage_id, "threat");
                let mut lin_pop = 0.0;
                let mut lin_energy = 0.0;
                let mut overmind_signal = 0.0;
                if let Some(record) = self.lineage_registry.lineages.get(&e.metabolism.lineage_id) {
                    lin_pop = (record.current_population as f32 / 100.0).min(1.0);
                    lin_energy = (record.total_energy_consumed as f32 / 10000.0).min(1.0);
                    overmind_signal = self
                        .lineage_registry
                        .get_memory_value(&e.metabolism.lineage_id, "overmind");
                }

                [
                    (dx_f / 20.0) as f32,
                    (dy_f / 20.0) as f32,
                    (e.metabolism.energy / e.metabolism.max_energy.max(1.0)) as f32,
                    (nearby_count as f32 / 10.0).min(1.0),
                    ph_f,
                    tribe_d,
                    kx as f32,
                    ky as f32,
                    sa,
                    sb,
                    wall_dist,
                    age_ratio.min(1.0),
                    f_type,
                    e.metabolism.trophic_potential,
                    e.intel.last_hidden[0],
                    e.intel.last_hidden[1],
                    e.intel.last_hidden[2],
                    e.intel.last_hidden[3],
                    e.intel.last_hidden[4],
                    e.intel.last_hidden[5],
                    sound_sense,
                    partner_energy,
                    b_press,
                    d_press,
                    shared_goal,
                    shared_threat,
                    lin_pop,
                    lin_energy,
                    overmind_signal,
                ]
            })
            .collect_into_vec(&mut perception_buffer);

        self.entities
            .par_iter()
            .zip(perception_buffer.par_iter())
            .map(|(e, inputs)| {
                let (mut outputs, next_hidden, activations) = e
                    .intel
                    .genotype
                    .brain
                    .forward_internal(*inputs, e.intel.last_hidden);
                if let Some(ref path) = e.health.pathogen {
                    if let Some((idx, offset)) = path.behavior_manipulation {
                        let out_idx = idx.saturating_sub(22);
                        if out_idx < 11 {
                            outputs[out_idx] = (outputs[out_idx] + offset).clamp(-1.0, 1.0);
                        }
                    }
                }
                EntityDecision {
                    outputs,
                    next_hidden,
                    activations,
                }
            })
            .collect_into_vec(&mut decision_buffer);

        let config = &self.config;
        let lineage_registry = &self.lineage_registry;
        let entities = &self.entities;

        let interaction_commands: Vec<InteractionCommand> = entities
            .par_iter()
            .enumerate()
            .flat_map(|(i, e)| {
                let mut cmds = Vec::new();
                let entity_seed = world_seed ^ tick ^ (e.id.as_u128() as u64);
                let mut local_rng = ChaCha8Rng::seed_from_u64(entity_seed);
                let decision = &decision_buffer[i];
                let outputs = decision.outputs;

                // Automatic feeding check
                let (dx_f, dy_f, _) = ecological::sense_nearest_food(e, food, food_hash);
                if dx_f.abs() < 1.5 && dy_f.abs() < 1.5 {
                    food_hash.query_callback(e.physics.x, e.physics.y, 1.5, |f_idx| {
                        cmds.push(InteractionCommand::EatFood {
                            food_index: f_idx,
                            attacker_idx: i,
                        });
                    });
                }

                if e.intel.bonded_to.is_none() && e.metabolism.has_metamorphosed {
                    if let Some(p_id) =
                        social::handle_symbiosis(i, entities, outputs, spatial_hash, config)
                    {
                        cmds.push(InteractionCommand::Bond {
                            target_idx: i,
                            partner_id: p_id,
                        });
                    }
                }

                if let Some(p_id) = e.intel.bonded_to {
                    if outputs[8] < 0.2 {
                        cmds.push(InteractionCommand::BondBreak { target_idx: i });
                    } else if let Some(&p_idx) = entity_id_map.get(&p_id) {
                        let partner = &entities[p_idx];
                        if e.is_mature(tick, config.metabolism.maturity_age)
                            && partner.is_mature(tick, config.metabolism.maturity_age)
                            && e.metabolism.energy > config.metabolism.reproduction_threshold
                            && partner.metabolism.energy > config.metabolism.reproduction_threshold
                        {
                            let ancestral = lineage_registry
                                .lineages
                                .get(&e.metabolism.lineage_id)
                                .and_then(|r| r.max_fitness_genotype.as_ref());
                            let mut repro_ctx = ReproductionContext {
                                tick,
                                config,
                                population: entities.len(),
                                traits: lineage_registry.get_traits(&e.metabolism.lineage_id),
                                is_radiation_storm: env.is_radiation_storm(),
                                rng: &mut local_rng,
                                ancestral_genotype: ancestral,
                            };
                            let (baby, dist) =
                                social::reproduce_sexual_parallel(e, partner, &mut repro_ctx);
                            cmds.push(InteractionCommand::Birth {
                                parent_idx: i,
                                baby: Box::new(baby),
                                genetic_distance: dist,
                            });
                            cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: p_idx,
                                amount: -(partner.metabolism.energy
                                    * partner.intel.genotype.reproductive_investment as f64),
                            });
                        }
                        let self_energy = e.metabolism.energy;
                        let partner_energy = snapshots[p_idx].energy;
                        if self_energy > partner_energy + 2.0 {
                            let diff = self_energy - partner_energy;
                            let amount = diff * 0.05;
                            if amount > 0.1 {
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: p_idx,
                                    amount,
                                });
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: -amount,
                                });
                            }
                        }
                    }
                }

                if outputs[3] > 0.5 {
                    spatial_hash.query_callback(e.physics.x, e.physics.y, 1.5, |t_idx| {
                        if i != t_idx && !social::are_same_tribe(e, &entities[t_idx], config) {
                            cmds.push(InteractionCommand::Kill {
                                target_idx: t_idx,
                                attacker_idx: i,
                                attacker_lineage: e.metabolism.lineage_id,
                                cause: "predation".to_string(),
                            });
                        }
                    });
                }

                if outputs[4] > 0.5 && e.metabolism.energy > e.metabolism.max_energy * 0.7 {
                    spatial_hash.query_callback(e.physics.x, e.physics.y, 3.0, |t_idx| {
                        if i != t_idx && social::are_same_tribe(e, &entities[t_idx], config) {
                            let partner = &entities[t_idx];
                            if partner.metabolism.energy < partner.metabolism.max_energy * 0.5 {
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: t_idx,
                                    amount: e.metabolism.energy * 0.1,
                                });
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: -e.metabolism.energy * 0.1,
                                });
                            }
                        }
                    });
                }

                if e.is_mature(tick, config.metabolism.maturity_age)
                    && e.metabolism.energy > config.metabolism.reproduction_threshold
                {
                    let mut repro_ctx = ReproductionContext {
                        tick,
                        config,
                        population: entities.len(),
                        traits: lineage_registry.get_traits(&e.metabolism.lineage_id),
                        is_radiation_storm: env.is_radiation_storm(),
                        rng: &mut local_rng,
                        ancestral_genotype: lineage_registry
                            .lineages
                            .get(&e.metabolism.lineage_id)
                            .and_then(|r| r.max_fitness_genotype.as_ref()),
                    };
                    let (baby, dist) = social::reproduce_asexual_parallel(e, &mut repro_ctx);
                    cmds.push(InteractionCommand::Birth {
                        parent_idx: i,
                        baby: Box::new(baby),
                        genetic_distance: dist,
                    });
                }

                if (outputs[9] > 0.5 || outputs[10] > 0.5) && e.metabolism.has_metamorphosed {
                    if outputs[9] > outputs[10] {
                        cmds.push(InteractionCommand::Dig {
                            x: e.physics.x,
                            y: e.physics.y,
                            attacker_idx: i,
                        });
                    } else {
                        cmds.push(InteractionCommand::Build {
                            x: e.physics.x,
                            y: e.physics.y,
                            attacker_idx: i,
                            is_nest: true,
                            is_outpost: outputs[10] > 0.8,
                        });
                    }
                }

                if let Some(new_color) = social::start_tribal_split(
                    e,
                    spatial_hash.count_nearby(e.physics.x, e.physics.y, e.physics.sensing_range)
                        as f32
                        / 10.0,
                    config,
                    &mut local_rng,
                ) {
                    cmds.push(InteractionCommand::TribalSplit {
                        target_idx: i,
                        new_color,
                    });
                }

                if !e.metabolism.has_metamorphosed
                    && (tick - e.metabolism.birth_tick)
                        > (config.metabolism.maturity_age as f32
                            * e.intel.genotype.maturity_gene
                            * config.metabolism.metamorphosis_trigger_maturity)
                            as u64
                {
                    cmds.push(InteractionCommand::Metamorphosis { target_idx: i });
                }

                cmds
            })
            .collect();

        self.perception_buffer = perception_buffer;
        self.decision_buffer = decision_buffer;

        interaction_commands
    }

    fn execute_actions(
        &mut self,
        env: &mut Environment,
        entity_snapshots: &[InternalEntitySnapshot],
        entity_id_map: &HashMap<uuid::Uuid, usize>,
    ) {
        let config = &self.config;
        let terrain = &self.terrain;
        let spatial_hash = &self.spatial_hash;
        let pressure_grid = &self.pressure;
        let width = self.width;
        let height = self.height;

        let action_outputs: Vec<action::ActionOutput> = self
            .entities
            .par_iter_mut()
            .zip(self.decision_buffer.par_iter_mut())
            .map(|(e, decision)| {
                let EntityDecision {
                    outputs,
                    next_hidden,
                    activations,
                } = std::mem::take(decision);
                e.intel.last_hidden = next_hidden;
                e.intel.last_activations = activations;
                e.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

                let mut output = action::ActionOutput::default();
                action::action_system(
                    e,
                    outputs,
                    &mut action::ActionContext {
                        env,
                        config,
                        terrain,
                        snapshots: entity_snapshots,
                        entity_id_map,
                        spatial_hash,
                        pressure: pressure_grid,
                        width,
                        height,
                    },
                    &mut output,
                );
                output
            })
            .collect();

        let mut overmind_broadcasts = Vec::new();
        for res in action_outputs {
            for p in res.pheromones {
                self.pheromones.deposit(p.x, p.y, p.ptype, p.amount);
            }
            for s in res.sounds {
                self.sound.deposit(s.x, s.y, s.amount);
            }
            for pr in res.pressure {
                self.pressure.deposit(pr.x, pr.y, pr.ptype, pr.amount);
            }
            if let Some(b) = res.overmind_broadcast {
                overmind_broadcasts.push(b);
            }
            env.consume_oxygen(res.oxygen_drain);
        }

        for (l_id, amount) in overmind_broadcasts {
            self.lineage_registry
                .set_memory_value(&l_id, "overmind", amount);
        }
    }

    fn execute_interactions(
        &mut self,
        env: &mut Environment,
        interaction_commands: Vec<InteractionCommand>,
    ) -> Vec<LiveEvent> {
        let mut interaction_ctx = interaction::InteractionContext {
            terrain: &mut self.terrain,
            env,
            pop_stats: &mut self.pop_stats,
            lineage_registry: &mut self.lineage_registry,
            fossil_registry: &mut self.fossil_registry,
            logger: &mut self.logger,
            config: &self.config,
            tick: self.tick,
            width: self.width,
            height: self.height,
            social_grid: &mut self.social_grid,
            lineage_consumption: &mut self.lineage_consumption,
            food: &mut self.food,
            spatial_hash: &self.spatial_hash,
            rng: &mut self.rng,
        };

        let interaction_result = interaction::process_interaction_commands(
            &mut self.entities,
            interaction_commands,
            &mut interaction_ctx,
        );

        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();

        self.killed_ids = interaction_result.killed_ids;
        self.eaten_food_indices = interaction_result.eaten_food_indices;
        self.new_babies = interaction_result.new_babies;

        interaction_result.events
    }

    fn finalize_tick(&mut self, env: &mut Environment, events: &mut Vec<LiveEvent>) {
        let config = &self.config;

        let population_count = self.entities.len();
        let mut rng_vec: Vec<_> = (0..population_count)
            .map(|i| ChaCha8Rng::seed_from_u64(self.tick ^ i as u64))
            .collect();

        self.entities
            .par_iter_mut()
            .zip(rng_vec.par_iter_mut())
            .for_each(|(e, rng)| {
                biological::biological_system(e, population_count, config, rng);
            });

        // Sequential infection spread
        for i in 0..self.entities.len() {
            biological::handle_infection(
                i,
                &mut self.entities,
                &self.killed_ids,
                &self.active_pathogens,
                &self.spatial_hash,
                &mut self.rng,
            );
        }

        let mut alive_entities = std::mem::take(&mut self.alive_entities);
        let tick = self.tick;

        let entities = std::mem::take(&mut self.entities);
        for e in entities {
            if self.killed_ids.contains(&e.id) || e.metabolism.energy <= 0.0 {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
                if let Some(legend) = social::archive_if_legend(&e, tick, &self.logger) {
                    history::update_best_legend(
                        &mut self.lineage_registry,
                        &mut self.best_legends,
                        legend,
                    );
                }
                let fertilize_amount = (e.metabolism.max_energy
                    * self.config.ecosystem.corpse_fertility_mult as f64)
                    as f32
                    / 100.0;
                self.terrain
                    .fertilize(e.physics.x, e.physics.y, fertilize_amount);
                self.terrain
                    .add_biomass(e.physics.x, e.physics.y, fertilize_amount * 10.0);
            } else {
                alive_entities.push(e);
            }
        }

        self.entities.append(&mut alive_entities);
        self.entities.append(&mut self.new_babies);
        self.alive_entities = alive_entities;

        if !self.eaten_food_indices.is_empty() {
            let mut i = 0;
            let eaten = &self.eaten_food_indices;
            self.food.retain(|_| {
                let k = !eaten.contains(&i);
                i += 1;
                k
            });
            self.food_dirty = true;
        }

        if self.tick.is_multiple_of(self.config.world.fossil_interval) {
            let outpost_counts = civilization::count_outposts_by_lineage(&self.terrain);
            self.lineage_registry.check_goals(
                self.tick,
                &self.social_grid,
                self.width,
                self.height,
                &outpost_counts,
            );
            let _ = self
                .lineage_registry
                .save(format!("{}/lineages.json", self.log_dir));
            let _ = self
                .fossil_registry
                .save(&format!("{}/fossils.json.gz", self.log_dir));
            let snap_ev = LiveEvent::Snapshot {
                tick: self.tick,
                stats: self.pop_stats.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(snap_ev.clone());
            events.push(snap_ev);
            history::handle_fossilization(
                &self.lineage_registry,
                &mut self.fossil_registry,
                &mut self.best_legends,
                self.tick,
            );
            self.lineage_registry.prune();
        }

        civilization::handle_outposts(
            &mut self.terrain,
            &mut self.entities,
            &self.spatial_hash,
            self.width,
            self.config.social.silo_energy_capacity,
            self.config.social.outpost_energy_capacity,
        );
        if self
            .tick
            .is_multiple_of(self.config.world.power_grid_interval)
        {
            civilization::resolve_power_grid(&mut self.terrain, self.width, self.height);
        }

        stats::update_stats(
            self.tick,
            &self.entities,
            self.food.len(),
            env.carbon_level,
            self.config.evolution.mutation_rate,
            &mut self.pop_stats,
            &mut self.hall_of_fame,
            &self.terrain,
        );

        self.observer
            .observe(self.tick, &self.pop_stats, &self.lineage_registry, env);

        for n in self.observer.consume_narrations() {
            events.push(LiveEvent::Narration {
                tick: n.tick,
                text: n.text,
                severity: n.severity,
                timestamp: Utc::now().to_rfc3339(),
            });
        }

        if self.terrain.is_dirty {
            self.cached_terrain = std::sync::Arc::new(self.terrain.clone());
            self.terrain.is_dirty = false;
        }
        if self.pheromones.is_dirty {
            self.cached_pheromones = std::sync::Arc::new(self.pheromones.clone());
            self.pheromones.is_dirty = false;
        }
        if self.sound.is_dirty {
            self.cached_sound = std::sync::Arc::new(self.sound.clone());
            self.sound.is_dirty = false;
        }
        if self.pressure.is_dirty {
            self.cached_pressure = std::sync::Arc::new(self.pressure.clone());
            self.pressure.is_dirty = false;
        }
        self.cached_social_grid = std::sync::Arc::new(self.social_grid.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handle_movement_wall_bounce() {
        let mut config = AppConfig::default();
        config.world.width = 10;
        config.world.height = 10;
        let mut world = World::new(0, config).unwrap();
        world
            .terrain
            .set_cell_type(5, 5, crate::model::terrain::TerrainType::Wall);
        let mut entity = crate::model::lifecycle::create_entity(4.5, 4.5, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 1.0;
        action::handle_movement(&mut entity, 1.0, &world.terrain, world.width, world.height);
        assert!(entity.physics.vx < 0.0);
    }
}
