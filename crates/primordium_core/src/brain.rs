pub use primordium_data::{Brain, Connection, Genotype, Node, NodeType};
use rand::Rng;
use std::collections::HashMap;

pub trait BrainLogic {
    fn new_random() -> Self;
    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self;
    fn forward(&self, inputs: [f32; 29], last_hidden: [f32; 6]) -> ([f32; 12], [f32; 6]);
    fn forward_internal(
        &self,
        inputs: [f32; 29],
        last_hidden: [f32; 6],
    ) -> ([f32; 12], [f32; 6], primordium_data::Activations);
    fn learn(&mut self, inputs: [f32; 29], last_hidden: [f32; 6], reinforcement: f32);
    fn mutate_with_config<R: Rng>(
        &mut self,
        config: &crate::config::AppConfig,
        specialization: Option<primordium_data::Specialization>,
        rng: &mut R,
    );
    fn genotype_distance(&self, other: &Brain) -> f32;
    fn distance(&self, other: &Brain) -> f32;
    fn crossover_with_rng<R: Rng>(&self, other: &Brain, rng: &mut R) -> Brain;
    fn crossover(&self, other: &Brain) -> Brain;
    fn remodel_for_adult_with_rng<R: Rng>(&mut self, rng: &mut R);
}

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

pub fn create_brain_random_with_rng<R: Rng>(rng: &mut R) -> Brain {
    let mut nodes = Vec::new();

    let input_labels = [
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

    for (i, label) in input_labels.iter().enumerate() {
        nodes.push(Node {
            id: i,
            node_type: NodeType::Input,
            label: Some(label.to_string()),
        });
    }
    let output_labels = [
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
    for (i, label) in output_labels.iter().enumerate() {
        nodes.push(Node {
            id: i + 29,
            node_type: NodeType::Output,
            label: Some(label.to_string()),
        });
    }
    for i in 41..47 {
        nodes.push(Node {
            id: i,
            node_type: NodeType::Hidden,
            label: None,
        });
    }

    let mut connections = Vec::new();
    for i in 0..29 {
        for h in 41..47 {
            connections.push(Connection {
                from: i,
                to: h,
                weight: rng.gen_range(-1.0..1.0),
                enabled: true,
                innovation: get_innovation_id(i, h),
            });
        }
    }
    for h in 41..47 {
        for o in 29..41 {
            connections.push(Connection {
                from: h,
                to: o,
                weight: rng.gen_range(-1.0..1.0),
                enabled: true,
                innovation: get_innovation_id(h, o),
            });
        }
    }

    Brain {
        nodes,
        connections,
        next_node_id: 47,
        learning_rate: 0.0,
        weight_deltas: HashMap::new(),
    }
}

/// Deterministic Innovation ID based on connection topology.
/// Essential for NEAT crossover to align matching genes across different organisms.
fn get_innovation_id(from: usize, to: usize) -> usize {
    let h = (from as u64) << 32 | (to as u64);
    let mut hash = 0xcbf29ce484222325u64;
    for byte in h.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    (hash as usize) & 0x7FFFFFFF
}

/// Deterministic Node ID for splitting a connection.
/// Ensures that splitting the same connection (A -> B) always produces a node with the same ID.
fn get_split_node_id(from: usize, to: usize) -> usize {
    let h = (from as u64) << 32 | (to as u64);
    let mut hash = 0x84222325cbf29ce4u64; // Different seed
    for byte in h.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    // Reserve low IDs for original inputs/outputs/hidden (0..1000)
    (hash as usize % 1_000_000) + 1000
}

pub fn create_genotype_random_with_rng<R: Rng>(rng: &mut R) -> Genotype {
    let brain = Brain::new_random_with_rng(rng);
    Genotype {
        brain,
        sensing_range: 10.0,
        max_speed: 1.0,
        max_energy: 100.0,
        lineage_id: uuid::Uuid::new_v4(),
        metabolic_niche: 0.5,
        trophic_potential: 0.5,
        reproductive_investment: 0.5,
        maturity_gene: 1.0,
        mate_preference: 0.5,
        pairing_bias: 0.5,
        specialization_bias: [0.33, 0.33, 0.34],
    }
}

impl BrainLogic for Brain {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }

    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        create_brain_random_with_rng(rng)
    }

    fn forward(&self, inputs: [f32; 29], last_hidden: [f32; 6]) -> ([f32; 12], [f32; 6]) {
        let (outputs, next_hidden, _) = self.forward_internal(inputs, last_hidden);
        (outputs, next_hidden)
    }

    fn forward_internal(
        &self,
        inputs: [f32; 29],
        _last_hidden: [f32; 6],
    ) -> ([f32; 12], [f32; 6], primordium_data::Activations) {
        let max_id = self.nodes.iter().map(|n| n.id).max().unwrap_or(63);
        let mut node_values = vec![0.0f32; (max_id + 1).max(64)];
        for (i, &val) in inputs.iter().enumerate() {
            node_values[i] = val;
        }

        for conn in &self.connections {
            if !conn.enabled {
                continue;
            }
            let val = node_values[conn.from];
            node_values[conn.to] += val * conn.weight;
        }

        let mut outputs = [0.0; 12];
        for node in &self.nodes {
            node_values[node.id] = node_values[node.id].tanh();
        }

        for (i, output) in outputs.iter_mut().enumerate() {
            *output = node_values[i + 29];
        }

        let mut next_hidden = [0.0; 6];
        for (i, val) in next_hidden.iter_mut().enumerate() {
            *val = node_values[i + 41];
        }

        (
            outputs,
            next_hidden,
            primordium_data::Activations(node_values),
        )
    }

    fn learn(&mut self, inputs: [f32; 29], last_hidden: [f32; 6], reinforcement: f32) {
        if self.learning_rate.abs() < 1e-4 || reinforcement.abs() < 1e-4 {
            return;
        }
        let (_, _, activations) = self.forward_internal(inputs, last_hidden);
        for conn in &mut self.connections {
            if !conn.enabled {
                continue;
            }
            let pre = activations.0[conn.from];
            let post = activations.0[conn.to];
            let delta = self.learning_rate * reinforcement * pre * post;
            conn.weight += delta;
            conn.weight = conn.weight.clamp(-5.0, 5.0);
            let entry = self.weight_deltas.entry(conn.innovation).or_insert(0.0);
            *entry = (*entry * 0.9) + delta.abs();
        }
    }

    fn mutate_with_config<R: Rng>(
        &mut self,
        config: &crate::config::AppConfig,
        specialization: Option<primordium_data::Specialization>,
        rng: &mut R,
    ) {
        let rate = config.evolution.mutation_rate;
        let amount = config.evolution.mutation_amount;

        for conn in &mut self.connections {
            if rng.gen::<f32>() < rate {
                let mut mut_amount = amount;

                if let Some(spec) = specialization {
                    use primordium_data::Specialization;
                    let is_protected = match spec {
                        Specialization::Soldier => conn.to == 32,
                        Specialization::Engineer => conn.to == 38 || conn.to == 39,
                        _ => false,
                    };
                    if is_protected {
                        mut_amount *= 0.1;
                    }
                }

                conn.weight += rng.gen_range(-mut_amount..mut_amount);
                conn.weight = conn.weight.clamp(-5.0, 5.0);
            }
        }

        let topo_rate = config.evolution.mutation_rate * 0.1;

        if rng.gen::<f32>() < topo_rate {
            let from_idx = rng.gen_range(0..self.nodes.len());
            let to_idx = rng.gen_range(0..self.nodes.len());
            let from = self.nodes[from_idx].id;
            let to_node = &self.nodes[to_idx];
            if !matches!(to_node.node_type, NodeType::Input) {
                let innovation = get_innovation_id(from, to_node.id);
                // Only add if it doesn't already exist
                if !self.connections.iter().any(|c| c.innovation == innovation) {
                    self.connections.push(Connection {
                        from,
                        to: to_node.id,
                        weight: rng.gen_range(-1.0..1.0),
                        enabled: true,
                        innovation,
                    });
                }
            }
        }

        if rng.gen::<f32>() < topo_rate * 0.5 && !self.connections.is_empty() {
            let idx = rng.gen_range(0..self.connections.len());
            if self.connections[idx].enabled {
                self.connections[idx].enabled = false;
                let from = self.connections[idx].from;
                let to = self.connections[idx].to;
                let new_id = get_split_node_id(from, to);

                // Check if node already exists in this brain
                if !self.nodes.iter().any(|n| n.id == new_id) {
                    self.nodes.push(Node {
                        id: new_id,
                        node_type: NodeType::Hidden,
                        label: None,
                    });
                }

                self.connections.push(Connection {
                    from,
                    to: new_id,
                    weight: 1.0,
                    enabled: true,
                    innovation: get_innovation_id(from, new_id),
                });
                self.connections.push(Connection {
                    from: new_id,
                    to,
                    weight: self.connections[idx].weight,
                    enabled: true,
                    innovation: get_innovation_id(new_id, to),
                });
            }
        }

        if rng.gen_bool(0.1) {
            self.connections
                .retain(|c| c.weight.abs() >= config.brain.pruning_threshold || !c.enabled);
        }
    }

    fn genotype_distance(&self, other: &Brain) -> f32 {
        let mut weight_diff = 0.0;
        let mut matching = 0;
        let mut map1 = HashMap::new();
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
        let mut child_nodes = self.nodes.clone();
        let mut child_connections = Vec::new();
        let mut map2 = HashMap::new();
        for c in &other.connections {
            map2.insert(c.innovation, c);
        }

        for c1 in &self.connections {
            if let Some(c2) = map2.get(&c1.innovation) {
                if rng.gen_bool(0.5) {
                    child_connections.push(c1.clone());
                } else {
                    child_connections.push((*c2).clone());
                }
            } else {
                child_connections.push(c1.clone());
            }
        }

        let mut existing_node_ids: std::collections::HashSet<usize> =
            child_nodes.iter().map(|n| n.id).collect();

        let other_node_map: HashMap<usize, &Node> = other.nodes.iter().map(|n| (n.id, n)).collect();

        for c in &child_connections {
            if !existing_node_ids.contains(&c.from) {
                if let Some(&n) = other_node_map.get(&c.from) {
                    child_nodes.push(n.clone());
                    existing_node_ids.insert(c.from);
                }
            }
            if !existing_node_ids.contains(&c.to) {
                if let Some(&n) = other_node_map.get(&c.to) {
                    child_nodes.push(n.clone());
                    existing_node_ids.insert(c.to);
                }
            }
        }

        Brain {
            nodes: child_nodes,
            connections: child_connections,
            next_node_id: self.next_node_id.max(other.next_node_id),
            learning_rate: if rng.gen_bool(0.5) {
                self.learning_rate
            } else {
                other.learning_rate
            },
            weight_deltas: HashMap::new(),
        }
    }

    fn crossover(&self, other: &Brain) -> Brain {
        let mut rng = rand::thread_rng();
        self.crossover_with_rng(other, &mut rng)
    }

    fn remodel_for_adult_with_rng<R: Rng>(&mut self, rng: &mut R) {
        let adult_outputs = [34, 35, 36, 37, 38, 39, 40];
        let hidden_nodes: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| matches!(n.node_type, NodeType::Hidden))
            .map(|n| n.id)
            .collect();

        if hidden_nodes.is_empty() {
            return;
        }

        for &out_id in &adult_outputs {
            let has_conn = self.connections.iter().any(|c| c.to == out_id && c.enabled);

            if !has_conn {
                let from = hidden_nodes[rng.gen_range(0..hidden_nodes.len())];
                let innovation = get_innovation_id(from, out_id);
                if !self.connections.iter().any(|c| c.innovation == innovation) {
                    self.connections.push(Connection {
                        from,
                        to: out_id,
                        weight: rng.gen_range(-1.0..1.0),
                        enabled: true,
                        innovation,
                    });
                }
            }
        }
        self.learning_rate = (self.learning_rate + 0.05).clamp(0.0, 0.5);
    }
}

impl GenotypeLogic for Genotype {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }

    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        let brain = Brain::new_random_with_rng(rng);
        Self {
            brain,
            sensing_range: 10.0,
            max_speed: 1.0,
            max_energy: 100.0,
            lineage_id: uuid::Uuid::new_v4(),
            metabolic_niche: 0.5,
            trophic_potential: 0.5,
            reproductive_investment: 0.5,
            maturity_gene: 1.0,
            mate_preference: 0.5,
            pairing_bias: 0.5,
            specialization_bias: [0.33, 0.33, 0.34],
        }
    }

    fn crossover_with_rng<R: Rng>(&self, other: &Self, rng: &mut R) -> Self {
        let brain = self.brain.crossover_with_rng(&other.brain, rng);

        let mut child_genotype = if rng.gen_bool(0.5) {
            self.clone()
        } else {
            other.clone()
        };
        child_genotype.brain = brain;
        child_genotype
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
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let genotype = serde_json::from_slice(&bytes)?;
        Ok(genotype)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_innovation_id_determinism() {
        let id1 = get_innovation_id(10, 20);
        let id2 = get_innovation_id(10, 20);
        let id3 = get_innovation_id(20, 10);
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_split_node_id_determinism() {
        let id1 = get_split_node_id(5, 15);
        let id2 = get_split_node_id(5, 15);
        let id3 = get_split_node_id(15, 5);
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
}
