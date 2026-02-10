use super::*;
use primordium_data::{Brain, Genotype, Node};
use rand::Rng;
use std::collections::HashMap;

pub fn brain_crossover_with_rng<R: Rng>(brain: &Brain, other: &Brain, rng: &mut R) -> Brain {
    let mut child_nodes = brain.nodes.clone();
    let mut child_connections = Vec::new();
    let mut map2 = HashMap::new();
    for c in &other.connections {
        map2.insert(c.innovation, c);
    }

    for c1 in &brain.connections {
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
        next_node_id: brain.next_node_id.max(other.next_node_id),
        learning_rate: if rng.gen_bool(0.5) {
            brain.learning_rate
        } else {
            other.learning_rate
        },
        weight_deltas: HashMap::new(),
        node_idx_map: HashMap::new(),
        topological_order: Vec::new(),
        forward_connections: Vec::new(),
        recurrent_connections: Vec::new(),
        incoming_forward_connections: HashMap::new(),
        fast_forward_order: Vec::new(),
        fast_incoming: Vec::new(),
    };
    child.initialize_node_idx_map();
    child
}

pub fn genotype_crossover_with_rng<R: Rng>(
    genotype: &Genotype,
    other: &Genotype,
    rng: &mut R,
) -> Genotype {
    use crate::brain::BrainLogic;
    let brain = genotype.brain.crossover_with_rng(&other.brain, rng);

    let mut child_genotype = if rng.gen_bool(0.5) {
        genotype.clone()
    } else {
        other.clone()
    };
    child_genotype.brain = brain;
    child_genotype
}
