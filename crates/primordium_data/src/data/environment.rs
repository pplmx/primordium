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
/// Epigenetic traits accumulated by high-fitness lineages across generations.
pub enum AncestralTrait {
    /// Reduced metabolic costs under environmental stress.
    HardenedMetabolism,
    /// Extended sensory perception range.
    AcuteSenses,
    /// Increased maximum movement speed.
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
/// Shared strategic objective for an entire lineage, influencing collective neural bias.
pub enum LineageGoal {
    /// Prioritise territory spread and population growth.
    Expansion,
    /// Prioritise combat supremacy and resource control.
    Dominance,
    /// Prioritise survival under environmental stress.
    Resilience,
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
/// Aggregated macro-level statistics for the entire simulation population.
pub struct PopulationStats {
    /// Total number of living entities.
    pub population: usize,
    /// Mean lifespan across all living entities (in ticks).
    pub avg_lifespan: f64,
    /// Mean Shannon entropy of all neural network weight distributions.
    pub avg_brain_entropy: f64,
    /// Number of distinct lineages currently alive.
    pub species_count: usize,
    /// Highest fitness score among all living entities.
    pub top_fitness: f64,
    /// Mean fitness score across all living entities (for DDA).
    pub avg_fitness: f64,
    /// Total biomass of herbivore-niche entities.
    pub biomass_h: f64,
    /// Total biomass of carnivore-niche entities.
    pub biomass_c: f64,
    /// Number of food items currently on the map.
    pub food_count: usize,
    /// Per-lineage population counts keyed by lineage UUID.
    pub lineage_counts: HashMap<Uuid, usize>,
    /// Global atmospheric carbon level (drives climate shifts).
    pub carbon_level: f64,
    /// Number of grid regions classified as biodiversity hotspots.
    pub biodiversity_hotspots: usize,
    /// Population-aware mutation multiplier (0.5×–3.0×).
    pub mutation_scale: f32,
    /// Rate of phenotypic change across generations.
    pub evolutionary_velocity: f32,
    /// Average soil fertility across all terrain cells.
    pub global_fertility: f32,
    /// Highest generation number among all living entities.
    pub max_generation: u32,
    /// Rolling window of recent death ages for mortality analysis.
    pub recent_deaths: VecDeque<f64>,
    /// Rolling window of recent migration distances.
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
            avg_fitness: 0.0,
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
/// Tagged union of all simulation events emitted during a tick.
///
/// Serialised with `#[serde(tag = "event")]` for streaming JSONL output.
pub enum LiveEvent {
    /// A new entity has been born (spontaneous or via reproduction).
    Birth {
        id: Uuid,
        parent_id: Option<Uuid>,
        gen: u32,
        tick: u64,
        timestamp: String,
        x: Option<f64>,
        y: Option<f64>,
    },
    /// An entity has died.
    Death {
        id: Uuid,
        age: u64,
        offspring: u32,
        tick: u64,
        timestamp: String,
        cause: String,
        x: Option<f64>,
        y: Option<f64>,
    },
    /// An entity completed larval-to-adult metamorphosis.
    Metamorphosis {
        id: Uuid,
        name: String,
        tick: u64,
        timestamp: String,
    },
    /// A tribe split due to overcrowding or low-rank fragmentation.
    TribalSplit {
        id: Uuid,
        lineage_id: Uuid,
        tick: u64,
        timestamp: String,
    },
    /// Global climate state transitioned (e.g. Temperate → Warm).
    ClimateShift {
        from: String,
        to: String,
        tick: u64,
        timestamp: String,
    },
    /// Population dropped to zero — total extinction.
    Extinction {
        population: usize,
        tick: u64,
        timestamp: String,
    },
    /// Ecological stability alert (e.g. trophic cascade, overpopulation).
    EcoAlert {
        message: String,
        tick: u64,
        timestamp: String,
    },
    /// Periodic macro-state snapshot for history browsing.
    Snapshot {
        tick: u64,
        stats: PopulationStats,
        timestamp: String,
    },
    /// AI narrator commentary on current world state.
    Narration {
        tick: u64,
        text: String,
        severity: f32,
        timestamp: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
/// Leaderboard tracking the top-fitness living entities.
pub struct HallOfFame {
    /// Top living entities sorted by fitness score (max 3).
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
/// Preserved record of an extinct lineage archived in the fossil registry.
pub struct Fossil {
    /// UUID of the extinct lineage.
    pub lineage_id: Uuid,
    /// Display name of the lineage.
    pub name: String,
    /// RGB colour signature of the lineage.
    pub color_rgb: (u8, u8, u8),
    /// Mean lifespan of members at time of extinction (ticks).
    pub avg_lifespan: f64,
    /// Highest generation reached before extinction.
    pub max_generation: u32,
    /// Cumulative offspring produced across the lineage's history.
    pub total_offspring: u32,
    /// Tick at which the last member died.
    pub extinct_tick: u64,
    /// Largest population the lineage ever achieved.
    pub peak_population: usize,
    /// Representative genotype preserved for atavistic recall.
    pub genotype: Genotype,
}

#[derive(Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
/// Biographical record of a notable deceased entity preserved for ancestry analysis.
pub struct Legend {
    /// Unique entity identifier.
    pub id: Uuid,
    /// Parent entity identifier (`None` for spontaneously generated).
    pub parent_id: Option<Uuid>,
    /// Lineage this entity belonged to.
    pub lineage_id: Uuid,
    /// Tick at which the entity was born.
    pub birth_tick: u64,
    /// Tick at which the entity died.
    pub death_tick: u64,
    /// Total ticks the entity survived.
    pub lifespan: u64,
    /// Generational depth from lineage founder.
    pub generation: u32,
    /// Number of offspring produced during lifetime.
    pub offspring_count: u32,
    /// Highest energy level ever recorded for this entity.
    pub peak_energy: f64,
    /// ISO 8601 timestamp of birth.
    pub birth_timestamp: String,
    /// ISO 8601 timestamp of death.
    pub death_timestamp: String,
    /// Full genotype snapshot at time of death.
    pub genotype: Genotype,
    /// RGB colour at time of death.
    pub color_rgb: (u8, u8, u8),
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize, Default,
)]
#[archive(check_bytes)]
/// Bounded collection of [`Fossil`] records, capped at 100 entries sorted by offspring count.
pub struct FossilRegistry {
    /// Archived fossils, ordered by `total_offspring` descending after cap enforcement.
    pub fossils: Vec<Fossil>,
}

impl FossilRegistry {
    /// Inserts a fossil, evicting the least-prolific entry when capacity exceeds 100.
    pub fn add_fossil(&mut self, fossil: Fossil) {
        self.fossils.push(fossil);
        if self.fossils.len() > 100 {
            self.fossils
                .sort_by(|a, b| b.total_offspring.cmp(&a.total_offspring));
            self.fossils.truncate(100);
        }
    }
}
