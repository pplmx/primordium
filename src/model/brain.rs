use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
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

        let mut innov = 0;
        let mut connections = Vec::new();
        for i in 0..29 {
            for h in 41..47 {
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
        for h in 41..47 {
            for o in 29..41 {
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
            next_node_id: 47,
            learning_rate: 0.0,
            weight_deltas: HashMap::new(),
        }
    }

    pub fn forward(&self, inputs: [f32; 23], last_hidden: [f32; 6]) -> ([f32; 12], [f32; 6]) {
        let (outputs, next_hidden, _) = self.forward_internal(inputs, last_hidden);
        (outputs, next_hidden)
    }

    pub fn forward_internal(
        &self,
        inputs: [f32; 23],
        last_hidden: [f32; 6],
    ) -> ([f32; 12], [f32; 6], HashMap<usize, f32>) {
        let mut node_values: HashMap<usize, f32> = HashMap::new();

        for (i, &val) in inputs.iter().take(14).enumerate() {
            node_values.insert(i, val);
        }
        for (i, &val) in last_hidden.iter().enumerate() {
            node_values.insert(i + 14, val);
        }
        node_values.insert(20, inputs[14]);
        node_values.insert(21, inputs[15]);
        node_values.insert(22, inputs[16]);
        node_values.insert(23, inputs[17]);
        node_values.insert(24, inputs[18]);
        node_values.insert(25, inputs[19]);
        node_values.insert(26, inputs[20]);
        node_values.insert(27, inputs[21]);
        node_values.insert(28, inputs[22]);

        let mut new_values = node_values.clone();
        for conn in &self.connections {
            if !conn.enabled {
                continue;
            }
            let val = *node_values.get(&conn.from).unwrap_or(&0.0);
            let entry = new_values.entry(conn.to).or_insert(0.0);
            *entry += val * conn.weight;
        }

        let mut outputs = [0.0; 12];
        for node in &self.nodes {
            if let Some(val) = new_values.get_mut(&node.id) {
                *val = val.tanh();
            }
        }

        for (i, output) in outputs.iter_mut().enumerate() {
            *output = *new_values.get(&(i + 29)).unwrap_or(&0.0);
        }

        let mut next_hidden = [0.0; 6];
        for (i, val) in next_hidden.iter_mut().enumerate() {
            *val = *new_values.get(&(i + 41)).unwrap_or(&0.0);
        }

        (outputs, next_hidden, new_values)
    }

    pub fn learn(&mut self, inputs: [f32; 23], last_hidden: [f32; 6], reinforcement: f32) {
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

            let entry = self.weight_deltas.entry(conn.innovation).or_insert(0.0);
            *entry = (*entry * 0.9) + delta.abs();
        }
    }

    pub fn mutate_with_config(
        &mut self,
        config: &crate::model::config::AppConfig,
        specialization: Option<crate::model::state::entity::Specialization>,
    ) {
        let mut rng = rand::thread_rng();
        let rate = config.evolution.mutation_rate;
        let amount = config.evolution.mutation_amount;

        for conn in &mut self.connections {
            if rng.gen::<f32>() < rate {
                let mut mut_amount = amount;

                // Phase 62: Functional Neural Modules (Protected weight sets)
                if let Some(spec) = specialization {
                    use crate::model::state::entity::Specialization;
                    let is_protected = match spec {
                        Specialization::Soldier => {
                            // Soldiers protect weights related to Aggression (output 32)
                            // ID 29+3 = 32
                            conn.to == 32
                        }
                        Specialization::Engineer => {
                            // Engineers protect weights related to Dig/Build (outputs 38, 39)
                            // ID 29+9=38, 29+10=39
                            conn.to == 38 || conn.to == 39
                        }
                        _ => false,
                    };
                    if is_protected {
                        mut_amount *= 0.1; // 90% mutation resistance
                    }
                }

                conn.weight += rng.gen_range(-mut_amount..mut_amount);
                conn.weight = conn.weight.clamp(-5.0, 5.0);
            }
        }

        for conn in &mut self.connections {
            if rng.gen::<f32>() < config.evolution.mutation_rate {
                conn.weight += rng
                    .gen_range(-config.evolution.mutation_amount..config.evolution.mutation_amount);
                conn.weight = conn.weight.clamp(-2.0, 2.0);
            }
        }

        let topo_rate = config.evolution.mutation_rate * 0.1;

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

    pub fn distance(&self, other: &Brain) -> f32 {
        self.genotype_distance(other)
    }

    pub fn crossover(&self, other: &Brain) -> Brain {
        crate::model::systems::intel::crossover_brains(self, other)
    }

    pub fn remodel_for_adult(&mut self) {
        let mut rng = rand::thread_rng();
        let adult_outputs = [34, 35, 36];
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
                self.connections.push(Connection {
                    from,
                    to: out_id,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation: self.connections.len() + 1_000_000,
                });
            }
        }
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

    #[test]
    fn test_brain_new_random_has_correct_dimensions() {
        let brain = Brain::new_random();
        assert!(brain.nodes.len() >= 45);
        let inputs = brain
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Input)
            .count();
        let outputs = brain
            .nodes
            .iter()
            .filter(|n| n.node_type == NodeType::Output)
            .count();
        assert_eq!(inputs, 29);
        assert_eq!(outputs, 12);
    }

    #[test]
    fn test_brain_learning_strengthens_connections() {
        let mut brain = Brain::new_random();
        brain.learning_rate = 0.5;
        let inputs = [1.0; 23];
        let hidden = [1.0; 6];
        let initial_weights: Vec<f32> = brain.connections.iter().map(|c| c.weight).collect();

        brain.learn(inputs, hidden, 1.0);

        let mut changed = false;
        for (i, conn) in brain.connections.iter().enumerate() {
            if conn.enabled && (conn.weight - initial_weights[i]).abs() > 1e-6 {
                changed = true;
                break;
            }
        }
        assert!(changed, "At least one weight should have changed");
    }
}
