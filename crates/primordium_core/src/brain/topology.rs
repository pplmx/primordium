use super::*;
use primordium_data::{Brain, Connection, Genotype, Node, NodeType};
use rand::Rng;
use std::collections::HashMap;

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
        fast_forward_order: Vec::new(),
        incoming_flat: Vec::new(),
        incoming_offsets: Vec::new(),
    };
    brain.initialize_node_idx_map();
    brain
}

pub fn get_innovation_id(from: usize, to: usize) -> usize {
    let h = (from as u64) << 32 | (to as u64);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in h.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3_u64);
    }
    (hash as usize) & 0x7FFF_FFFF
}

pub fn get_split_node_id(from: usize, to: usize) -> usize {
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
        max_energy: rng.gen_range(400.0..600.0),
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

pub fn initialize_node_idx_map(brain: &mut Brain) {
    brain.node_idx_map.clear();
    for (idx, node) in brain.nodes.iter().enumerate() {
        brain.node_idx_map.insert(node.id, idx);
    }

    let mut adj = HashMap::new();
    for conn in &brain.connections {
        if conn.enabled {
            adj.entry(conn.from).or_insert_with(Vec::new).push(conn.to);
        }
    }

    let mut order = Vec::new();
    let mut visited = HashMap::new();
    let max_depth = brain.nodes.len() + BRAIN_INPUTS + 100;

    fn dfs(
        u: usize,
        adj: &HashMap<usize, Vec<usize>>,
        visited: &mut HashMap<usize, u8>,
        order: &mut Vec<usize>,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth {
            return;
        }

        visited.insert(u, 1);
        if let Some(neighbors) = adj.get(&u) {
            for &v in neighbors {
                if !visited.contains_key(&v) {
                    dfs(v, adj, visited, order, depth + 1, max_depth);
                }
                // If neighbor is currently being visited (value 1), we found a cycle
                // Mark it as visited to prevent infinite recursion
                if visited.get(&v) == Some(&1) {
                    visited.insert(v, 2);
                }
            }
        }
        visited.insert(u, 2);
        order.push(u);
    }

    for i in 0..BRAIN_INPUTS {
        if !visited.contains_key(&i) {
            dfs(i, &adj, &mut visited, &mut order, 0, max_depth);
        }
    }
    for node in &brain.nodes {
        if !visited.contains_key(&node.id) {
            dfs(node.id, &adj, &mut visited, &mut order, 0, max_depth);
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

    let node_ids = brain
        .nodes
        .iter()
        .map(|n| n.id)
        .collect::<std::collections::HashSet<_>>();

    for idx in 0..brain.connections.len() {
        let conn = &mut brain.connections[idx];
        if !conn.enabled {
            continue;
        }

        if !node_ids.contains(&conn.from) || !node_ids.contains(&conn.to) {
            conn.enabled = false;
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

    let mut fast_forward_order = Vec::with_capacity(order.len());
    for &id in &order {
        if let Some(&idx) = brain.node_idx_map.get(&id) {
            fast_forward_order.push(idx);
        }
    }

    let mut incoming_flat = Vec::new();
    let mut incoming_offsets = Vec::with_capacity(brain.nodes.len() + 1);
    incoming_offsets.push(0);

    for node_idx in 0..brain.nodes.len() {
        let to_id = brain.nodes[node_idx].id;
        if let Some(conn_indices) = incoming.get(&to_id) {
            for &conn_idx in conn_indices {
                let conn = &brain.connections[conn_idx];
                if let Some(&from_idx) = brain.node_idx_map.get(&conn.from) {
                    incoming_flat.push((from_idx, conn_idx));
                }
            }
        }
        incoming_offsets.push(incoming_flat.len());
    }

    brain.topological_order = order;
    brain.forward_connections = forward;
    brain.recurrent_connections = recurrent;
    brain.incoming_forward_connections = incoming;
    brain.fast_forward_order = fast_forward_order;
    brain.incoming_flat = incoming_flat;
    brain.incoming_offsets = incoming_offsets;
}
