use crate::pheromone::PheromoneGrid;
use crate::pressure::PressureGrid;
use crate::sound::SoundGrid;
use crate::terrain::TerrainGrid;
use primordium_data::Food;
use primordium_data::{EntityStatus, HallOfFame, PopulationStats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntitySnapshot {
    pub id: Uuid,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub energy: f64,
    pub max_energy: f64,
    pub generation: u32,
    pub age: u64,
    pub offspring: u32,
    pub lineage_id: Uuid,
    pub rank: f32,
    pub status: EntityStatus,
    pub trophic_potential: f32,
    pub bonded_to: Option<Uuid>,
    pub last_vocalization: f32,
    pub last_activations: HashMap<i32, f32>,
    pub weight_deltas: HashMap<usize, f32>,
    pub genotype_hex: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnapshot>,
    pub food: Vec<Food>,
    pub stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: Arc<TerrainGrid>,
    pub pheromones: Arc<PheromoneGrid>,
    pub sound: Arc<SoundGrid>,
    pub pressure: Arc<PressureGrid>,
    pub social_grid: Arc<Vec<u8>>,
    pub rank_grid: Arc<Vec<f32>>,
    pub width: u16,
    pub height: u16,
}
