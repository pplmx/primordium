use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;

/// Forward pass through brain.
/// Forward pass through brain.
pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 15],
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
    population: usize,
) {
    let mut rng = rand::thread_rng();

    // NEW: Phase 39 - Genetic Bottleneck & Stasis Logic
    let mut effective_mutation_rate = config.mutation_rate;
    let mut effective_mutation_amount = config.mutation_amount;

    if config.population_aware && population > 0 {
        if population < config.bottleneck_threshold {
            // Population Bottleneck: Increase mutation to find escape routes (Resilience)
            // Scale up to 3x the base rate
            let scaling = (config.bottleneck_threshold as f32 / population as f32).min(3.0);
            effective_mutation_rate *= scaling;
            effective_mutation_amount *= scaling;
        } else if population > config.stasis_threshold {
            // Evolutionary Stasis: Large established populations resist change
            // Scale down to 0.5x
            effective_mutation_rate *= 0.5;
        }
    }

    // 1. Mutate Brain (Topology + Weights)
    // Create a temporary config for the brain mutation that uses the effective rate
    let mut brain_config = config.clone();
    brain_config.mutation_rate = effective_mutation_rate;
    brain_config.mutation_amount = effective_mutation_amount;
    mutate_brain(&mut genotype.brain, &brain_config);

    // 2. Mutate Traits
    let mutate_val = |v: &mut f64, min: f64, max: f64, rng: &mut rand::rngs::ThreadRng| {
        if rng.gen::<f32>() < effective_mutation_rate {
            *v += rng
                .gen_range(-effective_mutation_amount as f64..effective_mutation_amount as f64)
                * *v;
        }
        *v = v.clamp(min, max);
    };

    mutate_val(&mut genotype.sensing_range, 3.0, 15.0, &mut rng);
    mutate_val(&mut genotype.max_speed, 0.5, 3.0, &mut rng);

    // Mutate Maturity Gene (0.5 to 2.0)
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.maturity_gene +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.maturity_gene = genotype.maturity_gene.clamp(0.5, 2.0);

    // Max Energy scales with Maturity (Larger body takes longer to grow)
    genotype.max_energy = (200.0 * genotype.maturity_gene as f64).clamp(100.0, 500.0);

    // Mutate Metabolic Niche (0.0 to 1.0)
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.metabolic_niche +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.metabolic_niche = genotype.metabolic_niche.clamp(0.0, 1.0);

    // Mutate Trophic Potential (0.0 to 1.0)
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.trophic_potential +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.trophic_potential = genotype.trophic_potential.clamp(0.0, 1.0);

    // Mutate Reproductive Investment (0.1 to 0.9)
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.reproductive_investment +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.reproductive_investment = genotype.reproductive_investment.clamp(0.1, 0.9);

    // Mutate Mate Preference (0.0 to 1.0)
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.mate_preference +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.mate_preference = genotype.mate_preference.clamp(0.0, 1.0);

    // NEW: Phase 47 - Mutate Learning Rate (0.0 to 0.5)
    if rng.gen::<f32>() < effective_mutation_rate {
        // Higher mutation for learning rate to kickstart it? No, keep standard.
        // Actually, we delegate to brain.mutate_with_config, but brain is already mutated above.
        // Wait, brain.mutate_with_config DOES mutate learning_rate (if I implemented it right).
        // Let's verify brain.mutate_with_config implementation.
        // Yes, Step 33 implementation included learning_rate mutation.
        // So we don't need to do it here for the Brain struct fields.
    }

    // NEW: Phase 39 - Genetic Drift
    // Small populations experience random trait randomization (Drift)
    if population < 10 && population > 0 && rng.gen_bool(0.05) {
        // Randomly flip a major trait
        match rng.gen_range(0..5) {
            // Increased range
            0 => genotype.trophic_potential = rng.gen_range(0.0..1.0),
            1 => genotype.metabolic_niche = rng.gen_range(0.0..1.0),
            2 => genotype.mate_preference = rng.gen_range(0.0..1.0),
            3 => genotype.maturity_gene = rng.gen_range(0.5..2.0),
            _ => genotype.brain.learning_rate = rng.gen_range(0.0..0.5),
        }
    }
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
    let mate_preference = if rng.gen_bool(0.5) {
        p1.mate_preference
    } else {
        p2.mate_preference
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
        mate_preference,
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
        learning_rate: if rng.gen_bool(0.5) {
            p1.learning_rate
        } else {
            p2.learning_rate
        },
    }
}
