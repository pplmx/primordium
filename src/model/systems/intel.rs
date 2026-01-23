use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;

/// Forward pass through brain.
pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 14],
    last_hidden: [f32; 6],
) -> ([f32; 8], [f32; 6]) {
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

    // 1. Mutate Brain (Topology + Weights)
    mutate_brain(&mut genotype.brain, config);

    // 2. Mutate Traits
    let mutate_val = |v: &mut f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng| {
        if rng.gen::<f32>() < config.mutation_rate {
            *v += rng.gen_range(-config.mutation_amount as f64..config.mutation_amount as f64) * *v;
        }
        *v = v.clamp(min, max);
    };

    mutate_val(&mut genotype.sensing_range, 3.0, 15.0, &mut rng);
    mutate_val(&mut genotype.max_speed, 0.5, 3.0, &mut rng);

    // Mutate Maturity Gene (0.5 to 2.0)
    if rng.gen::<f32>() < config.mutation_rate {
        genotype.maturity_gene += rng.gen_range(-config.mutation_amount..config.mutation_amount);
    }
    genotype.maturity_gene = genotype.maturity_gene.clamp(0.5, 2.0);

    // Max Energy scales with Maturity (Larger body takes longer to grow)
    genotype.max_energy = (200.0 * genotype.maturity_gene as f64).clamp(100.0, 500.0);

    // Mutate Metabolic Niche (0.0 to 1.0)
    if rng.gen::<f32>() < config.mutation_rate {
        genotype.metabolic_niche += rng.gen_range(-config.mutation_amount..config.mutation_amount);
    }
    genotype.metabolic_niche = genotype.metabolic_niche.clamp(0.0, 1.0);

    // Mutate Trophic Potential (0.0 to 1.0)
    if rng.gen::<f32>() < config.mutation_rate {
        genotype.trophic_potential +=
            rng.gen_range(-config.mutation_amount..config.mutation_amount);
    }
    genotype.trophic_potential = genotype.trophic_potential.clamp(0.0, 1.0);

    // Mutate Reproductive Investment (0.1 to 0.9)
    if rng.gen::<f32>() < config.mutation_rate {
        genotype.reproductive_investment +=
            rng.gen_range(-config.mutation_amount..config.mutation_amount);
    }
    genotype.reproductive_investment = genotype.reproductive_investment.clamp(0.1, 0.9);
}

/// Crossover between two genotypes.
pub fn crossover_genotypes(
    p1: &crate::model::state::entity::Genotype,
    p2: &crate::model::state::entity::Genotype,
) -> crate::model::state::entity::Genotype {
    let mut rng = rand::thread_rng();

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
    let metabolic_niche = if rng.gen_bool(0.5) {
        p1.metabolic_niche
    } else {
        p2.metabolic_niche
    };
    let trophic_potential = if rng.gen_bool(0.5) {
        p1.trophic_potential
    } else {
        p2.trophic_potential
    };
    let reproductive_investment = if rng.gen_bool(0.5) {
        p1.reproductive_investment
    } else {
        p2.reproductive_investment
    };
    let maturity_gene = if rng.gen_bool(0.5) {
        p1.maturity_gene
    } else {
        p2.maturity_gene
    };

    crate::model::state::entity::Genotype {
        brain,
        sensing_range,
        max_speed,
        max_energy,
        lineage_id: p1.lineage_id, // Inherit lineage from first parent
        metabolic_niche,
        trophic_potential,
        reproductive_investment,
        maturity_gene,
    }
}

/// Perform crossover between two parent brains to create a child brain.
pub fn crossover_brains(p1: &Brain, p2: &Brain) -> Brain {
    let mut rng = rand::thread_rng();
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
