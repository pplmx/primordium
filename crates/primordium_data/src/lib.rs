use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Input,
    Hidden,
    Output,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
    pub enabled: bool,
    pub innovation: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub next_node_id: usize,
    pub learning_rate: f32,
    #[serde(skip, default = "HashMap::new")]
    pub weight_deltas: HashMap<usize, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Specialization {
    Soldier,
    Engineer,
    Provider,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AncestralTrait {
    HardenedMetabolism,
    AcuteSenses,
    SwiftMovement,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LineageGoal {
    Expansion,
    Dominance,
    Resilience,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatorySensor {
    Oxygen,
    Carbon,
    EnergyRatio,
    NearbyKin,
    AgeRatio,
    Clock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulatoryOperator {
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegulatoryRule {
    pub sensor: RegulatorySensor,
    pub threshold: f32,
    pub operator: RegulatoryOperator,
    pub target: GeneType,
    pub modifier: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneType {
    Trophic,
    Sensing,
    Speed,
    ReproInvest,
    Maturity,
    MaxEnergy,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityStatus {
    Starving,
    Larva,
    Juvenile,
    Infected,
    Sharing,
    Mating,
    Hunting,
    Foraging,
    Soldier,
    Bonded,
    InTransit,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum TerrainType {
    #[default]
    Plains,
    Mountain,
    River,
    Oasis,
    Barren,
    Wall,
    Forest,
    Desert,
    Nest,
    Outpost,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum OutpostSpecialization {
    #[default]
    Standard,
    Silo,
    Nursery,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LineageInfo {
    pub lineage_id: Uuid,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ColorRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Appearance {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            symbol: '●',
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Physics {
    pub home_x: f64,
    pub home_y: f64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
    pub sensing_range: f64,
    pub max_speed: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    pub trophic_potential: f32,
    pub energy: f64,
    #[serde(skip)]
    pub prev_energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub birth_tick: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub lineage_id: Uuid,
    pub has_metamorphosed: bool,
    #[serde(default)]
    pub is_in_transit: bool,
    #[serde(default)]
    pub migration_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathogen {
    pub id: Uuid,
    pub lethality: f32,
    pub transmission: f32,
    pub duration: u32,
    pub virulence: f32,
    pub behavior_manipulation: Option<(usize, f32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub pathogen: Option<Pathogen>,
    pub infection_timer: u32,
    pub immunity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genotype {
    pub brain: Brain,
    pub sensing_range: f64,
    pub max_speed: f64,
    pub max_energy: f64,
    pub lineage_id: Uuid,
    pub metabolic_niche: f32,
    pub trophic_potential: f32,
    pub reproductive_investment: f32,
    pub maturity_gene: f32,
    pub mate_preference: f32,
    pub pairing_bias: f32,
    pub specialization_bias: [f32; 3],
    pub regulatory_rules: Vec<RegulatoryRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activations(pub Vec<f32>);

impl Default for Activations {
    fn default() -> Self {
        Self(vec![0.0; 64])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intel {
    pub genotype: Genotype,
    #[serde(skip)]
    pub last_hidden: [f32; 6],
    #[serde(skip)]
    pub last_aggression: f32,
    pub last_share_intent: f32,
    pub last_signal: f32,
    pub last_vocalization: f32,
    pub reputation: f32,
    pub rank: f32,
    pub bonded_to: Option<Uuid>,
    #[serde(skip)]
    pub last_inputs: [f32; 29],
    #[serde(skip)]
    pub last_activations: Activations,
    pub specialization: Option<Specialization>,
    pub spec_meters: HashMap<Specialization, f32>,
    pub ancestral_traits: HashSet<AncestralTrait>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    #[serde(flatten)]
    pub identity: Identity,
    pub position: Position,
    pub velocity: Velocity,
    pub appearance: Appearance,
    pub physics: Physics,
    pub metabolism: Metabolism,
    pub health: Health,
    pub intel: Intel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MetabolicNiche(pub f32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Energy(pub f64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub value: f64,
    pub symbol: char,
    pub color_rgb: (u8, u8, u8),
    pub nutrient_type: f32,
}

impl Food {
    pub fn new(x: u16, y: u16, nutrient_type: f32) -> Self {
        let color = if nutrient_type < 0.5 {
            (0, 255, 0)
        } else {
            (0, 100, 255)
        };

        Self {
            x,
            y,
            value: 50.0,
            symbol: '*',
            color_rgb: color,
            nutrient_type,
        }
    }
}

impl Brain {
    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let brain = serde_json::from_slice(&bytes)?;
        Ok(brain)
    }
}

impl Genotype {
    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let genotype = serde_json::from_slice(&bytes)?;
        Ok(genotype)
    }
}
