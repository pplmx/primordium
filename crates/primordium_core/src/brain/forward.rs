use super::*;
use primordium_data::Brain;

pub fn forward(
    brain: &Brain,
    inputs: [f32; BRAIN_INPUTS],
    last_hidden: [f32; BRAIN_MEMORY],
) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
    let mut activations = primordium_data::Activations::default();
    forward_internal(brain, inputs, last_hidden, &mut activations)
}

pub fn forward_internal(
    brain: &Brain,
    inputs: [f32; BRAIN_INPUTS],
    _last_hidden: [f32; BRAIN_MEMORY],
    activations: &mut primordium_data::Activations,
) -> ([f32; BRAIN_OUTPUTS], [f32; BRAIN_MEMORY]) {
    let node_count = brain.nodes.len();
    if brain.node_idx_map.is_empty() {
        let outputs = [0.0; BRAIN_OUTPUTS];
        let next_hidden = [0.0; BRAIN_MEMORY];
        return (outputs, next_hidden);
    }

    std::mem::swap(&mut activations.0, &mut activations.1);
    activations.prepare(node_count);

    let prev_values = &activations.1;
    let node_values = &mut activations.0;

    for &conn_idx in &brain.recurrent_connections {
        let conn = &brain.connections[conn_idx];
        if let (Some(&f_idx), Some(&t_idx)) = (
            brain.node_idx_map.get(&conn.from),
            brain.node_idx_map.get(&conn.to),
        ) {
            if f_idx < prev_values.len() {
                node_values[t_idx] += prev_values[f_idx] * conn.weight;
            }
        }
    }

    for &node_idx in &brain.fast_forward_order {
        let node_id = brain.nodes[node_idx].id;

        if node_id < BRAIN_INPUTS {
            node_values[node_idx] = inputs[node_id];
            continue;
        }

        if node_idx < brain.incoming_offsets.len() - 1 {
            let start = brain.incoming_offsets[node_idx];
            let end = brain.incoming_offsets[node_idx + 1];
            for &(from_idx, conn_idx) in &brain.incoming_flat[start..end] {
                node_values[node_idx] += node_values[from_idx] * brain.connections[conn_idx].weight;
            }
        }

        node_values[node_idx] = node_values[node_idx].tanh();
    }

    let mut outputs = [0.0; BRAIN_OUTPUTS];
    for (i, output) in outputs.iter_mut().enumerate() {
        if let Some(&idx) = brain.node_idx_map.get(&(i + BRAIN_INPUTS)) {
            *output = node_values[idx];
        }
    }

    let mut next_hidden = [0.0; BRAIN_MEMORY];
    for (i, hidden) in next_hidden.iter_mut().enumerate() {
        if let Some(&idx) = brain.node_idx_map.get(&(i + BRAIN_HIDDEN_START)) {
            *hidden = node_values[idx];
        }
    }

    (outputs, next_hidden)
}

pub fn learn(brain: &mut Brain, activations: &primordium_data::Activations, reinforcement: f32) {
    if brain.learning_rate.abs() < 1e-4 || reinforcement.abs() < 1e-4 {
        return;
    }

    for conn in &mut brain.connections {
        if !conn.enabled {
            continue;
        }

        let (pre, post) = {
            let f = brain.node_idx_map.get(&conn.from);
            let t = brain.node_idx_map.get(&conn.to);
            match (f, t) {
                (Some(&fi), Some(&ti)) => (activations.0[fi], activations.0[ti]),
                _ => (0.0, 0.0),
            }
        };

        let reinforcement = reinforcement.clamp(-10.0, 10.0);
        let delta = brain.learning_rate * reinforcement * pre * post;
        conn.weight += delta;
        conn.weight = conn.weight.clamp(-5.0, 5.0);
        let entry = brain.weight_deltas.entry(conn.innovation).or_insert(0.0);
        *entry = (*entry * 0.9) + delta.abs();
    }
}
