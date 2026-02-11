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
    /// Input node (receives sensory data).
    Input,
    /// Hidden node (internal processing).
    Hidden,
    /// Output node (produces actions).
    Output,
}

/// A node in the neural network brain.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Node {
    /// Unique node identifier.
    pub id: usize,
    /// Type of neural node.
    pub node_type: NodeType,
    /// Optional descriptive label.
    pub label: Option<String>,
}

/// A connection between two nodes in the neural network.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Connection {
    /// Source node ID.
    pub from: usize,
    /// Target node ID.
    pub to: usize,
    /// Connection weight.
    pub weight: f32,
    /// Whether connection is active.
    pub enabled: bool,
    /// Innovation number for NEAT crossover.
    pub innovation: usize,
}

/// The neural network brain of an organism.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Brain {
    /// All neural nodes.
    pub nodes: Vec<Node>,
    /// All neural connections.
    pub connections: Vec<Connection>,
    /// Next available node ID.
    pub next_node_id: usize,
    /// Learning rate for Hebbian plasticity.
    pub learning_rate: f32,
    /// Weight change cache (not serialized).
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub weight_deltas: HashMap<usize, f32>,
    /// Node ID to index mapping (not serialized).
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub node_idx_map: HashMap<usize, usize>,
    /// Topological sort order (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub topological_order: Vec<usize>,
    /// Forward connection indices (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub forward_connections: Vec<usize>,
    /// Recurrent connection indices (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub recurrent_connections: Vec<usize>,
    /// Incoming connections per node (not serialized).
    #[serde(skip, default = "HashMap::new")]
    #[with(rkyv::with::Skip)]
    pub incoming_forward_connections: HashMap<usize, Vec<usize>>,
    /// Optimized forward pass order (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub fast_forward_order: Vec<usize>,
    /// Flattened incoming connections (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub incoming_flat: Vec<(usize, usize)>,
    /// Offsets for flattened connections (not serialized).
    #[serde(skip, default = "Vec::new")]
    #[with(rkyv::with::Skip)]
    pub incoming_offsets: Vec<usize>,
}

/// Caste specialization for evolved entities.
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
    /// Combat-focused caste with damage bonuses.
    Soldier,
    /// Terraforming-focused caste with dig/build efficiency.
    Engineer,
    /// Sharing-focused caste with energy transfer efficiency.
    Provider,
}

/// Environmental sensor for genetic regulation.
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
    /// Oxygen level sensor.
    Oxygen,
    /// Carbon level sensor.
    Carbon,
    /// Energy ratio sensor.
    EnergyRatio,
    /// Nearby kin count sensor.
    NearbyKin,
    /// Age ratio sensor.
    AgeRatio,
    /// Circadian clock sensor.
    Clock,
}

/// Comparison operator for genetic regulation.
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
    /// Greater than comparison.
    GreaterThan,
    /// Less than comparison.
    LessThan,
}

/// Type of gene for regulatory targeting.
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
    /// Trophic potential gene.
    Trophic,
    /// Sensing range gene.
    Sensing,
    /// Max speed gene.
    Speed,
    /// Reproductive investment gene.
    ReproInvest,
    /// Maturity gene.
    Maturity,
    /// Max energy gene.
    MaxEnergy,
}

/// Genetic regulation rule for adaptive gene expression.
#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct RegulatoryRule {
    /// Sensor to monitor.
    pub sensor: RegulatorySensor,
    /// Threshold value for activation.
    pub threshold: f32,
    /// Comparison operator.
    pub operator: RegulatoryOperator,
    /// Target gene to modify.
    pub target: GeneType,
    /// Modifier value to apply.
    pub modifier: f32,
}

/// Complete genetic blueprint of an organism.
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct Genotype {
    /// Neural network brain.
    pub brain: Brain,
    /// Sensing range radius.
    pub sensing_range: f64,
    /// Maximum movement speed.
    pub max_speed: f64,
    /// Maximum energy capacity.
    pub max_energy: f64,
    /// Lineage identifier.
    pub lineage_id: Uuid,
    /// Metabolic niche (0.0-1.0).
    pub metabolic_niche: f32,
    /// Trophic potential (0.0=herbivore, 1.0=carnivore).
    pub trophic_potential: f32,
    /// Reproductive investment (0.1-0.9).
    pub reproductive_investment: f32,
    /// Maturity gene (0.5-2.0).
    pub maturity_gene: f32,
    /// Mate preference threshold.
    pub mate_preference: f32,
    /// Pairing bias for mating.
    pub pairing_bias: f32,
    /// Specialization bias [Soldier, Engineer, Provider].
    pub specialization_bias: [f32; 3],
    /// Genetic regulation rules.
    pub regulatory_rules: Vec<RegulatoryRule>,
}

/// Neural network activation buffers.
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
    /// Prepare activation buffer for given node count.
    pub fn prepare(&mut self, node_count: usize) {
        if self.0.len() != node_count {
            self.0.resize(node_count, 0.0);
        } else {
            self.0.fill(0.0);
        }
    }
}

impl Brain {
    /// Serialize brain to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    /// Deserialize brain from hex string.
    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let brain = serde_json::from_slice(&bytes)?;
        Ok(brain)
    }
}

impl Genotype {
    /// Serialize genotype to hex string.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    /// Deserialize genotype from hex string.
    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let genotype = serde_json::from_slice(&bytes)?;
        Ok(genotype)
    }
}
