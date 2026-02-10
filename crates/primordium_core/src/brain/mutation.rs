use super::*;
use primordium_data::{Brain, Connection, Node, NodeType, Specialization};
use rand::Rng;

pub fn mutate_with_config<R: Rng>(
    brain: &mut Brain,
    config: &crate::config::AppConfig,
    specialization: Option<Specialization>,
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
        for c in &brain.connections {
            if c.enabled && target_nodes.contains(&c.to) {
                protected_nodes.insert(c.from);
            }
        }
    }

    for conn in &mut brain.connections {
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
        let from_idx = rng.gen_range(0..brain.nodes.len());
        let to_idx = rng.gen_range(0..brain.nodes.len());
        let from = brain.nodes[from_idx].id;
        let to_node = &brain.nodes[to_idx];
        if !matches!(to_node.node_type, NodeType::Input) {
            let innovation = topology::get_innovation_id(from, to_node.id);
            if !brain.connections.iter().any(|c| c.innovation == innovation) {
                brain.connections.push(Connection {
                    from,
                    to: to_node.id,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation,
                });
            }
        }
    }

    if rng.gen::<f32>() < topo_rate * 0.5 && !brain.connections.is_empty() {
        let idx = rng.gen_range(0..brain.connections.len());
        if brain.connections[idx].enabled {
            brain.connections[idx].enabled = false;
            let from = brain.connections[idx].from;
            let to = brain.connections[idx].to;
            let new_id = topology::get_split_node_id(from, to);

            if !brain.nodes.iter().any(|n| n.id == new_id) {
                brain.nodes.push(Node {
                    id: new_id,
                    node_type: NodeType::Hidden,
                    label: None,
                });
            }

            brain.connections.push(Connection {
                from,
                to: new_id,
                weight: 1.0,
                enabled: true,
                innovation: topology::get_innovation_id(from, new_id),
            });
            brain.connections.push(Connection {
                from: new_id,
                to,
                weight: brain.connections[idx].weight,
                enabled: true,
                innovation: topology::get_innovation_id(new_id, to),
            });
        }
    }

    if rng.gen_bool(0.1) {
        brain
            .connections
            .retain(|c| c.weight.abs() >= config.brain.pruning_threshold || !c.enabled);
    }
    brain.initialize_node_idx_map();
}

pub fn remodel_for_adult_with_rng<R: Rng>(brain: &mut Brain, rng: &mut R) {
    let adult_outputs = [34, 35, 36, 37, 38, 39, 40];
    let hidden_nodes: Vec<usize> = brain
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Hidden))
        .map(|n| n.id)
        .collect();

    if hidden_nodes.is_empty() {
        return;
    }

    for &out_id in &adult_outputs {
        let has_conn = brain
            .connections
            .iter()
            .any(|c| c.to == out_id && c.enabled);

        if !has_conn {
            let from = hidden_nodes[rng.gen_range(0..hidden_nodes.len())];
            let innovation = topology::get_innovation_id(from, out_id);
            if !brain.connections.iter().any(|c| c.innovation == innovation) {
                brain.connections.push(Connection {
                    from,
                    to: out_id,
                    weight: rng.gen_range(-1.0..1.0),
                    enabled: true,
                    innovation,
                });
            }
        }
    }
    brain.learning_rate = (brain.learning_rate + 0.05).clamp(0.0, 0.5);
    brain.initialize_node_idx_map();
}
