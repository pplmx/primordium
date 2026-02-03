pub use primordium_data::{Brain, Connection, Genotype, Node, NodeType};
use rand::Rng;
use std::collections::HashMap;

pub trait BrainLogic {
    fn new_random() -> Self;
    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self;
    fn forward(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]);
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
        specialization: Option<primordium_data::Specialization>,
        rng: &mut R,
    );
    fn genotype_distance(&self, other: &Brain) -> f32;
    fn distance(&self, other: &Brain) -> f32;
    fn crossover_with_rng<R: Rng>(&self, other: &Brain, rng: &mut R) -> Brain;
    fn crossover(&self, other: &Brain) -> Brain;
    fn remodel_for_adult_with_rng<R: Rng>(&mut self, rng: &mut R);
    fn initialize_node_idx_map(&mut self);
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

const INPUT_LABELS: [&str; 29] = [
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

const OUTPUT_LABELS: [&str; 12] = [
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

pub fn create_brain_random_with_rng<R: Rng>(rng: &mut R) -> Brain {
    let mut nodes = Vec::new();

    for (i, label) in INPUT_LABELS.iter().enumerate() {
        nodes.push(Node {
            id: i,
            node_type: NodeType::Input,
            label: Some(label.to_string()),
        });
    }

    for (i, label) in OUTPUT_LABELS.iter().enumerate() {
        nodes.push(Node {
            id: i + BRAIN_INPUTS,
            node_type: NodeType::Output,
            label: Some(label.to_string()),
        });
    }
    for i in BRAIN_HIDDEN_START..BRAIN_HIDDEN_END {
        nodes.push(Node {
            id: i,
            node_type: NodeType::Hidden,
            label: None,
        });
    }

    let mut connections = Vec::new();
    for i in 0..BRAIN_INPUTS {
        for h in BRAIN_HIDDEN_START..BRAIN_HIDDEN_END {
            connections.push(Connection {
                from: i,
                to: h,
                weight: rng.gen_range(-1.0..1.0),
                enabled: true,
                innovation: get_innovation_id(i, h),
            });
        }
    }
    for h in BRAIN_HIDDEN_START..BRAIN_HIDDEN_END {
        for o in BRAIN_INPUTS..BRAIN_HIDDEN_START {
            connections.push(Connection {
                from: h,
                to: o,
                weight: rng.gen_range(-1.0..1.0),
                enabled: true,
                innovation: get_innovation_id(h, o),
            });
        }
    }

    let mut brain = Brain {
        nodes,
        connections,
        next_node_id: BRAIN_HIDDEN_END,
        learning_rate: 0.0,
        weight_deltas: HashMap::new(),
        node_idx_map: HashMap::new(),
        topological_order: Vec::new(),
        forward_connections: Vec::new(),
        recurrent_connections: Vec::new(),
        incoming_forward_connections: HashMap::new(),
    };
    brain.initialize_node_idx_map();
    brain
}

fn get_innovation_id(from: usize, to: usize) -> usize {
    let h = (from as u64) << 32 | (to as u64);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in h.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3_u64);
    }
    (hash as usize) & 0x7FFF_FFFF
}

fn get_split_node_id(from: usize, to: usize) -> usize {
    let h = (from as u64) << 32 | (to as u64);
    let mut hash = 0x8422_2325_cbf2_9ce4_u64;
    for byte in h.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3_u64);
    }
    (hash as usize % 1_000_000) + 1000
}

pub fn create_genotype_random_with_rng<R: Rng>(rng: &mut R) -> Genotype {
    let brain = Brain::new_random_with_rng(rng);
    let lineage_id = uuid::Uuid::from_u128(rng.gen::<u128>());
    Genotype {
        brain,
        sensing_range: 10.0,
        max_speed: 1.0,
        max_energy: 100.0,
        lineage_id,
        metabolic_niche: 0.5,
        trophic_potential: 0.5,
        reproductive_investment: 0.5,
        maturity_gene: 1.0,
        mate_preference: 0.5,
        pairing_bias: 0.5,
        specialization_bias: [0.33, 0.33, 0.34],
        regulatory_rules: Vec::new(),
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

    fn forward(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        last_hidden: [f32; BRAIN_MEMORY],
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
        let mut activations = primordium_data::Activations::default();
        self.forward_internal(inputs, last_hidden, &mut activations)
    }

    fn forward_internal(
        &self,
        inputs: [f32; BRAIN_INPUTS],
        _last_hidden: [f32; BRAIN_MEMORY],
        activations: &mut primordium_data::Activations,
    ) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
        let node_count = self.nodes.len();
        if self.node_idx_map.is_empty() {
            let outputs = [0.0; BRAIN_OUTPUTS];
            let next_hidden = [0.0; BRAIN_MEMORY];
            return (outputs, next_hidden);
        }

        std::mem::swap(&mut activations.0, &mut activations.1);
        let prev_values = &activations.1;
        let node_values = &mut activations.0;
        node_values.clear();
        node_values.resize(node_count, 0.0);

        for &conn_idx in &self.recurrent_connections {
            let conn = &self.connections[conn_idx];
            if let (Some(&f_idx), Some(&t_idx)) = (
                self.node_idx_map.get(&conn.from),
                self.node_idx_map.get(&conn.to),
            ) {
                if f_idx < prev_values.len() {
                    node_values[t_idx] += prev_values[f_idx] * conn.weight;
                }
            }
        }

        for &node_id in &self.topological_order {
            let node_idx = *self
                .node_idx_map
                .get(&node_id)
                .expect("Topology error: node_id in order but not in map");

            if node_id < BRAIN_INPUTS {
                node_values[node_idx] = inputs[node_id];
                continue;
            }

            if let Some(incoming) = self.incoming_forward_connections.get(&node_id) {
                for &conn_idx in incoming {
                    let conn = &self.connections[conn_idx];
                    let from_idx = *self
                        .node_idx_map
                        .get(&conn.from)
                        .expect("Topology error: source node missing from map");
                    node_values[node_idx] += node_values[from_idx] * conn.weight;
                }
            }

            node_values[node_idx] = node_values[node_idx].tanh();
        }

        let mut outputs = [0.0; BRAIN_OUTPUTS];
        for (i, output) in outputs.iter_mut().enumerate() {
            if let Some(&idx) = self.node_idx_map.get(&(i + BRAIN_INPUTS)) {
                *output = node_values[idx];
            }
        }

        let mut next_hidden = [0.0; BRAIN_MEMORY];
        for (i, hidden) in next_hidden.iter_mut().enumerate() {
            if let Some(&idx) = self.node_idx_map.get(&(i + BRAIN_HIDDEN_START)) {
                *hidden = node_values[idx];
            }
        }

        (outputs, next_hidden)
    }

    fn learn(&mut self, activations: &primordium_data::Activations, reinforcement: f32) {
        if self.learning_rate.abs() < 1e-4 || reinforcement.abs() < 1e-4 {
            return;
        }

        let _use_dense = !self.node_idx_map.is_empty();

        for conn in &mut self.connections {
            if !conn.enabled {
                continue;
            }

            let (pre, post) = {
                let f = self.node_idx_map.get(&conn.from);
                let t = self.node_idx_map.get(&conn.to);
                match (f, t) {
                    (Some(&fi), Some(&ti)) => (activations.0[fi], activations.0[ti]),
                    _ => (0.0, 0.0),
                }
            };

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

        let mut protected_nodes = std::collections::HashSet::new();
        if let Some(spec) = specialization {
            use primordium_data::Specialization;
            let target_nodes = match spec {
                Specialization::Soldier => vec![32],
                Specialization::Engineer => vec![38, 39],
                Specialization::Provider => vec![33],
            };
            for &t in &target_nodes {
                protected_nodes.insert(t);
            }
            for c in &self.connections {
                if c.enabled && target_nodes.contains(&c.to) {
                    protected_nodes.insert(c.from);
                }
            }
        }

        for conn in &mut self.connections {
            if rng.gen::<f32>() < rate {
                let mut mut_amount = amount;

                if !protected_nodes.is_empty() && protected_nodes.contains(&conn.to) {
                    mut_amount *= 0.1;
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
        self.initialize_node_idx_map();
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

        let mut child = Brain {
            nodes: child_nodes,
            connections: child_connections,
            next_node_id: self.next_node_id.max(other.next_node_id),
            learning_rate: if rng.gen_bool(0.5) {
                self.learning_rate
            } else {
                other.learning_rate
            },
            weight_deltas: HashMap::new(),
            node_idx_map: HashMap::new(),
            topological_order: Vec::new(),
            forward_connections: Vec::new(),
            recurrent_connections: Vec::new(),
            incoming_forward_connections: HashMap::new(),
        };
        child.initialize_node_idx_map();
        child
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
        self.initialize_node_idx_map();
    }

    fn initialize_node_idx_map(&mut self) {
        self.node_idx_map.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            self.node_idx_map.insert(node.id, idx);
        }

        let mut adj = HashMap::new();
        for conn in &self.connections {
            if conn.enabled {
                adj.entry(conn.from).or_insert_with(Vec::new).push(conn.to);
            }
        }

        let mut order = Vec::new();
        let mut visited = HashMap::new();

        fn dfs(
            u: usize,
            adj: &HashMap<usize, Vec<usize>>,
            visited: &mut HashMap<usize, u8>,
            order: &mut Vec<usize>,
        ) {
            visited.insert(u, 1);
            if let Some(neighbors) = adj.get(&u) {
                for &v in neighbors {
                    if !visited.contains_key(&v) {
                        dfs(v, adj, visited, order);
                    }
                }
            }
            visited.insert(u, 2);
            order.push(u);
        }

        for i in 0..BRAIN_INPUTS {
            if !visited.contains_key(&i) {
                dfs(i, &adj, &mut visited, &mut order);
            }
        }
        for node in &self.nodes {
            if !visited.contains_key(&node.id) {
                dfs(node.id, &adj, &mut visited, &mut order);
            }
        }
        order.reverse();

        let mut node_ranks = HashMap::new();
        for (rank, &id) in order.iter().enumerate() {
            node_ranks.insert(id, rank);
        }

        let mut forward = Vec::new();
        let mut recurrent = Vec::new();
        let mut incoming = HashMap::new();

        for (idx, conn) in self.connections.iter().enumerate() {
            if !conn.enabled {
                continue;
            }
            let from_rank = node_ranks.get(&conn.from).unwrap_or(&0);
            let to_rank = node_ranks.get(&conn.to).unwrap_or(&0);

            if from_rank < to_rank {
                forward.push(idx);
                incoming.entry(conn.to).or_insert_with(Vec::new).push(idx);
            } else {
                recurrent.push(idx);
            }
        }

        self.topological_order = order;
        self.forward_connections = forward;
        self.recurrent_connections = recurrent;
        self.incoming_forward_connections = incoming;
    }
}

impl GenotypeLogic for Genotype {
    fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }

    fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        create_genotype_random_with_rng(rng)
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
