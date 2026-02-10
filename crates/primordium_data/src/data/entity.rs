use super::environment::AncestralTrait;
use super::genotype::{Activations, Genotype, Specialization};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct MetabolicNiche(pub f32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Energy(pub f64);

#[derive(
    Clone, Debug, Serialize, Deserialize, Default, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Identity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Pathogen {
    pub id: Uuid,
    pub lethality: f32,
    pub transmission: f32,
    pub duration: u32,
    pub virulence: f32,
    pub behavior_manipulation: Option<(usize, f32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Health {
    pub pathogen: Option<Pathogen>,
    pub infection_timer: u32,
    pub immunity: f32,
}

/// The cognitive state of an organism.
#[derive(
    Serialize, Deserialize, Debug, Clone, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Intel {
    pub genotype: std::sync::Arc<Genotype>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_hidden: [f32; 6],
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_aggression: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_share_intent: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_signal: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_vocalization: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub reputation: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub rank: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub bonded_to: Option<Uuid>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_inputs: [f32; 29],
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub last_activations: Activations,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub specialization: Option<Specialization>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub spec_meters: HashMap<Specialization, f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub ancestral_traits: HashSet<AncestralTrait>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
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

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
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
