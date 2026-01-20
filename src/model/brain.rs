use rand::Rng;
use serde::{Deserialize, Serialize};

/// Neural network brain with 6 inputs -> 6 hidden -> 5 outputs
///
/// Inputs:
/// 0. Food direction X (-1 to 1)
/// 1. Food direction Y (-1 to 1)
/// 2. Energy level (0 to 1)
/// 3. Neighbor density (0 to 1)
/// 4. Pheromone food strength (0 to 1) [NEW]
/// 5. Tribe density nearby (0 to 1) [NEW]
///
/// Outputs:
/// 0. Movement X (-1 to 1)
/// 1. Movement Y (-1 to 1)
/// 2. Speed (-1 to 1)
/// 3. Aggression (-1 to 1)
/// 4. Share intent (-1 to 1) [NEW]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub weights_ih: Vec<f32>, // 6 inputs -> 6 hidden (36 weights)
    pub weights_ho: Vec<f32>, // 6 hidden -> 5 outputs (30 weights)
    pub bias_h: Vec<f32>,     // 6 hidden biases
    pub bias_o: Vec<f32>,     // 5 output biases
}

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();

        let weights_ih: Vec<f32> = (0..36).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let weights_ho: Vec<f32> = (0..30).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let bias_h: Vec<f32> = (0..6).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let bias_o: Vec<f32> = (0..5).map(|_| rng.gen_range(-1.0..1.0)).collect();

        Self {
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
        }
    }

    pub fn forward(&self, inputs: [f32; 6]) -> [f32; 5] {
        // Input to Hidden (6 inputs -> 6 hidden)
        let mut hidden = [0.0; 6];
        for i in 0..6 {
            let mut sum = self.bias_h[i];
            for j in 0..6 {
                sum += inputs[j] * self.weights_ih[j * 6 + i];
            }
            hidden[i] = sum.tanh();
        }

        // Hidden to Output (6 hidden -> 5 outputs)
        let mut output = [0.0; 5];
        for i in 0..5 {
            let mut sum = self.bias_o[i];
            for j in 0..6 {
                sum += hidden[j] * self.weights_ho[j * 5 + i];
            }
            output[i] = sum.tanh();
        }
        output
    }

    pub fn mutate_with_config(&mut self, config: &crate::model::config::EvolutionConfig) {
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

        for w in self.weights_ih.iter_mut() {
            mutate_val(w);
        }
        for w in self.weights_ho.iter_mut() {
            mutate_val(w);
        }
        for b in self.bias_h.iter_mut() {
            mutate_val(b);
        }
        for b in self.bias_o.iter_mut() {
            mutate_val(b);
        }
    }

    pub fn genotype_distance(&self, other: &Brain) -> f32 {
        let mut sum_sq = 0.0;
        for (w1, w2) in self.weights_ih.iter().zip(other.weights_ih.iter()) {
            sum_sq += (w1 - w2).powi(2);
        }
        for (w1, w2) in self.weights_ho.iter().zip(other.weights_ho.iter()) {
            sum_sq += (w1 - w2).powi(2);
        }
        for (b1, b2) in self.bias_h.iter().zip(other.bias_h.iter()) {
            sum_sq += (b1 - b2).powi(2);
        }
        for (b1, b2) in self.bias_o.iter().zip(other.bias_o.iter()) {
            sum_sq += (b1 - b2).powi(2);
        }
        sum_sq.sqrt()
    }

    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let brain = serde_json::from_slice(&bytes)?;
        Ok(brain)
    }

    pub fn crossover(parent1: &Brain, parent2: &Brain) -> Self {
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
}
