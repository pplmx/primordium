use crate::model::brain::Brain;
use rand::Rng;

/// Forward pass with recurrence. Takes environmental inputs and previous hidden state.
/// Returns (Outputs, New Hidden State).
pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 6],
    last_hidden: [f32; 6],
) -> ([f32; 5], [f32; 6]) {
    // 1. Combine inputs with memory
    let mut combined_inputs = [0.0; 12];
    combined_inputs[0..6].copy_from_slice(&inputs);
    combined_inputs[6..12].copy_from_slice(&last_hidden);

    // 2. Input to Hidden (12 inputs -> 6 hidden)
    let mut hidden = [0.0; 6];
    for (i, h) in hidden.iter_mut().enumerate() {
        let mut sum = brain.bias_h[i];
        for (j, &input) in combined_inputs.iter().enumerate() {
            sum += input * brain.weights_ih[j * 6 + i];
        }
        *h = sum.tanh();
    }

    // 3. Hidden to Output (6 hidden -> 5 outputs)
    let mut output = [0.0; 5];
    for (i, o) in output.iter_mut().enumerate() {
        let mut sum = brain.bias_o[i];
        for (j, &h) in hidden.iter().enumerate() {
            sum += h * brain.weights_ho[j * 5 + i];
        }
        *o = sum.tanh();
    }
    (output, hidden)
}

/// Calculate genotype distance between two brains.
pub fn genotype_distance(b1: &Brain, b2: &Brain) -> f32 {
    let mut sum_sq = 0.0;
    for (w1, w2) in b1.weights_ih.iter().zip(b2.weights_ih.iter()) {
        sum_sq += (w1 - w2).powi(2);
    }
    for (w1, w2) in b1.weights_ho.iter().zip(b2.weights_ho.iter()) {
        sum_sq += (w1 - w2).powi(2);
    }
    for (b1, b2) in b1.bias_h.iter().zip(b2.bias_h.iter()) {
        sum_sq += (b1 - b2).powi(2);
    }
    for (b1, b2) in b1.bias_o.iter().zip(b2.bias_o.iter()) {
        sum_sq += (b1 - b2).powi(2);
    }
    sum_sq.sqrt()
}

/// Mutate brain weights based on evolution config.
pub fn mutate_brain(brain: &mut Brain, config: &crate::model::config::EvolutionConfig) {
    let mut rng = rand::thread_rng();

    let mut mutate_val = |v: &mut f32| {
        let r = rng.gen::<f32>();
        if r < config.drift_rate {
            *v += rng.gen_range(-config.drift_amount..config.drift_amount);
        } else if r < config.mutation_rate {
            *v += rng.gen_range(-config.mutation_amount..config.mutation_amount);
        }
        *v = v.clamp(-2.0, 2.0);
    };

    for w in brain.weights_ih.iter_mut() {
        mutate_val(w);
    }
    for w in brain.weights_ho.iter_mut() {
        mutate_val(w);
    }
    for b in brain.bias_h.iter_mut() {
        mutate_val(b);
    }
    for b in brain.bias_o.iter_mut() {
        mutate_val(b);
    }
}

/// Perform crossover between two parent brains to create a child brain.
pub fn crossover_brains(parent1: &Brain, parent2: &Brain) -> Brain {
    let mut rng = rand::thread_rng();
    let mut child = parent1.clone();

    // Randomly pick weights from either parent
    for i in 0..child.weights_ih.len() {
        if rng.gen_bool(0.5) {
            child.weights_ih[i] = parent2.weights_ih[i];
        }
    }
    for i in 0..child.weights_ho.len() {
        if rng.gen_bool(0.5) {
            child.weights_ho[i] = parent2.weights_ho[i];
        }
    }
    for i in 0..child.bias_h.len() {
        if rng.gen_bool(0.5) {
            child.bias_h[i] = parent2.bias_h[i];
        }
    }
    for i in 0..child.bias_o.len() {
        if rng.gen_bool(0.5) {
            child.bias_o[i] = parent2.bias_o[i];
        }
    }
    child
}

/// Mutate physical phenotype traits within a genotype.
pub fn mutate_genotype(
    genotype: &mut crate::model::state::entity::Genotype,
    config: &crate::model::config::EvolutionConfig,
) {
    let mut rng = rand::thread_rng();

    // 1. Mutate Brain
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
