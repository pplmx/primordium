use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;

/// Forward pass through brain.
pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 7],
    last_hidden: [f32; 6],
) -> ([f32; 6], [f32; 6]) {
    brain.forward(inputs, last_hidden)
}

/// Mutate brain weights and biases.
pub fn mutate_brain(brain: &mut Brain, config: &EvolutionConfig) {
    brain.mutate_with_config(config);
}

/// Calculate genetic distance between two brains.
pub fn genotype_distance(b1: &Brain, b2: &Brain) -> f32 {
    b1.genotype_distance(b2)
}

/// Mutate physical phenotype traits within a genotype.
pub fn mutate_genotype(
    genotype: &mut crate::model::state::entity::Genotype,
    config: &crate::model::config::EvolutionConfig,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    // 1. Mutate Brain (Topology + Weights)
    mutate_brain(&mut genotype.brain, config);

    // 2. Mutate Traits
    let mut mutate_trait = |v: &mut f64, min: f64, max: f64| {
        let r = rng.gen::<f32>();
        if r < config.mutation_rate {
            *v += rng.gen_range(-config.mutation_amount as f64..config.mutation_amount as f64) * *v;
        }
        *v = v.clamp(min, max);
    };

    mutate_trait(&mut genotype.sensing_range, 3.0, 15.0);
    mutate_trait(&mut genotype.max_speed, 0.5, 3.0);
    mutate_trait(&mut genotype.max_energy, 100.0, 500.0);
}

/// Crossover between two genotypes.
pub fn crossover_genotypes(
    p1: &crate::model::state::entity::Genotype,
    p2: &crate::model::state::entity::Genotype,
) -> crate::model::state::entity::Genotype {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    let brain = crossover_brains(&p1.brain, &p2.brain);
    let sensing_range = if rng.gen_bool(0.5) {
        p1.sensing_range
    } else {
        p2.sensing_range
    };
    let max_speed = if rng.gen_bool(0.5) {
        p1.max_speed
    } else {
        p2.max_speed
    };
    let max_energy = if rng.gen_bool(0.5) {
        p1.max_energy
    } else {
        p2.max_energy
    };

    crate::model::state::entity::Genotype {
        brain,
        sensing_range,
        max_speed,
        max_energy,
        lineage_id: p1.lineage_id, // Inherit lineage from first parent
    }
}

/// Perform crossover between two parent brains to create a child brain.
pub fn crossover_brains(p1: &Brain, p2: &Brain) -> Brain {
    let mut rng = rand::thread_rng();
    use rand::Rng;
    let mut child_nodes = p1.nodes.clone();
    let mut child_connections = Vec::new();

    let mut map2 = std::collections::HashMap::new();
    for c in &p2.connections {
        map2.insert(c.innovation, c);
    }

    // 1. Inherit connections
    for c1 in &p1.connections {
        if let Some(c2) = map2.get(&c1.innovation) {
            // Matching: Pick randomly
            if rng.gen_bool(0.5) {
                child_connections.push(c1.clone());
            } else {
                child_connections.push((*c2).clone());
            }
        } else {
            // Disjoint (only in P1): Inherit from P1
            child_connections.push(c1.clone());
        }
    }

    // 2. Ensure all necessary nodes exist for the inherited connections
    for c in &child_connections {
        if !child_nodes.iter().any(|n| n.id == c.from) {
            if let Some(n) = p2.nodes.iter().find(|n| n.id == c.from) {
                child_nodes.push(n.clone());
            }
        }
        if !child_nodes.iter().any(|n| n.id == c.to) {
            if let Some(n) = p2.nodes.iter().find(|n| n.id == c.to) {
                child_nodes.push(n.clone());
            }
        }
    }

    Brain {
        nodes: child_nodes,
        connections: child_connections,
        next_node_id: p1.next_node_id.max(p2.next_node_id),
    }
}
