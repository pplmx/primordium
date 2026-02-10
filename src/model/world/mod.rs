use crate::model::config::AppConfig;
use crate::model::history::{FossilRegistry, HallOfFame, HistoryLogger, PopulationStats};
use crate::model::lineage_registry::LineageRegistry;
use crate::model::observer::WorldObserver;
use crate::model::pheromone::PheromoneGrid;
use crate::model::sound::SoundGrid;
use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::TerrainGrid;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

fn default_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(0)
}

pub mod finalize;
pub mod init;
pub mod logic;
pub mod state;
pub mod systems;
pub mod update;

pub use state::{EntityComponents, EntityDecision, InternalEntitySnapshot};

pub struct SystemContext<'a> {
    pub config: &'a AppConfig,
    pub ecs: &'a hecs::World,
    pub food_hash: &'a SpatialHash,
    pub spatial_hash: &'a SpatialHash,
    pub pheromones: &'a PheromoneGrid,
    pub sound: &'a SoundGrid,
    pub pressure: &'a crate::model::pressure::PressureGrid,
    pub influence: &'a crate::model::influence::InfluenceGrid,
    pub terrain: &'a TerrainGrid,
    pub tick: u64,
    pub registry: &'a LineageRegistry,
    pub snapshots: &'a [crate::model::snapshot::InternalEntitySnapshot],
    pub food_handles: &'a [hecs::Entity],
    pub food_data: &'a [(f64, f64, f32)],
    pub world_seed: u64,
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub width: u16,
    pub height: u16,
    pub tick: u64,
    #[serde(skip, default = "hecs::World::new")]
    pub ecs: hecs::World,

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
    pub influence: crate::model::influence::InfluenceGrid,
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
    #[serde(skip, default = "default_rng")]
    pub rng: ChaCha8Rng,
    #[serde(skip, default)]
    pub killed_ids: HashSet<uuid::Uuid>,
    #[serde(skip, default)]
    pub eaten_food_indices: HashSet<usize>,

    #[serde(skip, default)]
    pub decision_buffer: Vec<EntityDecision>,
    #[serde(skip, default)]
    pub interaction_buffer: Vec<primordium_core::interaction::InteractionCommand>,
    #[serde(skip, default)]
    pub lineage_consumption: Vec<(uuid::Uuid, f64)>,
    #[serde(skip, default)]
    pub entity_snapshots: Vec<crate::model::snapshot::InternalEntitySnapshot>,

    #[serde(skip)]
    pub cached_terrain: Arc<TerrainGrid>,
    #[serde(skip)]
    pub cached_pheromones: Arc<PheromoneGrid>,
    #[serde(skip)]
    pub cached_sound: Arc<SoundGrid>,
    #[serde(skip)]
    pub cached_pressure: Arc<crate::model::pressure::PressureGrid>,
    #[serde(skip)]
    pub cached_influence: Arc<crate::model::influence::InfluenceGrid>,
    #[serde(skip)]
    pub cached_social_grid: Arc<Vec<u8>>,
    #[serde(skip)]
    pub cached_rank_grid: Arc<Vec<f32>>,
    pub food_dirty: bool,
    #[serde(skip, default)]
    pub food_count: std::sync::atomic::AtomicUsize,
    #[serde(skip, default)]
    pub last_persistence_error: Option<String>,
    #[serde(skip, default)]
    pub spatial_data_buffer: Vec<(f64, f64, uuid::Uuid)>,
    #[serde(skip, default)]
    pub spatial_sort_buffer: Vec<(f64, f64, uuid::Uuid, uuid::Uuid)>,
    #[serde(skip, default)]
    pub food_positions_buffer: Vec<(f64, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use primordium_core::systems::action;
    #[tokio::test]
    async fn test_handle_movement_wall_bounce() {
        let mut config = AppConfig::default();
        config.world.width = 10;
        config.world.height = 10;
        let mut world = World::new(0, config).expect("Failed to create world");
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
