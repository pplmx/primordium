use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of nodes in the neural network.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Input,
    Hidden,
    Output,
}

/// A single node (neuron) in the dynamic brain.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub node_type: NodeType,
    pub label: Option<String>,
}

/// A connection between two nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from: usize,
    pub to: usize,
    pub weight: f32,
    pub enabled: bool,
    pub innovation: usize,
}

/// Dynamic neural network brain (NEAT-lite).
///
/// Topology (Phase 52 Terraforming):
/// Inputs (0..22): 1 sensor inputs (14 environmental + 6 recurrent + 1 hearing + 1 partner)
/// Outputs (22..33): MoveX, MoveY, Speed, Aggro, Share, Color, EmitA, EmitB, Bond, Dig, Build (11 nodes)
/// Hidden (33..39): Initial hidden nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub next_node_id: usize,
    pub learning_rate: f32,
}

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        // 1. Create Inputs (0..22)
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
        ];

        for (i, label) in input_labels.iter().enumerate() {
            nodes.push(Node {
                id: i,
                node_type: NodeType::Input,
                label: Some(label.to_string()),
            });
        }
        // 2. Create Outputs (22..33)
        let output_labels = [
            "MoveX", "MoveY", "Speed", "Aggro", "Share", "Color", "EmitA", "EmitB", "Bond", "Dig",
            "Build",
        ];
        for (i, label) in output_labels.iter().enumerate() {
            nodes.push(Node {
                id: i + 22,
                node_type: NodeType::Output,
                label: Some(label.to_string()),
            });
        }
        // 3. Create initial Hidden layer (33..39)
        for i in 33..39 {
            nodes.push(Node {
                id: i,
                node_type: NodeType::Hidden,
                label: None,
            });
        }

        let mut innov = 0;
        // 4. Initial connections: Input -> Hidden
        for i in 0..22 {
            for h in 33..39 {
                connections.push(Connection {
                    from: i,
                    to: h,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation: innov,
                });
                innov += 1;
            }
        }
        // 5. Initial connections: Hidden -> Output
        for h in 33..39 {
            for o in 22..33 {
                connections.push(Connection {
                    from: h,
                    to: o,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation: innov,
                });
                innov += 1;
            }
        }

        Self {
            nodes,
            connections,
            next_node_id: 39,
            learning_rate: 0.0, // Default to 0, evolves later
        }
    }

    /// Forward pass through the graph.
    pub fn forward(&self, inputs: [f32; 16], last_hidden: [f32; 6]) -> ([f32; 11], [f32; 6]) {
        let (outputs, next_hidden, _) = self.forward_internal(inputs, last_hidden);
        (outputs, next_hidden)
    }

    /// Internal forward pass that also returns activations for learning.
    pub fn forward_internal(
        &self,
        inputs: [f32; 16],
        last_hidden: [f32; 6],
    ) -> ([f32; 11], [f32; 6], HashMap<usize, f32>) {
        let mut node_values: HashMap<usize, f32> = HashMap::new();

        // 1. Load inputs (14 sensors)
        for (i, &val) in inputs.iter().take(14).enumerate() {
            node_values.insert(i, val);
        }
        // 2. Load memory (6 memory inputs) - Mapped to 14..20
        for (i, &val) in last_hidden.iter().enumerate() {
            node_values.insert(i + 14, val);
        }
        // 3. Load Hearing input (Index 20)
        node_values.insert(20, inputs[14]);
        // 4. Load Partner Energy input (Index 21)
        node_values.insert(21, inputs[15]);

        let mut new_values = node_values.clone();
        for conn in &self.connections {
            if !conn.enabled {
                continue;
            }
            let val = *node_values.get(&conn.from).unwrap_or(&0.0);
            let entry = new_values.entry(conn.to).or_insert(0.0);
            *entry += val * conn.weight;
        }

        let mut outputs = [0.0; 11];
        for node in &self.nodes {
            if let Some(val) = new_values.get_mut(&node.id) {
                *val = val.tanh();
            }
        }

        // Outputs range: 22..33
        for (i, output) in outputs.iter_mut().enumerate() {
            *output = *new_values.get(&(i + 22)).unwrap_or(&0.0);
        }

        // Hidden range: 33..39
        let mut next_hidden = [0.0; 6];
        for (i, val) in next_hidden.iter_mut().enumerate() {
            *val = *new_values.get(&(i + 33)).unwrap_or(&0.0);
        }

        (outputs, next_hidden, new_values)
    }

    /// Hebbian Learning: Update weights based on activation correlation and reinforcement.
    pub fn learn(&mut self, inputs: [f32; 16], last_hidden: [f32; 6], reinforcement: f32) {
        if self.learning_rate.abs() < 1e-4 || reinforcement.abs() < 1e-4 {
            return;
        }

        let (_, _, activations) = self.forward_internal(inputs, last_hidden);

        for conn in &mut self.connections {
            if !conn.enabled {
                continue;
            }

            let pre = *activations.get(&conn.from).unwrap_or(&0.0);
            let post = *activations.get(&conn.to).unwrap_or(&0.0);

            let delta = self.learning_rate * reinforcement * pre * post;
            conn.weight += delta;
            conn.weight = conn.weight.clamp(-5.0, 5.0);
        }
    }

    pub fn mutate_with_config(&mut self, config: &crate::model::config::EvolutionConfig) {
        let mut rng = rand::thread_rng();

        if rng.gen::<f32>() < config.mutation_rate {
            self.learning_rate +=
                rng.gen_range(-config.mutation_amount..config.mutation_amount) * 0.1;
            self.learning_rate = self.learning_rate.clamp(0.0, 0.5);
        }

        for conn in &mut self.connections {
            if rng.gen::<f32>() < config.mutation_rate {
                conn.weight += rng.gen_range(-config.mutation_amount..config.mutation_amount);
                conn.weight = conn.weight.clamp(-2.0, 2.0);
            }
        }

        let topo_rate = config.mutation_rate * 0.1;

        if rng.gen::<f32>() < topo_rate {
            let from = self.nodes[rng.gen_range(0..self.nodes.len())].id;
            let to = self.nodes[rng.gen_range(0..self.nodes.len())].id;
            if !matches!(
                self.nodes.iter().find(|n| n.id == to).unwrap().node_type,
                NodeType::Input
            ) {
                self.connections.push(Connection {
                    from,
                    to,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation: self.connections.len(),
                });
            }
        }

        if rng.gen::<f32>() < topo_rate * 0.5 && !self.connections.is_empty() {
            let idx = rng.gen_range(0..self.connections.len());
            if self.connections[idx].enabled {
                self.connections[idx].enabled = false;
                let from = self.connections[idx].from;
                let to = self.connections[idx].to;
                let new_id = self.next_node_id;
                self.next_node_id += 1;
                self.nodes.push(Node {
                    id: new_id,
                    node_type: NodeType::Hidden,
                    label: None,
                });
                self.connections.push(Connection {
                    from,
                    to: new_id,
                    weight: 1.0,
                    enabled: true,
                    innovation: self.connections.len(),
                });
                self.connections.push(Connection {
                    from: new_id,
                    to,
                    weight: self.connections[idx].weight,
                    enabled: true,
                    innovation: self.connections.len(),
                });
            }
        }
    }

    pub fn genotype_distance(&self, other: &Brain) -> f32 {
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
        let disjoint = (self.connections.len() + other.connections.len()) - (2 * matching);
        (weight_diff / matching.max(1) as f32) + (disjoint as f32 * 0.5) + lr_diff
    }

    /// Structured remodeling during metamorphosis: ensure adult-stage outputs are connected.
    pub fn remodel_for_adult(&mut self) {
        let mut rng = rand::thread_rng();
        // Adult outputs: Bond (30), Dig (31), Build (32)
        let adult_outputs = [30, 31, 32];
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
            // Check if this output has any incoming connections
            let has_conn = self.connections.iter().any(|c| c.to == out_id && c.enabled);

            if !has_conn {
                // Connect a random hidden node to this output
                let from = hidden_nodes[rng.gen_range(0..hidden_nodes.len())];
                self.connections.push(Connection {
                    from,
                    to: out_id,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation: self.connections.len() + 1000, // Offset to avoid collisions
                });
            }
        }

        // Boost learning rate slightly for adult phase adaptation
        self.learning_rate = (self.learning_rate + 0.05).clamp(0.0, 0.5);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::systems::intel;

    #[test]
    fn test_brain_new_random_has_correct_dimensions() {
        let brain = Brain::new_random();
        assert_eq!(brain.nodes.len(), 39);
        // 22 inputs, 11 outputs, 6 hidden
        // 22*6 = 132
        // 6*11 = 66
        // Total = 198
        assert_eq!(brain.connections.len(), 198);
    }

    #[test]
    fn test_brain_forward_is_deterministic() {
        let brain = Brain::new_random();
        let inputs = [0.0; 16];
        let last_hidden = [0.0; 6];
        let (output1, _) = intel::brain_forward(&brain, inputs, last_hidden);
        let (output2, _) = intel::brain_forward(&brain, inputs, last_hidden);
        assert_eq!(output1, output2);
    }

    #[test]
    fn test_brain_learning_strengthens_connections() {
        let mut brain = Brain::new_random();
        brain.learning_rate = 0.5;
        let inputs = [1.0; 16];
        let last_hidden = [0.0; 6];
        let conn_idx = brain.connections.iter().position(|c| c.enabled).unwrap();
        let old_weight = brain.connections[conn_idx].weight;
        brain.learn(inputs, last_hidden, 1.0);
        let new_weight = brain.connections[conn_idx].weight;
        assert_ne!(old_weight, new_weight);
    }
}
