use super::entity::Entity;
use super::genotype::Genotype;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
pub enum AncestralTrait {
    HardenedMetabolism,
    AcuteSenses,
    SwiftMovement,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum LineageGoal {
    Expansion,
    Dominance,
    Resilience,
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct PopulationStats {
    pub population: usize,
    pub avg_lifespan: f64,
    pub avg_brain_entropy: f64,
    pub species_count: usize,
    pub top_fitness: f64,
    pub biomass_h: f64,
    pub biomass_c: f64,
    pub food_count: usize,
    pub lineage_counts: HashMap<Uuid, usize>,
    pub carbon_level: f64,
    pub biodiversity_hotspots: usize,
    pub mutation_scale: f32,
    pub evolutionary_velocity: f32,
    pub global_fertility: f32,
    pub max_generation: u32,
    pub recent_deaths: VecDeque<f64>,
    pub recent_distances: VecDeque<f32>,
}

impl Default for PopulationStats {
    fn default() -> Self {
        Self {
            population: 0,
            avg_lifespan: 0.0,
            avg_brain_entropy: 0.0,
            species_count: 0,
            top_fitness: 0.0,
            biomass_h: 0.0,
            biomass_c: 0.0,
            food_count: 0,
            lineage_counts: HashMap::new(),
            carbon_level: 0.0,
            biodiversity_hotspots: 0,
            mutation_scale: 1.0,
            evolutionary_velocity: 0.0,
            global_fertility: 1.0,
            max_generation: 0,
            recent_deaths: VecDeque::with_capacity(100),
            recent_distances: VecDeque::with_capacity(100),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[serde(tag = "event")]
#[archive(check_bytes)]
pub enum LiveEvent {
    Birth {
        id: Uuid,
        parent_id: Option<Uuid>,
        gen: u32,
        tick: u64,
        timestamp: String,
    },
    Death {
        id: Uuid,
        age: u64,
        offspring: u32,
        tick: u64,
        timestamp: String,
        cause: String,
    },
    Metamorphosis {
        id: Uuid,
        name: String,
        tick: u64,
        timestamp: String,
    },
    TribalSplit {
        id: Uuid,
        lineage_id: Uuid,
        tick: u64,
        timestamp: String,
    },
    ClimateShift {
        from: String,
        to: String,
        tick: u64,
        timestamp: String,
    },
    Extinction {
        population: usize,
        tick: u64,
        timestamp: String,
    },
    EcoAlert {
        message: String,
        tick: u64,
        timestamp: String,
    },
    Snapshot {
        tick: u64,
        stats: PopulationStats,
        timestamp: String,
    },
    Narration {
        tick: u64,
        text: String,
        severity: f32,
        timestamp: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct HallOfFame {
    pub top_living: Vec<(f64, Entity)>,
}

impl Default for HallOfFame {
    fn default() -> Self {
        Self {
            top_living: Vec::with_capacity(3),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Fossil {
    pub lineage_id: Uuid,
    pub name: String,
    pub color_rgb: (u8, u8, u8),
    pub avg_lifespan: f64,
    pub max_generation: u32,
    pub total_offspring: u32,
    pub extinct_tick: u64,
    pub peak_population: usize,
    pub genotype: Genotype,
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Legend {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub lineage_id: Uuid,
    pub birth_tick: u64,
    pub death_tick: u64,
    pub lifespan: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub peak_energy: f64,
    pub birth_timestamp: String,
    pub death_timestamp: String,
    pub genotype: Genotype,
    pub color_rgb: (u8, u8, u8),
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize, Default,
)]
#[archive(check_bytes)]
pub struct FossilRegistry {
    pub fossils: Vec<Fossil>,
}

impl FossilRegistry {
    pub fn add_fossil(&mut self, fossil: Fossil) {
        self.fossils.push(fossil);
        if self.fossils.len() > 100 {
            self.fossils
                .sort_by(|a, b| b.total_offspring.cmp(&a.total_offspring));
            self.fossils.truncate(100);
        }
    }
}
