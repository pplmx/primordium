use crate::model::config::AppConfig;
use crate::model::history::{FossilRegistry, HallOfFame, HistoryLogger, PopulationStats};
use crate::model::lifecycle;
use crate::model::lineage_registry::LineageRegistry;
use crate::model::observer::WorldObserver;
use crate::model::pheromone::PheromoneGrid;
use crate::model::sound::SoundGrid;
use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::TerrainGrid;
use crate::model::world::World;
use primordium_data::{Food, MetabolicNiche, Position};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::Arc;

impl World {
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
            cached_terrain: Arc::new(terrain.clone()),
            cached_pheromones: Arc::new(pheromones.clone()),
            cached_sound: Arc::new(sound.clone()),
            cached_pressure: Arc::new(pressure.clone()),
            cached_social_grid: Arc::new(social_grid.clone()),
            cached_rank_grid: Arc::new(vec![
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
            killed_ids: Default::default(),
            eaten_food_indices: Default::default(),
            decision_buffer: Vec::new(),
            lineage_consumption: Vec::new(),
            entity_snapshots: Vec::new(),
            food_dirty: true,
            spatial_data_buffer: Vec::new(),
            food_positions_buffer: Vec::new(),
        })
    }

    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        let log_dir = "logs".to_string();
        Self::new_at(initial_population, config, &log_dir)
    }

    pub fn load_persistent(&mut self) -> anyhow::Result<()> {
        self.lineage_registry =
            LineageRegistry::load(format!("{}/lineages.json", self.log_dir)).unwrap_or_default();
        self.fossil_registry =
            FossilRegistry::load(&format!("{}/fossils.json.gz", self.log_dir)).unwrap_or_default();
        Ok(())
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
}
