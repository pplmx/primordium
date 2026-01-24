use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;

pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 16],
    last_hidden: [f32; 6],
) -> ([f32; 9], [f32; 6]) {
    brain.forward(inputs, last_hidden)
}

pub fn mutate_brain(brain: &mut Brain, config: &EvolutionConfig) {
    brain.mutate_with_config(config);
}

pub fn genotype_distance(b1: &Brain, b2: &Brain) -> f32 {
    b1.genotype_distance(b2)
}

pub fn mutate_genotype(
    genotype: &mut crate::model::state::entity::Genotype,
    config: &crate::model::config::EvolutionConfig,
    population: usize,
) {
    let mut rng = rand::thread_rng();
    let mut effective_mutation_rate = config.mutation_rate;
    let mut effective_mutation_amount = config.mutation_amount;

    if config.population_aware && population > 0 {
        if population < config.bottleneck_threshold {
            let scaling = (config.bottleneck_threshold as f32 / population as f32).min(3.0);
            effective_mutation_rate *= scaling;
            effective_mutation_amount *= scaling;
        } else if population > config.stasis_threshold {
            effective_mutation_rate *= 0.5;
        }
    }

    let mut brain_config = config.clone();
    brain_config.mutation_rate = effective_mutation_rate;
    brain_config.mutation_amount = effective_mutation_amount;
    mutate_brain(&mut genotype.brain, &brain_config);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.sensing_range = (genotype.sensing_range
            + rng.gen_range(-effective_mutation_amount as f64..effective_mutation_amount as f64)
                * genotype.sensing_range)
            .clamp(3.0, 15.0);
    }
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.max_speed = (genotype.max_speed
            + rng.gen_range(-effective_mutation_amount as f64..effective_mutation_amount as f64)
                * genotype.max_speed)
            .clamp(0.5, 3.0);
    }

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.maturity_gene +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.maturity_gene = genotype.maturity_gene.clamp(0.5, 2.0);
    genotype.max_energy = (200.0 * genotype.maturity_gene as f64).clamp(100.0, 500.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.metabolic_niche +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.metabolic_niche = genotype.metabolic_niche.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.trophic_potential +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.trophic_potential = genotype.trophic_potential.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.reproductive_investment +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.reproductive_investment = genotype.reproductive_investment.clamp(0.1, 0.9);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.mate_preference +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.mate_preference = genotype.mate_preference.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.pairing_bias +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.pairing_bias = genotype.pairing_bias.clamp(0.0, 1.0);

    if population < 10 && population > 0 && rng.gen_bool(0.05) {
        match rng.gen_range(0..5) {
            0 => genotype.trophic_potential = rng.gen_range(0.0..1.0),
            1 => genotype.metabolic_niche = rng.gen_range(0.0..1.0),
            2 => genotype.mate_preference = rng.gen_range(0.0..1.0),
            3 => genotype.maturity_gene = rng.gen_range(0.5..2.0),
            _ => genotype.pairing_bias = rng.gen_range(0.0..1.0),
        }
    }
}

pub fn crossover_genotypes(
    p1: &crate::model::state::entity::Genotype,
    p2: &crate::model::state::entity::Genotype,
) -> crate::model::state::entity::Genotype {
    let mut rng = rand::thread_rng();
    let brain = crossover_brains(&p1.brain, &p2.brain);

    crate::model::state::entity::Genotype {
        brain,
        sensing_range: if rng.gen_bool(0.5) {
            p1.sensing_range
        } else {
            p2.sensing_range
        },
        max_speed: if rng.gen_bool(0.5) {
            p1.max_speed
        } else {
            p2.max_speed
        },
        max_energy: if rng.gen_bool(0.5) {
            p1.max_energy
        } else {
            p2.max_energy
        },
        lineage_id: p1.lineage_id,
        metabolic_niche: if rng.gen_bool(0.5) {
            p1.metabolic_niche
        } else {
            p2.metabolic_niche
        },
        trophic_potential: if rng.gen_bool(0.5) {
            p1.trophic_potential
        } else {
            p2.trophic_potential
        },
        reproductive_investment: if rng.gen_bool(0.5) {
            p1.reproductive_investment
        } else {
            p2.reproductive_investment
        },
        maturity_gene: if rng.gen_bool(0.5) {
            p1.maturity_gene
        } else {
            p2.maturity_gene
        },
        mate_preference: if rng.gen_bool(0.5) {
            p1.mate_preference
        } else {
            p2.mate_preference
        },
        pairing_bias: if rng.gen_bool(0.5) {
            p1.pairing_bias
        } else {
            p2.pairing_bias
        },
    }
}

pub fn crossover_brains(p1: &Brain, p2: &Brain) -> Brain {
    let mut rng = rand::thread_rng();
    let mut child_nodes = p1.nodes.clone();
    let mut child_connections = Vec::new();
    let mut map2 = std::collections::HashMap::new();
    for c in &p2.connections {
        map2.insert(c.innovation, c);
    }

    for c1 in &p1.connections {
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
        learning_rate: if rng.gen_bool(0.5) {
            p1.learning_rate
        } else {
            p2.learning_rate
        },
    }
}
