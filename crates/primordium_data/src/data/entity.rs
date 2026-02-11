use super::environment::AncestralTrait;
use super::genotype::{Activations, Genotype, Specialization};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// World position of an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Position {
    /// X coordinate in world space.
    pub x: f64,
    /// Y coordinate in world space.
    pub y: f64,
}

/// Metabolism niche specialization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct MetabolicNiche(pub f32);

/// Energy level of an entity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Energy(pub f64);

/// Velocity of an entity.
#[derive(
    Clone, Debug, Serialize, Deserialize, Default, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Velocity {
    /// X component of velocity.
    pub vx: f64,
    /// Y component of velocity.
    pub vy: f64,
}

/// Visual appearance of an entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Appearance {
    /// Red color component (0-255).
    pub r: u8,
    /// Green color component (0-255).
    pub g: u8,
    /// Blue color component (0-255).
    pub b: u8,
    /// Display symbol character.
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

/// Unique identification of an entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Identity {
    /// Unique entity identifier.
    pub id: Uuid,
    /// Parent entity identifier, if any.
    pub parent_id: Option<Uuid>,
}

/// Physical properties and state of an entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Physics {
    /// Home X coordinate (birth location).
    pub home_x: f64,
    /// Home Y coordinate (birth location).
    pub home_y: f64,
    /// Current X coordinate.
    pub x: f64,
    /// Current Y coordinate.
    pub y: f64,
    /// Velocity X component.
    pub vx: f64,
    /// Velocity Y component.
    pub vy: f64,
    /// Red color component.
    pub r: u8,
    /// Green color component.
    pub g: u8,
    /// Blue color component.
    pub b: u8,
    /// Display symbol.
    pub symbol: char,
    /// Sensing range radius.
    pub sensing_range: f64,
    /// Maximum movement speed.
    pub max_speed: f64,
}

/// Metabolic state and history of an entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Metabolism {
    /// Trophic level (0.0=herbivore, 1.0=carnivore).
    pub trophic_potential: f32,
    /// Current energy level.
    pub energy: f64,
    /// Previous energy level (not serialized).
    #[serde(skip)]
    pub prev_energy: f64,
    /// Maximum energy capacity.
    pub max_energy: f64,
    /// Peak energy achieved in lifetime.
    pub peak_energy: f64,
    /// Tick when entity was born.
    pub birth_tick: u64,
    /// Generation number.
    pub generation: u32,
    /// Number of offspring produced.
    pub offspring_count: u32,
    /// Lineage identifier.
    pub lineage_id: Uuid,
    /// Whether entity has metamorphosed to adult.
    pub has_metamorphosed: bool,
    /// Whether entity is currently migrating.
    #[serde(default)]
    pub is_in_transit: bool,
    /// Migration batch identifier.
    #[serde(default)]
    pub migration_id: Option<Uuid>,
}

/// Pathogen state for infection simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Pathogen {
    /// Unique pathogen identifier.
    pub id: Uuid,
    /// Lethality (0.0-1.0).
    pub lethality: f32,
    /// Transmission rate (0.0-1.0).
    pub transmission: f32,
    /// Infection duration in ticks.
    pub duration: u32,
    /// Virulence factor.
    pub virulence: f32,
    /// Optional behavior manipulation (node index, intensity).
    pub behavior_manipulation: Option<(usize, f32)>,
}

/// Health and immunity state of an entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Health {
    /// Active pathogen infection, if any.
    pub pathogen: Option<Pathogen>,
    /// Time remaining in current infection.
    pub infection_timer: u32,
    /// Immunity level (0.0-1.0).
    pub immunity: f32,
}

/// The cognitive state of an organism.
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Intel {
    /// Neural network genotype.
    pub genotype: std::sync::Arc<Genotype>,
    /// Hidden layer values from previous tick (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_hidden: [f32; 6],
    /// Last aggression output (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_aggression: f32,
    /// Last share intent output (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_share_intent: f32,
    /// Last signal output (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_signal: f32,
    /// Last vocalization output (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_vocalization: f32,
    /// Social reputation score (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub reputation: f32,
    /// Social rank (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub rank: f32,
    /// Bonded partner ID (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub bonded_to: Option<Uuid>,
    /// Last neural network inputs (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_inputs: [f32; 29],
    /// Last neural network activations (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_activations: Activations,
    /// Current caste specialization (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub specialization: Option<Specialization>,
    /// Specialization progress meters (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub spec_meters: HashMap<Specialization, f32>,
    /// Inherited ancestral traits (not serialized).
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub ancestral_traits: HashSet<AncestralTrait>,
}

/// A complete organism entity.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Entity {
    /// Identity information (flattened).
    #[serde(flatten)]
    pub identity: Identity,
    /// World position.
    pub position: Position,
    /// Velocity vector.
    pub velocity: Velocity,
    /// Visual appearance.
    pub appearance: Appearance,
    /// Physical properties.
    pub physics: Physics,
    /// Metabolic state.
    pub metabolism: Metabolism,
    /// Health state.
    pub health: Health,
    /// Cognitive state.
    pub intel: Intel,
}

/// Enumeration of possible entity life stages and states.
#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum EntityStatus {
    /// Entity is starving (low energy).
    Starving,
    /// Entity is in larval stage.
    Larva,
    /// Entity is juvenile (not yet adult).
    Juvenile,
    /// Entity is infected with a pathogen.
    Infected,
    /// Entity is sharing energy.
    Sharing,
    /// Entity is mating.
    Mating,
    /// Entity is hunting prey.
    Hunting,
    /// Entity is foraging for food.
    Foraging,
    /// Entity is in soldier caste.
    Soldier,
    /// Entity is bonded to a partner.
    Bonded,
    /// Entity is migrating between worlds.
    InTransit,
}
