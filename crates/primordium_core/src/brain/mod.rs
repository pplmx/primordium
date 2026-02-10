pub mod crossover;
pub mod forward;
pub mod mutation;
pub mod topology;

pub use primordium_data::{Brain, Connection, Genotype, Node, NodeType, Specialization};
use rand::Rng;

pub use topology::{create_brain_random_with_rng, create_genotype_random_with_rng};

/// Trait defining the core logic for neural network brains in Primordium.
pub trait BrainLogic {
    fn new_random() -> Self;
    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self;

    #[must_use]
    fn forward(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]);

    #[must_use]
    fn forward_internal(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
        activations: &mut primordium_data::Activations,
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]);

    fn learn(&mut self, activations: &primordium_data::Activations, reinforcement: f32);

    fn mutate_with_config<R: Rng>(
        &mut self,
        config: &crate::config::AppConfig,
        specialization: Option<Specialization>,
        rng: &mut R,
    );

    fn genotype_distance(&self, other: &Brain) -> f32;
    fn distance(&self, other: &Brain) -> f32;
    fn crossover_with_rng<R: Rng>(&self, other: &Brain, rng: &mut R) -> Brain;
    fn crossover(&self, other: &Brain) -> Brain;
    fn remodel_for_adult_with_rng<R: Rng>(&mut self, rng: &mut R);
    fn initialize_node_idx_map(&mut self);
}

/// Trait defining the genetic interface for organism genotypes.
pub trait GenotypeLogic {
    fn new_random() -> Self;
    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self;
    fn crossover_with_rng<R: Rng>(&self, other: &Genotype, rng: &mut R) -> Genotype;
    fn crossover(&self, other: &Genotype) -> Genotype;
    fn distance(&self, other: &Genotype) -> f32;
    fn relatedness(&self, other: &Genotype) -> f32;
    fn to_hex(&self) -> String;
    fn from_hex(hex_str: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

pub const INPUT_LABELS: [&str; 29] = [
    "FoodDX",
    "FoodDY",
    "Energy",
    "Density",
    "Phero",
    "Tribe",
    "KX",
    "KY",
    "SA",
    "SB",
    "WL",
    "AG",
    "NT",
    "TP",
    "Mem0",
    "Mem1",
    "Mem2",
    "Mem3",
    "Mem4",
    "Mem5",
    "Hear",
    "PartnerEnergy",
    "BuildPressure",
    "DigPressure",
    "SharedGoal",
    "SharedThreat",
    "LineagePop",
    "LineageEnergy",
    "Overmind",
];

pub const OUTPUT_LABELS: [&str; 12] = [
    "MoveX",
    "MoveY",
    "Speed",
    "Aggro",
    "Share",
    "Color",
    "EmitA",
    "EmitB",
    "Bond",
    "Dig",
    "Build",
    "OvermindEmit",
];

pub const BRAIN_INPUTS: usize = INPUT_LABELS.len();
pub const BRAIN_OUTPUTS: usize = OUTPUT_LABELS.len();
pub const BRAIN_MEMORY: usize = 6;
pub const BRAIN_HIDDEN_START: usize = BRAIN_INPUTS + BRAIN_OUTPUTS;
pub const BRAIN_HIDDEN_END: usize = BRAIN_HIDDEN_START + 6;

impl BrainLogic for Brain {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }

    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        topology::create_brain_random_with_rng(rng)
    }

    fn forward(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
        forward::forward(self, inputs, last_hidden)
    }

    fn forward_internal(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
        activations: &mut primordium_data::Activations,
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
        forward::forward_internal(self, inputs, last_hidden, activations)
    }

    fn learn(&mut self, activations: &primordium_data::Activations, reinforcement: f32) {
        forward::learn(self, activations, reinforcement)
    }

    fn mutate_with_config<R: Rng>(
        &mut self,
        config: &crate::config::AppConfig,
        specialization: Option<Specialization>,
        rng: &mut R,
    ) {
        mutation::mutate_with_config(self, config, specialization, rng)
    }

    fn genotype_distance(&self, other: &Brain) -> f32 {
        let mut weight_diff = 0.0;
        let mut matching = 0;
        let mut map1 = std::collections::HashMap::new();
        for c in &self.connections {
            map1.insert(c.innovation, c.weight);
        }
        for c in &other.connections {
            if let Some(w1) = map1.get(&c.innovation) {
                weight_diff += (w1 - c.weight).abs();
                matching += 1;
            }
        }

        let lr_diff = (self.learning_rate - other.learning_rate).abs();
        let disjoint =
            (self.connections.len() + other.connections.len()).saturating_sub(2 * matching);
        (weight_diff / matching.max(1) as f32) + (disjoint as f32 * 0.5) + lr_diff
    }

    fn distance(&self, other: &Brain) -> f32 {
        self.genotype_distance(other)
    }

    fn crossover_with_rng<R: Rng>(&self, other: &Brain, rng: &mut R) -> Brain {
        crossover::brain_crossover_with_rng(self, other, rng)
    }

    fn crossover(&self, other: &Brain) -> Brain {
        let mut rng = rand::thread_rng();
        self.crossover_with_rng(other, &mut rng)
    }

    fn remodel_for_adult_with_rng<R: Rng>(&mut self, rng: &mut R) {
        mutation::remodel_for_adult_with_rng(self, rng)
    }

    fn initialize_node_idx_map(&mut self) {
        topology::initialize_node_idx_map(self)
    }
}

impl GenotypeLogic for Genotype {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }

    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        topology::create_genotype_random_with_rng(rng)
    }

    fn crossover_with_rng<R: Rng>(&self, other: &Self, rng: &mut R) -> Self {
        crossover::genotype_crossover_with_rng(self, other, rng)
    }

    fn crossover(&self, other: &Self) -> Self {
        let mut rng = rand::thread_rng();
        self.crossover_with_rng(other, &mut rng)
    }

    fn distance(&self, other: &Self) -> f32 {
        self.brain.distance(&other.brain)
    }

    fn relatedness(&self, other: &Self) -> f32 {
        let dist = self.distance(other);
        (1.0 - (dist / 10.0)).clamp(0.0, 1.0)
    }

    fn to_hex(&self) -> String {
        match serde_json::to_vec(self) {
            Ok(bytes) => hex::encode(bytes),
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize genotype to JSON");
                String::new()
            }
        }
    }

    fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes =
            hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Invalid hex encoding: {}", e))?;

        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Empty hex string"));
        }

        let genotype = serde_json::from_slice(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize genotype: {}", e))?;
        Ok(genotype)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primordium_data::NodeType;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_innovation_id_determinism() {
        let id1 = topology::get_innovation_id(10, 20);
        let id2 = topology::get_innovation_id(10, 20);
        let id3 = topology::get_innovation_id(20, 10);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_split_node_id_determinism() {
        let id1 = topology::get_split_node_id(5, 15);
        let id2 = topology::get_split_node_id(5, 15);
        let id3 = topology::get_split_node_id(15, 5);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert!(id1 >= 1000);
    }

    #[test]
    fn test_crossover_brains_preserves_innovation_alignment() {
        let mut rng = ChaCha8Rng::seed_from_u64(456);
        let p1 = Brain::new_random_with_rng(&mut rng);
        let mut rng = ChaCha8Rng::seed_from_u64(789);
        let p2 = Brain::new_random_with_rng(&mut rng);

        let mut rng = ChaCha8Rng::seed_from_u64(111);
        let child = p1.crossover_with_rng(&p2, &mut rng);

        assert!(!child.nodes.is_empty());

        for conn in &child.connections {
            assert!(
                child.nodes.iter().any(|n| n.id == conn.from),
                "Connection from {} has no source node",
                conn.from
            );
            assert!(
                child.nodes.iter().any(|n| n.id == conn.to),
                "Connection to {} has no target node",
                conn.to
            );
        }
    }

    #[test]
    fn test_brain_new_random_creates_valid_brain() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let brain = Brain::new_random_with_rng(&mut rng);

        assert!(!brain.nodes.is_empty(), "Brain should have nodes");
        assert!(
            !brain.connections.is_empty(),
            "Brain should have connections"
        );

        let input_count = brain
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Input)
            .count();
        let output_count = brain
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Output)
            .count();

        assert_eq!(
            input_count, BRAIN_INPUTS,
            "Should have correct number of inputs"
        );
        assert_eq!(
            output_count, BRAIN_OUTPUTS,
            "Should have correct number of outputs"
        );
    }

    #[test]
    fn test_brain_forward_produces_valid_outputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let brain = Brain::new_random_with_rng(&mut rng);

        let inputs = [0.5; BRAIN_INPUTS];
        let hidden = [0.0; BRAIN_MEMORY];
        let (outputs, next_hidden) = brain.forward(inputs, hidden);

        assert_eq!(
            outputs.len(),
            BRAIN_OUTPUTS,
            "Should produce correct number of outputs"
        );
        assert_eq!(
            next_hidden.len(),
            BRAIN_MEMORY,
            "Should produce correct number of hidden states"
        );

        for (i, &val) in outputs.iter().enumerate() {
            assert!(!val.is_nan(), "Output {} should not be NaN", i);
            assert!(!val.is_infinite(), "Output {} should not be infinite", i);
        }
    }

    #[test]
    fn test_brain_forward_deterministic() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let brain1 = Brain::new_random_with_rng(&mut rng);
        let brain2 = brain1.clone();

        let inputs = [0.5; BRAIN_INPUTS];
        let hidden = [0.0; BRAIN_MEMORY];
        let (outputs1, _) = brain1.forward(inputs, hidden);
        let (outputs2, _) = brain2.forward(inputs, hidden);

        assert_eq!(outputs1, outputs2, "Forward pass should be deterministic");
    }

    #[test]
    fn test_brain_to_hex_roundtrip() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let brain = Brain::new_random_with_rng(&mut rng);

        let hex = brain.to_hex();
        let restored = Brain::from_hex(&hex).expect("Should deserialize successfully");

        assert_eq!(
            brain.nodes.len(),
            restored.nodes.len(),
            "Node count should match"
        );
    }
}
