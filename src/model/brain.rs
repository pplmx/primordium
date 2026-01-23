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
/// Topology (Default):
/// Inputs (0..15): 9 environmental + 6 recurrent
/// Outputs (15..23): MoveX, MoveY, Speed, Aggro, Share, Color, EmitA, EmitB
/// Hidden (23..29): Initial hidden nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub next_node_id: usize,
}

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut nodes = Vec::new();
        let mut connections = Vec::new();

        // 1. Create Inputs (0..15)
        for i in 0..15 {
            nodes.push(Node {
                id: i,
                node_type: NodeType::Input,
            });
        }
        // 2. Create Outputs (15..23)
        for i in 15..23 {
            nodes.push(Node {
                id: i,
                node_type: NodeType::Output,
            });
        }
        // 3. Create initial Hidden layer (23..29)
        for i in 23..29 {
            nodes.push(Node {
                id: i,
                node_type: NodeType::Hidden,
            });
        }

        let mut innov = 0;
        // 4. Initial connections: Input -> Hidden
        for i in 0..15 {
            for h in 23..29 {
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
        for h in 23..29 {
            for o in 15..23 {
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
            next_node_id: 29,
        }
    }

    /// Forward pass through the graph.
    pub fn forward(&self, inputs: [f32; 9], last_hidden: [f32; 6]) -> ([f32; 8], [f32; 6]) {
        let mut node_values: HashMap<usize, f32> = HashMap::new();

        // 1. Load inputs (9 sensors)
        for (i, &val) in inputs.iter().enumerate() {
            node_values.insert(i, val);
        }
        // 2. Load memory (6 memory inputs)
        for (i, &val) in last_hidden.iter().enumerate() {
            node_values.insert(i + 9, val);
        }

        let mut new_values = node_values.clone();
        for conn in &self.connections {
            if !conn.enabled {
                continue;
            }
            let val = *node_values.get(&conn.from).unwrap_or(&0.0);
            let entry = new_values.entry(conn.to).or_insert(0.0);
            *entry += val * conn.weight;
        }

        let mut outputs = [0.0; 8];
        for node in &self.nodes {
            if let Some(val) = new_values.get_mut(&node.id) {
                *val = val.tanh();
            }
        }

        for (i, output) in outputs.iter_mut().enumerate() {
            *output = *new_values.get(&(i + 15)).unwrap_or(&0.0);
        }

        // 5. Extract new "hidden" state for memory (using IDs 23..29 as canonical hidden)
        let mut next_hidden = [0.0; 6];
        for (i, val) in next_hidden.iter_mut().enumerate() {
            *val = *new_values.get(&(i + 23)).unwrap_or(&0.0);
        }

        (outputs, next_hidden)
    }

    pub fn mutate_with_config(&mut self, config: &crate::model::config::EvolutionConfig) {
        let mut rng = rand::thread_rng();

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
        let disjoint = (self.connections.len() + other.connections.len()) - (2 * matching);
        (weight_diff / matching.max(1) as f32) + (disjoint as f32 * 0.5)
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
        assert_eq!(brain.nodes.len(), 29);
        // 15 inputs, 8 outputs, 6 initial hidden
        // In->Hid: 15*6 = 90
        // Hid->Out: 6*8 = 48
        // Total = 138
        assert_eq!(brain.connections.len(), 138);
    }

    #[test]
    fn test_brain_forward_is_deterministic() {
        let brain = Brain::new_random();
        let inputs = [0.5, -0.5, 0.3, 0.0, 0.1, 0.2, 0.1, 0.0, 0.0];
        let last_hidden = [0.0; 6];
        let (output1, _) = intel::brain_forward(&brain, inputs, last_hidden);
        let (output2, _) = intel::brain_forward(&brain, inputs, last_hidden);
        assert_eq!(output1, output2);
    }

    #[test]
    fn test_brain_mutate_topology_adds_genes() {
        let mut brain = Brain::new_random();
        let initial_conns = brain.connections.len();
        let initial_nodes = brain.nodes.len();
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 1.0,
            mutation_amount: 0.5,
            drift_rate: 0.0,
            drift_amount: 0.0,
            speciation_rate: 0.0,
        };
        for _ in 0..100 {
            brain.mutate_with_config(&config);
        }
        assert!(brain.connections.len() > initial_conns || brain.nodes.len() > initial_nodes);
    }

    #[test]
    fn test_brain_hex_roundtrip() {
        let original = Brain::new_random();
        let hex = original.to_hex();
        let restored = Brain::from_hex(&hex).expect("Should decode successfully");
        assert_eq!(original.nodes.len(), restored.nodes.len());
        assert_eq!(original.connections.len(), restored.connections.len());
    }
}
