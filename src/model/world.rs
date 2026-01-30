use crate::model::config::AppConfig;
use crate::model::environment::Environment;
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
use primordium_data::{Entity, Food};
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
use primordium_data::{GeneType, Health, Intel, MetabolicNiche, Metabolism, Physics, Position};
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

#[derive(Default)]
pub struct EntityDecision {
    pub outputs: [f32; 12],
    pub next_hidden: [f32; 6],
    pub activations: primordium_data::Activations,
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub width: u16,
    pub height: u16,
    pub tick: u64,
    #[serde(skip, default = "hecs::World::new")]
    pub ecs: hecs::World,

    pub entities_persist: Vec<primordium_data::Entity>,
    pub food_persist: Vec<primordium_data::Food>,

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
    pub killed_ids: HashSet<uuid::Uuid>,
    #[serde(skip, default)]
    pub eaten_food_indices: HashSet<usize>,

    #[serde(skip, default)]
    pub decision_buffer: Vec<EntityDecision>,
    #[serde(skip, default)]
    pub lineage_consumption: Vec<(uuid::Uuid, f64)>,
    #[serde(skip, default)]
    pub entity_snapshots: Vec<InternalEntitySnapshot>,

    #[serde(skip)]
    pub cached_terrain: std::sync::Arc<TerrainGrid>,
    #[serde(skip)]
    pub cached_pheromones: std::sync::Arc<PheromoneGrid>,
    #[serde(skip)]
    pub cached_sound: std::sync::Arc<SoundGrid>,
    #[serde(skip)]
    pub cached_pressure: std::sync::Arc<crate::model::pressure::PressureGrid>,
    #[serde(skip)]
    pub cached_social_grid: std::sync::Arc<Vec<u8>>,
    #[serde(skip)]
    pub cached_rank_grid: std::sync::Arc<Vec<f32>>,
    pub food_dirty: bool,
    #[serde(skip, default)]
    pub spatial_data_buffer: Vec<(f64, f64, uuid::Uuid)>,
    #[serde(skip, default)]
    pub food_positions_buffer: Vec<(f64, f64)>,
}

impl World {
    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        let log_dir = "logs".to_string();
        Self::new_at(initial_population, config, &log_dir)
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
        let logger = HistoryLogger::new_at(log_dir)?;
        let mut lineage_registry = LineageRegistry::new();
        let mut ecs = hecs::World::new();
        for _ in 0..initial_population {
            let e = lifecycle::create_entity_with_rng(
                rng.gen_range(1.0..config.world.width as f64 - 1.0),
                rng.gen_range(1.0..config.world.height as f64 - 1.0),
                0,
                &mut rng,
            );
            lineage_registry.record_birth(e.metabolism.lineage_id, 1, 0);
            ecs.spawn((
                e.identity,
                e.position,
                e.velocity,
                e.appearance,
                e.physics,
                e.metabolism,
                e.health,
                e.intel,
            ));
        }
        for _ in 0..config.world.initial_food {
            let fx = rng.gen_range(1..config.world.width - 1);
            let fy = rng.gen_range(1..config.world.height - 1);
            let n_type = rng.gen_range(0.0..1.0);
            ecs.spawn((
                Position {
                    x: fx as f64,
                    y: fy as f64,
                },
                MetabolicNiche(n_type),
                Food::new(fx, fy, n_type),
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
            tick: 0,
            ecs,
            entities_persist: Vec::new(),
            food_persist: Vec::new(),
            logger,
            spatial_hash: SpatialHash::new(5.0, config.world.width, config.world.height),
            food_hash: SpatialHash::new(5.0, config.world.width, config.world.height),
            pop_stats: PopulationStats::default(),
            hall_of_fame: HallOfFame::default(),
            cached_terrain: std::sync::Arc::new(terrain.clone()),
            cached_pheromones: std::sync::Arc::new(pheromones.clone()),
            cached_sound: std::sync::Arc::new(sound.clone()),
            cached_pressure: std::sync::Arc::new(pressure.clone()),
            cached_social_grid: std::sync::Arc::new(social_grid.clone()),
            cached_rank_grid: std::sync::Arc::new(vec![
                0.0;
                config.world.width as usize
                    * config.world.height as usize
            ]),
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

    pub fn prepare_for_save(&mut self) {
        self.entities_persist = self.get_all_entities();
        self.food_persist.clear();
        for (_handle, f) in self.ecs.query::<&Food>().iter() {
            self.food_persist.push(f.clone());
        }
        self.food_persist.sort_by_key(|f| (f.x, f.y));
    }

    pub fn post_load(&mut self) {
        self.ecs = hecs::World::new();
        for e in std::mem::take(&mut self.entities_persist) {
            self.ecs.spawn((
                e.identity,
                e.position,
                e.velocity,
                e.appearance,
                e.physics,
                e.metabolism,
                e.health,
                e.intel,
            ));
        }
        for f in std::mem::take(&mut self.food_persist) {
            self.ecs.spawn((
                Position {
                    x: f.x as f64,
                    y: f.y as f64,
                },
                MetabolicNiche(f.nutrient_type),
                f,
            ));
        }
        self.food_dirty = true;
    }

    pub fn spawn_entity(&mut self, e: Entity) -> hecs::Entity {
        self.ecs.spawn((
            e.identity,
            e.position,
            e.velocity,
            e.appearance,
            e.physics,
            e.metabolism,
            e.health,
            e.intel,
        ))
    }

    pub fn apply_genetic_edit(&mut self, entity_id: uuid::Uuid, gene: GeneType, delta: f32) {
        let mut query = self.ecs.query::<(
            &primordium_data::Identity,
            &mut primordium_data::Intel,
            &mut primordium_data::Metabolism,
            &mut primordium_data::Physics,
        )>();
        for (_handle, (identity, intel, met, phys)) in query.iter() {
            if identity.id == entity_id {
                match gene {
                    GeneType::Trophic => {
                        intel.genotype.trophic_potential =
                            (intel.genotype.trophic_potential + delta).clamp(0.0, 1.0);
                        met.trophic_potential = intel.genotype.trophic_potential;
                    }
                    GeneType::Sensing => {
                        intel.genotype.sensing_range =
                            (intel.genotype.sensing_range + delta as f64).clamp(3.0, 30.0);
                        phys.sensing_range = intel.genotype.sensing_range;
                    }
                    GeneType::Speed => {
                        intel.genotype.max_speed =
                            (intel.genotype.max_speed + delta as f64).clamp(0.1, 5.0);
                        phys.max_speed = intel.genotype.max_speed;
                    }
                    GeneType::ReproInvest => {
                        intel.genotype.reproductive_investment =
                            (intel.genotype.reproductive_investment + delta).clamp(0.1, 0.9);
                    }
                    GeneType::Maturity => {
                        intel.genotype.maturity_gene =
                            (intel.genotype.maturity_gene + delta).clamp(0.1, 5.0);
                    }
                    GeneType::MaxEnergy => {
                        intel.genotype.max_energy =
                            (intel.genotype.max_energy + delta as f64).clamp(50.0, 2000.0);
                        met.max_energy = intel.genotype.max_energy;
                    }
                }
                break;
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
                let query = self.ecs.query_mut::<&mut Metabolism>();
                let mut components: Vec<_> = query.into_iter().collect();
                let count = (components.len() / 10).max(1);
                let amount_per = (amount * sign) / count as f32;
                for (_handle, met) in components.iter_mut().take(count) {
                    met.energy = (met.energy + amount_per as f64).clamp(0.0, met.max_energy);
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
                        let n_type = rng.gen_range(0.0..1.0);
                        self.ecs.spawn((
                            Position {
                                x: fx as f64,
                                y: fy as f64,
                            },
                            MetabolicNiche(n_type),
                            Food::new(fx, fy, n_type),
                        ));
                    }
                } else {
                    let mut food_entities = Vec::new();
                    for (handle, _) in self.ecs.query::<&Food>().iter() {
                        food_entities.push(handle);
                    }
                    for &handle in food_entities.iter().take(amount as usize) {
                        let _ = self.ecs.despawn(handle);
                    }
                }
                self.food_dirty = true;
            }
        }
    }

    pub fn apply_relief(&mut self, lineage_id: uuid::Uuid, amount: f32) {
        let mut members = Vec::new();
        for (handle, met) in self.ecs.query::<&Metabolism>().iter() {
            if met.lineage_id == lineage_id {
                members.push(handle);
            }
        }

        if !members.is_empty() {
            let amount_per = amount as f64 / members.len() as f64;
            for handle in members {
                if let Ok(mut met) = self.ecs.get::<&mut Metabolism>(handle) {
                    met.energy = (met.energy + amount_per).min(met.max_energy);
                }
            }
        }
    }

    pub fn clear_research_deltas(&mut self, entity_id: uuid::Uuid) {
        for (_handle, (identity, intel)) in self
            .ecs
            .query_mut::<(&primordium_data::Identity, &mut primordium_data::Intel)>()
        {
            if identity.id == entity_id {
                intel.genotype.brain.weight_deltas.clear();
                break;
            }
        }
    }

    pub fn create_snapshot(
        &self,
        selected_id: Option<uuid::Uuid>,
    ) -> crate::model::snapshot::WorldSnapshot {
        use crate::model::snapshot::{EntitySnapshot, WorldSnapshot};
        let mut entities = Vec::new();

        for (_handle, (identity, physics, metabolism, intel, health)) in self
            .ecs
            .query::<(
                &primordium_data::Identity,
                &primordium_data::Physics,
                &primordium_data::Metabolism,
                &primordium_data::Intel,
                &primordium_data::Health,
            )>()
            .iter()
        {
            entities.push(EntitySnapshot {
                id: identity.id,
                name: identity.name.clone(),
                x: physics.x,
                y: physics.y,
                r: physics.r,
                g: physics.g,
                b: physics.b,
                energy: metabolism.energy,
                max_energy: metabolism.max_energy,
                generation: metabolism.generation,
                age: self.tick - metabolism.birth_tick,
                offspring: metabolism.offspring_count,
                lineage_id: metabolism.lineage_id,
                rank: intel.rank,
                status: lifecycle::calculate_status(
                    metabolism,
                    health,
                    intel,
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
                last_vocalization: intel.last_vocalization,
                bonded_to: intel.bonded_to,
                trophic_potential: metabolism.trophic_potential,
                last_activations: if Some(identity.id) == selected_id {
                    intel
                        .last_activations
                        .0
                        .iter()
                        .enumerate()
                        .filter(|(_, &v)| v.abs() > 0.001)
                        .map(|(k, v)| (k as i32, *v))
                        .collect()
                } else {
                    HashMap::new()
                },
                weight_deltas: if Some(identity.id) == selected_id {
                    intel.genotype.brain.weight_deltas.clone()
                } else {
                    HashMap::new()
                },
                genotype_hex: if Some(identity.id) == selected_id {
                    Some(intel.genotype.to_hex())
                } else {
                    None
                },
            });
        }
        entities.sort_by_key(|e| e.id);

        let mut food = Vec::new();
        for (_handle, f) in self.ecs.query::<&Food>().iter() {
            food.push(f.clone());
        }
        food.sort_by_key(|f| (f.x, f.y));

        WorldSnapshot {
            tick: self.tick,
            entities,
            food,
            stats: self.pop_stats.clone(),
            hall_of_fame: self.hall_of_fame.clone(),
            terrain: self.cached_terrain.clone(),
            pheromones: self.cached_pheromones.clone(),
            sound: self.cached_sound.clone(),
            pressure: self.cached_pressure.clone(),
            social_grid: self.cached_social_grid.clone(),
            rank_grid: self.cached_rank_grid.clone(),
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

        {
            let mut query = self.ecs.query::<(&Metabolism, &mut Intel)>();
            let mut components: Vec<_> = query.into_iter().collect();
            components
                .par_iter_mut()
                .for_each(|(_handle, (met, intel))| {
                    intel.rank = social::calculate_social_rank_components(met, intel, tick, config);
                });
        }

        let (id_map, handles, food_handles) = self.prepare_spatial_hash();

        self.capture_entity_snapshots();

        self.learn_and_bond_check_parallel(&id_map);

        let interaction_commands =
            self.perceive_and_decide(env, &id_map, &food_handles, world_seed);

        let snapshots = std::mem::take(&mut self.entity_snapshots);
        self.execute_actions(env, &snapshots, &id_map);
        self.entity_snapshots = snapshots;

        let interaction_res =
            self.execute_interactions(env, interaction_commands, &handles, &food_handles);
        let (mut interaction_events, new_babies) = interaction_res;
        events.append(&mut interaction_events);

        self.finalize_tick(env, &mut events, &handles, new_babies);

        self.pheromones.update();
        self.sound.update();
        self.pressure.update();

        if self.config.world.deterministic {
            env.tick_deterministic(self.tick);
        } else {
            env.tick();
        }

        Ok(events)
    }

    fn update_environment_and_resources(&mut self, env: &mut Environment, world_seed: u64) {
        action::handle_game_modes_ecs(
            &mut self.ecs,
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

        self.lineage_registry.decay_memory(0.99);

        let pop_count = self.get_population_count();
        environment::handle_disasters(
            env,
            pop_count,
            &mut self.terrain,
            &mut self.rng,
            &self.config,
        );

        let (_total_plant_biomass, total_sequestration) =
            self.terrain
                .update(self.pop_stats.biomass_h, self.tick, world_seed);

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
        env.add_carbon(pop_count as f64 * self.config.ecosystem.carbon_emission_rate);
        env.consume_oxygen(pop_count as f64 * self.config.metabolism.oxygen_consumption_rate);
        env.tick();

        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut self.rng);

        ecological::spawn_food_ecs(
            &mut self.ecs,
            env,
            &self.terrain,
            &self.config,
            self.width,
            self.height,
            &mut self.rng,
        );

        if self.food_dirty {
            let mut food_positions = std::mem::take(&mut self.food_positions_buffer);
            food_positions.clear();
            for (_handle, (pos, _)) in self
                .ecs
                .query::<(&Position, &crate::model::food::Food)>()
                .iter()
            {
                food_positions.push((pos.x, pos.y));
            }
            self.food_hash
                .build_parallel(&food_positions, self.width, self.height);
            self.food_dirty = false;
            self.food_positions_buffer = food_positions;
        }
    }

    pub fn prepare_spatial_hash(
        &mut self,
    ) -> (
        HashMap<uuid::Uuid, usize>,
        Vec<hecs::Entity>,
        Vec<hecs::Entity>,
    ) {
        let mut entity_data: Vec<_> = self
            .ecs
            .query::<(
                &primordium_data::Identity,
                &primordium_data::Position,
                &primordium_data::Metabolism,
            )>()
            .iter()
            .map(|(h, (i, p, m))| (i.id, h, p.x, p.y, m.lineage_id))
            .collect();
        entity_data.sort_by_key(|d| d.0);

        let mut spatial_data = std::mem::take(&mut self.spatial_data_buffer);
        spatial_data.clear();

        let mut entity_handles = Vec::new();
        let mut entity_id_map = HashMap::new();

        for (id, handle, x, y, lid) in entity_data {
            let idx = entity_handles.len();
            entity_id_map.insert(id, idx);
            entity_handles.push(handle);
            spatial_data.push((x, y, lid));
        }

        self.spatial_hash
            .build_with_lineage(&spatial_data, self.width, self.height);

        let mut food_handles = Vec::new();
        let mut food_positions = Vec::new();
        for (handle, (pos, _)) in self.ecs.query::<(&Position, &Food)>().iter() {
            food_handles.push(handle);
            food_positions.push((pos.x, pos.y));
        }

        self.food_hash
            .build_parallel(&food_positions, self.width, self.height);

        self.spatial_data_buffer = spatial_data; // Restore buffer
        (entity_id_map, entity_handles, food_handles)
    }

    pub fn capture_entity_snapshots(&mut self) {
        self.entity_snapshots.clear();
        for (_handle, (identity, physics, metabolism, intel, health)) in self
            .ecs
            .query::<(
                &primordium_data::Identity,
                &primordium_data::Physics,
                &primordium_data::Metabolism,
                &primordium_data::Intel,
                &primordium_data::Health,
            )>()
            .iter()
        {
            self.entity_snapshots.push(InternalEntitySnapshot {
                id: identity.id,
                lineage_id: metabolism.lineage_id,
                x: physics.x,
                y: physics.y,
                energy: metabolism.energy,
                birth_tick: metabolism.birth_tick,
                offspring_count: metabolism.offspring_count,
                r: physics.r,
                g: physics.g,
                b: physics.b,
                rank: intel.rank,
                status: lifecycle::calculate_status(
                    metabolism,
                    health,
                    intel,
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
            });
        }
        self.entity_snapshots.sort_by_key(|s| s.id);
    }

    fn learn_and_bond_check_parallel(&mut self, _id_map: &HashMap<uuid::Uuid, usize>) {
        let _tick = self.tick;
        let _config = &self.config;
        for (_handle, (intel, metabolism)) in self
            .ecs
            .query_mut::<(&mut primordium_data::Intel, &primordium_data::Metabolism)>()
        {
            let reinforcement = if metabolism.energy > metabolism.prev_energy {
                0.1
            } else {
                -0.05
            };
            intel
                .genotype
                .brain
                .learn(intel.last_inputs, intel.last_hidden, reinforcement as f32);
        }
    }

    fn perceive_and_decide(
        &mut self,
        env: &Environment,
        entity_id_map: &HashMap<uuid::Uuid, usize>,
        food_handles: &[hecs::Entity],
        world_seed: u64,
    ) -> Vec<InteractionCommand> {
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);
        let snapshots = &self.entity_snapshots;
        let ecs = &self.ecs;
        let food_hash = &self.food_hash;
        let spatial_hash = &self.spatial_hash;
        let pheromones = &self.pheromones;
        let sound = &self.sound;
        let pressure = &self.pressure;
        let terrain = &self.terrain;
        let tick = self.tick;
        let registry = &self.lineage_registry;

        let mut query = self.ecs.query::<(
            &primordium_data::Identity,
            &primordium_data::Position,
            &primordium_data::Velocity,
            &primordium_data::Physics,
            &primordium_data::Metabolism,
            &mut primordium_data::Intel,
            &primordium_data::Health,
        )>();
        let mut components: Vec<_> = query.iter().collect();
        components.sort_by_key(|(_h, (i, _p, _v, _ph, _m, _it, _hl))| i.id);

        decision_buffer.resize_with(components.len(), EntityDecision::default);

        components
            .par_iter_mut()
            .zip(decision_buffer.par_iter_mut())
            .for_each(
                |((_handle, (_identity, pos, _vel, phys, met, intel, health)), decision)| {
                    let (dx_f, dy_f, f_type) = ecological::sense_nearest_food_ecs_decomposed(
                        pos,
                        phys,
                        ecs,
                        food_hash,
                        food_handles,
                    );
                    let nearby_count = spatial_hash.count_nearby(pos.x, pos.y, phys.sensing_range);
                    let (ph_f, tribe_d, sa, sb) =
                        pheromones.sense_all(pos.x, pos.y, phys.sensing_range / 2.0);
                    let (kx, ky) =
                        spatial_hash.sense_kin(pos.x, pos.y, phys.sensing_range, met.lineage_id);
                    let wall_dist = terrain.sense_wall(pos.x, pos.y, 5.0);
                    let age_ratio = (tick - met.birth_tick) as f32 / 2000.0;
                    let sound_sense = sound.sense(pos.x, pos.y, phys.sensing_range);
                    let mut partner_energy = 0.0;
                    if let Some(p_id) = intel.bonded_to {
                        if let Some(&p_idx) = entity_id_map.get(&p_id) {
                            partner_energy =
                                (snapshots[p_idx].energy / met.max_energy.max(1.0)) as f32;
                        }
                    }
                    let (d_press, b_press) = pressure.sense(pos.x, pos.y, phys.sensing_range);
                    let shared_goal = registry.get_memory_value(&met.lineage_id, "goal");
                    let shared_threat = registry.get_memory_value(&met.lineage_id, "threat");
                    let mut lin_pop = 0.0;
                    let mut lin_energy = 0.0;
                    let mut overmind_signal = 0.0;
                    if let Some(record) = registry.lineages.get(&met.lineage_id) {
                        lin_pop = (record.current_population as f32 / 100.0).min(1.0);
                        lin_energy = (record.total_energy_consumed as f32 / 10000.0).min(1.0);
                        overmind_signal = registry.get_memory_value(&met.lineage_id, "overmind");
                    }

                    let inputs = [
                        (dx_f / 20.0) as f32,
                        (dy_f / 20.0) as f32,
                        (met.energy / met.max_energy.max(1.0)) as f32,
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
                        met.trophic_potential,
                        intel.last_hidden[0],
                        intel.last_hidden[1],
                        intel.last_hidden[2],
                        intel.last_hidden[3],
                        intel.last_hidden[4],
                        intel.last_hidden[5],
                        sound_sense,
                        partner_energy,
                        b_press,
                        d_press,
                        shared_goal,
                        shared_threat,
                        lin_pop,
                        lin_energy,
                        overmind_signal,
                    ];

                    let (mut outputs, next_hidden, activations) = intel
                        .genotype
                        .brain
                        .forward_internal(inputs, intel.last_hidden);
                    if let Some(ref path) = health.pathogen {
                        if let Some((idx, offset)) = path.behavior_manipulation {
                            let out_idx = idx.saturating_sub(22);
                            if out_idx < 11 {
                                outputs[out_idx] = (outputs[out_idx] + offset).clamp(-1.0, 1.0);
                            }
                        }
                    }
                    *decision = EntityDecision {
                        outputs,
                        next_hidden,
                        activations,
                    };
                },
            );

        let config = &self.config;

        let interaction_commands: Vec<InteractionCommand> = components
            .par_iter()
            .enumerate()
            .fold(
                Vec::new,
                |mut acc, (i, (_handle, (identity, pos, _vel, phys, met, intel, health)))| {
                    let entity_seed = world_seed ^ tick ^ (identity.id.as_u128() as u64);
                    let mut local_rng = ChaCha8Rng::seed_from_u64(entity_seed);
                    let decision = &decision_buffer[i];
                    let outputs = decision.outputs;

                    let (dx_f, dy_f, _) = ecological::sense_nearest_food_ecs_decomposed(
                        pos,
                        phys,
                        ecs,
                        food_hash,
                        food_handles,
                    );
                    if dx_f.abs() < 1.5 && dy_f.abs() < 1.5 {
                        food_hash.query_callback(pos.x, pos.y, 1.5, |f_idx| {
                            let food_handle = food_handles[f_idx];
                            let mut energy_gain = 0.0;
                            if let Ok(food_data) = ecs.get::<&Food>(food_handle) {
                                let trophic_eff = 1.0 - met.trophic_potential as f64;
                                if trophic_eff > 0.1 {
                                    let niche_eff = 1.0
                                        - (intel.genotype.metabolic_niche
                                            - food_data.nutrient_type)
                                            .abs();
                                    energy_gain = config.metabolism.food_value
                                        * niche_eff as f64
                                        * trophic_eff;
                                }
                            }
                            if energy_gain > 0.0 {
                                acc.push(InteractionCommand::EatFood {
                                    food_index: f_idx,
                                    attacker_idx: i,
                                    x: pos.x,
                                    y: pos.y,
                                    precalculated_energy_gain: energy_gain,
                                });
                            }
                        });
                    }

                    if intel.bonded_to.is_none() && met.has_metamorphosed {
                        if let Some(p_id) = social::handle_symbiosis_components(
                            i,
                            &components,
                            outputs,
                            spatial_hash,
                            config,
                        ) {
                            acc.push(InteractionCommand::Bond {
                                target_idx: i,
                                partner_id: p_id,
                            });
                        }
                    }

                    if let Some(p_id) = intel.bonded_to {
                        if outputs[8] < 0.2 {
                            acc.push(InteractionCommand::BondBreak { target_idx: i });
                        } else if let Some(&p_idx) = entity_id_map.get(&p_id) {
                            let partner_pos = components[p_idx].1 .1;
                            let partner_met = components[p_idx].1 .4;
                            let partner_intel = &components[p_idx].1 .5;
                            if lifecycle::is_mature_components(
                                met,
                                intel,
                                tick,
                                config.metabolism.maturity_age,
                            ) && lifecycle::is_mature_components(
                                partner_met,
                                partner_intel,
                                tick,
                                config.metabolism.maturity_age,
                            ) && met.energy > config.metabolism.reproduction_threshold
                                && partner_met.energy > config.metabolism.reproduction_threshold
                            {
                                let ancestral = registry
                                    .lineages
                                    .get(&met.lineage_id)
                                    .and_then(|r| r.max_fitness_genotype.as_ref());
                                let mut repro_ctx = ReproductionContext {
                                    tick,
                                    config,
                                    population: components.len(),
                                    traits: registry.get_traits(&met.lineage_id),
                                    is_radiation_storm: env.is_radiation_storm(),
                                    rng: &mut local_rng,
                                    ancestral_genotype: ancestral,
                                };
                                let (baby, dist) =
                                    social::reproduce_sexual_parallel_components_decomposed(
                                        pos,
                                        met,
                                        intel,
                                        partner_pos,
                                        partner_met,
                                        partner_intel,
                                        &mut repro_ctx,
                                    );
                                acc.push(InteractionCommand::Birth {
                                    parent_idx: i,
                                    baby: Box::new(baby),
                                    genetic_distance: dist,
                                });
                                acc.push(InteractionCommand::TransferEnergy {
                                    target_idx: p_idx,
                                    amount: -(partner_met.energy
                                        * partner_intel.genotype.reproductive_investment as f64),
                                });
                            }
                            let self_energy = met.energy;
                            let partner_energy = snapshots[p_idx].energy;
                            if self_energy > partner_energy + 2.0 {
                                let diff = self_energy - partner_energy;
                                let amount = diff * 0.05;
                                if amount > 0.1 {
                                    acc.push(InteractionCommand::TransferEnergy {
                                        target_idx: p_idx,
                                        amount,
                                    });
                                    acc.push(InteractionCommand::TransferEnergy {
                                        target_idx: i,
                                        amount: -amount,
                                    });
                                }
                            }
                        }
                    }

                    if outputs[4] > 0.5 && met.energy > met.max_energy * 0.7 {
                        spatial_hash.query_callback(pos.x, pos.y, 3.0, |t_idx| {
                            let target_phys = components[t_idx].1 .3;
                            if i != t_idx
                                && social::are_same_tribe_components(phys, target_phys, config)
                            {
                                let target_met = components[t_idx].1 .4;
                                if target_met.energy < target_met.max_energy * 0.5 {
                                    acc.push(InteractionCommand::TransferEnergy {
                                        target_idx: t_idx,
                                        amount: met.energy * 0.1,
                                    });
                                    acc.push(InteractionCommand::TransferEnergy {
                                        target_idx: i,
                                        amount: -met.energy * 0.1,
                                    });
                                }
                            }
                        });
                    }

                    if outputs[3] > 0.5 {
                        spatial_hash.query_callback(pos.x, pos.y, 1.5, |t_idx| {
                            if i != t_idx {
                                let target_snap = &snapshots[t_idx];
                                let color_dist = (phys.r as i32 - target_snap.r as i32).abs()
                                    + (phys.g as i32 - target_snap.g as i32).abs()
                                    + (phys.b as i32 - target_snap.b as i32).abs();

                                if color_dist >= config.social.tribe_color_threshold {
                                    let mut multiplier = 1.0;
                                    let attacker_status = lifecycle::calculate_status(
                                        met,
                                        health,
                                        intel,
                                        config.brain.activation_threshold,
                                        tick,
                                        config.metabolism.maturity_age,
                                    );
                                    if attacker_status == primordium_data::EntityStatus::Soldier
                                        || intel.specialization
                                            == Some(primordium_data::Specialization::Soldier)
                                    {
                                        multiplier *= config.social.soldier_damage_mult;
                                    }

                                    let mut allies = 0;
                                    spatial_hash.query_callback(
                                        target_snap.x,
                                        target_snap.y,
                                        2.0,
                                        |n_idx| {
                                            if n_idx != t_idx {
                                                let n_snap = &snapshots[n_idx];
                                                let n_color_dist =
                                                    (target_snap.r as i32 - n_snap.r as i32).abs()
                                                        + (target_snap.g as i32 - n_snap.g as i32)
                                                            .abs()
                                                        + (target_snap.b as i32 - n_snap.b as i32)
                                                            .abs();
                                                if n_color_dist
                                                    < config.social.tribe_color_threshold
                                                {
                                                    allies += 1;
                                                }
                                            }
                                        },
                                    );

                                    let defense_mult = (1.0 - allies as f64 * 0.15).max(0.4);
                                    let success_chance =
                                        (multiplier * defense_mult).min(1.0) as f32;

                                    let competition_mult = (1.0
                                        - (self.pop_stats.biomass_c
                                            / config.ecosystem.predation_competition_scale))
                                        .max(config.ecosystem.predation_min_efficiency);

                                    let energy_gain = target_snap.energy
                                        * config.ecosystem.predation_energy_gain_fraction
                                        * multiplier
                                        * defense_mult
                                        * competition_mult;

                                    acc.push(InteractionCommand::Kill {
                                        target_idx: t_idx,
                                        attacker_idx: i,
                                        attacker_lineage: met.lineage_id,
                                        cause: "Predation".to_string(),
                                        precalculated_energy_gain: energy_gain,
                                        success_chance,
                                    });
                                }
                            }
                        });
                    }

                    if lifecycle::is_mature_components(
                        met,
                        intel,
                        tick,
                        config.metabolism.maturity_age,
                    ) && met.energy > config.metabolism.reproduction_threshold
                    {
                        let mut repro_ctx = ReproductionContext {
                            tick,
                            config,
                            population: components.len(),
                            traits: registry.get_traits(&met.lineage_id),
                            is_radiation_storm: env.is_radiation_storm(),
                            rng: &mut local_rng,
                            ancestral_genotype: registry
                                .lineages
                                .get(&met.lineage_id)
                                .and_then(|r| r.max_fitness_genotype.as_ref()),
                        };
                        let (baby, dist) = social::reproduce_asexual_parallel_components_decomposed(
                            pos,
                            met,
                            intel,
                            &mut repro_ctx,
                        );
                        acc.push(InteractionCommand::Birth {
                            parent_idx: i,
                            baby: Box::new(baby),
                            genetic_distance: dist,
                        });
                    }

                    if (outputs[9] > 0.5 || outputs[10] > 0.5) && met.has_metamorphosed {
                        if outputs[9] > outputs[10] {
                            acc.push(InteractionCommand::Dig {
                                x: phys.x,
                                y: phys.y,
                                attacker_idx: i,
                            });
                        } else {
                            let build_val = outputs[10];
                            let spec = if build_val > 0.9 {
                                Some(primordium_data::OutpostSpecialization::Nursery)
                            } else if build_val > 0.8 {
                                Some(primordium_data::OutpostSpecialization::Silo)
                            } else {
                                Some(primordium_data::OutpostSpecialization::Standard)
                            };
                            acc.push(InteractionCommand::Build {
                                x: phys.x,
                                y: phys.y,
                                attacker_idx: i,
                                is_nest: true,
                                is_outpost: build_val > 0.8,
                                outpost_spec: spec,
                            });
                        }
                    }

                    if let Some(new_color) = social::start_tribal_split_components(
                        phys,
                        met,
                        intel,
                        spatial_hash.count_nearby(phys.x, phys.y, phys.sensing_range) as f32 / 10.0,
                        config,
                        &mut local_rng,
                    ) {
                        acc.push(InteractionCommand::TribalSplit {
                            target_idx: i,
                            new_color,
                        });
                    }

                    if !met.has_metamorphosed
                        && (tick - met.birth_tick)
                            > (config.metabolism.maturity_age as f32
                                * intel.genotype.maturity_gene
                                * config.metabolism.metamorphosis_trigger_maturity)
                                as u64
                    {
                        acc.push(InteractionCommand::Metamorphosis { target_idx: i });
                    }

                    acc
                },
            )
            .reduce(Vec::new, |mut a, b| {
                a.extend(b);
                a
            });

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
        let pheromones = &self.pheromones;
        let sound = &self.sound;
        let width = self.width;
        let height = self.height;

        let mut query = self.ecs.query::<(
            &primordium_data::Identity,
            &mut primordium_data::Position,
            &mut primordium_data::Velocity,
            &primordium_data::Physics,
            &mut primordium_data::Metabolism,
            &mut primordium_data::Intel,
        )>();
        let mut components: Vec<_> = query.iter().collect();

        let (total_oxygen_drain, overmind_broadcasts): (f64, Vec<(uuid::Uuid, f32)>) = components
            .par_iter_mut()
            .zip(self.decision_buffer.par_iter_mut())
            .map(
                |((_handle, (identity, pos, velocity, phys, met, intel)), decision)| {
                    let EntityDecision {
                        outputs,
                        next_hidden,
                        activations,
                    } = std::mem::take(decision);
                    intel.last_hidden = next_hidden;
                    intel.last_activations = activations;
                    intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;

                    let mut output = action::ActionOutput::default();
                    action::action_system_components(
                        &identity.id,
                        pos,
                        velocity,
                        phys,
                        met,
                        intel,
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

                    for p in output.pheromones {
                        pheromones.deposit_parallel(p.x, p.y, p.ptype, p.amount);
                    }
                    for s in output.sounds {
                        sound.deposit_parallel(s.x, s.y, s.amount);
                    }
                    for pr in output.pressure {
                        pressure_grid.deposit_parallel(pr.x, pr.y, pr.ptype, pr.amount);
                    }

                    (output.oxygen_drain, output.overmind_broadcast)
                },
            )
            .fold(
                || (0.0, Vec::new()),
                |mut acc, (drain, broadcast)| {
                    acc.0 += drain;
                    if let Some(b) = broadcast {
                        acc.1.push(b);
                    }
                    acc
                },
            )
            .reduce(
                || (0.0, Vec::new()),
                |mut a, b| {
                    a.0 += b.0;
                    a.1.extend(b.1);
                    a
                },
            );

        env.consume_oxygen(total_oxygen_drain);
        for (l_id, amount) in overmind_broadcasts {
            self.lineage_registry
                .set_memory_value(&l_id, "overmind", amount);
        }
    }

    fn execute_interactions(
        &mut self,
        env: &mut Environment,
        interaction_commands: Vec<InteractionCommand>,
        entity_handles: &[hecs::Entity],
        food_handles: &[hecs::Entity],
    ) -> (Vec<LiveEvent>, Vec<Entity>) {
        let (state_cmds, struct_cmds): (Vec<_>, Vec<_>) =
            interaction_commands.into_iter().partition(|cmd| {
                matches!(
                    cmd,
                    InteractionCommand::TransferEnergy { .. }
                        | InteractionCommand::UpdateReputation { .. }
                        | InteractionCommand::Fertilize { .. }
                )
            });

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
            food_handles,
            spatial_hash: &self.spatial_hash,
            rng: &mut self.rng,
        };

        interaction::process_interaction_commands_ecs(
            &mut self.ecs,
            entity_handles,
            state_cmds,
            &mut interaction_ctx,
        );

        let interaction_result = interaction::process_interaction_commands_ecs(
            &mut self.ecs,
            entity_handles,
            struct_cmds,
            &mut interaction_ctx,
        );

        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();

        self.killed_ids = interaction_result.killed_ids;
        self.eaten_food_indices = interaction_result.eaten_food_indices;

        (interaction_result.events, interaction_result.new_babies)
    }

    fn update_rank_grid(&mut self) {
        let mut rank_grid = vec![0.0f32; self.width as usize * self.height as usize];
        let width = self.width as usize;
        let height = self.height as usize;

        for (_handle, (phys, intel)) in self.ecs.query::<(&Physics, &Intel)>().iter() {
            let ex = phys.x as i32;
            let ey = phys.y as i32;
            let r = 3;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = ex + dx;
                    let ny = ey + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let idx = (ny as usize * width) + nx as usize;
                        let dist_sq = dx * dx + dy * dy;
                        let weight = (1.0 - (dist_sq as f32 / (r * r) as f32).sqrt()).max(0.0);
                        rank_grid[idx] += intel.rank * weight;
                    }
                }
            }
        }
        self.cached_rank_grid = std::sync::Arc::new(rank_grid);
    }

    fn finalize_tick(
        &mut self,
        env: &mut Environment,
        events: &mut Vec<LiveEvent>,
        entity_handles: &[hecs::Entity],
        new_babies: Vec<Entity>,
    ) {
        let config = &self.config;
        let tick = self.tick;

        {
            let mut query = self
                .ecs
                .query::<(&mut Metabolism, &mut Intel, &mut Health, &Physics)>();
            let mut components: Vec<_> = query.iter().collect();
            let population_count = components.len();

            components.par_iter_mut().enumerate().for_each(
                |(i, (_handle, (met, intel, health, phys)))| {
                    let mut rng = ChaCha8Rng::seed_from_u64(tick ^ i as u64);
                    biological::biological_system_components(
                        met,
                        intel,
                        health,
                        phys,
                        population_count,
                        config,
                        &mut rng,
                    );
                },
            );
        }

        for &handle in entity_handles {
            biological::handle_infection_ecs(
                handle,
                &mut self.ecs,
                entity_handles,
                &self.killed_ids,
                &self.active_pathogens,
                &self.spatial_hash,
                &mut self.rng,
            );
        }

        let mut dead_handles = Vec::new();
        for &handle in entity_handles {
            let is_dead = if let (Ok(identity), Ok(metabolism)) = (
                self.ecs.get::<&primordium_data::Identity>(handle),
                self.ecs.get::<&Metabolism>(handle),
            ) {
                self.killed_ids.contains(&identity.id) || metabolism.energy <= 0.0
            } else {
                false
            };

            if is_dead {
                dead_handles.push(handle);
            }
        }

        for handle in dead_handles {
            let met = self.ecs.get::<&Metabolism>(handle);
            let identity = self.ecs.get::<&primordium_data::Identity>(handle);

            if let (Ok(met), Ok(identity)) = (met, identity) {
                self.lineage_registry.record_death(met.lineage_id);

                if let (Ok(phys), Ok(intel), Ok(_health)) = (
                    self.ecs.get::<&Physics>(handle),
                    self.ecs.get::<&Intel>(handle),
                    self.ecs.get::<&Health>(handle),
                ) {
                    if let Some(legend) = social::archive_if_legend_components(
                        &identity,
                        &met,
                        &intel,
                        &phys,
                        tick,
                        &self.logger,
                    ) {
                        history::update_best_legend(
                            &mut self.lineage_registry,
                            &mut self.best_legends,
                            legend,
                        );
                    }
                    let fertilize_amount = (met.max_energy
                        * self.config.ecosystem.corpse_fertility_mult as f64)
                        as f32
                        / 100.0;
                    self.terrain.fertilize(phys.x, phys.y, fertilize_amount);
                    self.terrain
                        .add_biomass(phys.x, phys.y, fertilize_amount * 10.0);
                }
            }
            let _ = self.ecs.despawn(handle);
        }

        for baby in new_babies {
            self.ecs.spawn((
                baby.identity,
                baby.position,
                baby.velocity,
                baby.appearance,
                baby.physics,
                baby.metabolism,
                baby.health,
                baby.intel,
            ));
        }

        if !self.eaten_food_indices.is_empty() {
            self.food_dirty = true;
        }

        self.killed_ids.clear();
        self.eaten_food_indices.clear();

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

        civilization::handle_outposts_ecs(
            &mut self.terrain,
            &mut self.ecs,
            entity_handles,
            &self.spatial_hash,
            &self.entity_snapshots,
            self.width,
            self.config.social.silo_energy_capacity,
            self.config.social.outpost_energy_capacity,
        );

        // Phase 66: Resolve contested ownership and specialization upgrades
        civilization::resolve_contested_ownership(
            &mut self.terrain,
            self.width,
            self.height,
            &self.spatial_hash,
            &self.entity_snapshots,
            &self.lineage_registry,
        );
        civilization::resolve_outpost_upgrades(
            &mut self.terrain,
            self.width,
            self.height,
            &self.spatial_hash,
            &self.entity_snapshots,
            &self.lineage_registry,
        );

        if self
            .tick
            .is_multiple_of(self.config.world.power_grid_interval)
        {
            civilization::resolve_power_grid(
                &mut self.terrain,
                self.width,
                self.height,
                &self.lineage_registry,
            );
        }

        let snapshot = self.create_snapshot(None);
        stats::update_stats(
            self.tick,
            &snapshot.entities,
            snapshot.food.len(),
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

        if self.tick.is_multiple_of(10) {
            self.update_rank_grid();
        }
    }

    pub fn get_population_count(&self) -> usize {
        self.ecs
            .query::<&primordium_data::Identity>()
            .iter()
            .count()
    }

    pub fn get_food_count(&self) -> usize {
        self.ecs.query::<&Food>().iter().count()
    }

    pub fn get_all_entities(&self) -> Vec<primordium_data::Entity> {
        let mut entities = Vec::new();
        for (
            _handle,
            (identity, position, velocity, appearance, physics, metabolism, health, intel),
        ) in self
            .ecs
            .query::<(
                &primordium_data::Identity,
                &primordium_data::Position,
                &primordium_data::Velocity,
                &primordium_data::Appearance,
                &primordium_data::Physics,
                &primordium_data::Metabolism,
                &primordium_data::Health,
                &primordium_data::Intel,
            )>()
            .iter()
        {
            entities.push(primordium_data::Entity {
                identity: identity.clone(),
                position: *position,
                velocity: velocity.clone(),
                appearance: appearance.clone(),
                physics: physics.clone(),
                metabolism: metabolism.clone(),
                health: health.clone(),
                intel: intel.clone(),
            });
        }
        entities.sort_by_key(|e| e.identity.id);
        entities
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
        entity.velocity.vx = 1.0;
        entity.velocity.vy = 1.0;
        action::handle_movement(&mut entity, 1.0, &world.terrain, world.width, world.height);
        assert!(entity.velocity.vx < 0.0);
    }
}
