use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Type of neural network node.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum NodeType {
    Input,
    Hidden,
    Output,
}

/// A node in the neural network brain.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub label: Option<String>,
}

/// A connection between two nodes in the neural network.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
    pub enabled: bool,
    pub innovation: usize,
}

/// The neural network brain of an organism.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Brain {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub next_node_id: usize,
    pub learning_rate: f32,
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub weight_deltas: HashMap<usize, f32>,
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub node_idx_map: HashMap<usize, usize>,
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub topological_order: Vec<usize>,
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub forward_connections: Vec<usize>,
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub recurrent_connections: Vec<usize>,
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub incoming_forward_connections: HashMap<usize, Vec<usize>>,
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub fast_forward_order: Vec<usize>,
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub fast_incoming: Vec<Vec<(usize, usize)>>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
pub enum Specialization {
    Soldier,
    Engineer,
    Provider,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum RegulatorySensor {
    Oxygen,
    Carbon,
    EnergyRatio,
    NearbyKin,
    AgeRatio,
    Clock,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum RegulatoryOperator {
    GreaterThan,
    LessThan,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum GeneType {
    Trophic,
    Sensing,
    Speed,
    ReproInvest,
    Maturity,
    MaxEnergy,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct RegulatoryRule {
    pub sensor: RegulatorySensor,
    pub threshold: f32,
    pub operator: RegulatoryOperator,
    pub target: GeneType,
    pub modifier: f32,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
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
    pub specialization_bias: [f32; 3], // Soldier, Engineer, Provider
    pub regulatory_rules: Vec<RegulatoryRule>,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Activations(pub Vec<f32>, pub Vec<f32>);

impl Default for Activations {
    fn default() -> Self {
        Self(vec![0.0; 64], vec![0.0; 64])
    }
}

impl Activations {
    pub fn prepare(&mut self, node_count: usize) {
        if self.0.len() != node_count {
            self.0.resize(node_count, 0.0);
        } else {
            self.0.fill(0.0);
        }
    }
}

impl Brain {
    #[must_use]
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
    #[must_use]
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
