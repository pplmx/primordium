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
        for (i, h) in hidden.iter_mut().enumerate() {
            let mut sum = self.bias_h[i];
            for (j, &input) in inputs.iter().enumerate() {
                sum += input * self.weights_ih[j * 6 + i];
            }
            *h = sum.tanh();
        }

        // Hidden to Output (6 hidden -> 5 outputs)
        let mut output = [0.0; 5];
        for (i, o) in output.iter_mut().enumerate() {
            let mut sum = self.bias_o[i];
            for (j, &h) in hidden.iter().enumerate() {
                sum += h * self.weights_ho[j * 5 + i];
            }
            *o = sum.tanh();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brain_new_random_has_correct_dimensions() {
        let brain = Brain::new_random();
        assert_eq!(
            brain.weights_ih.len(),
            36,
            "Should have 6x6=36 input-hidden weights"
        );
        assert_eq!(
            brain.weights_ho.len(),
            30,
            "Should have 6x5=30 hidden-output weights"
        );
        assert_eq!(brain.bias_h.len(), 6, "Should have 6 hidden biases");
        assert_eq!(brain.bias_o.len(), 5, "Should have 5 output biases");
    }

    #[test]
    fn test_brain_forward_is_deterministic() {
        let brain = Brain::new_random();
        let inputs = [0.5, -0.5, 0.3, 0.0, 0.1, 0.2];

        let output1 = brain.forward(inputs);
        let output2 = brain.forward(inputs);

        assert_eq!(output1, output2, "Same inputs should produce same outputs");
    }

    #[test]
    fn test_brain_forward_output_in_valid_range() {
        let brain = Brain::new_random();
        let inputs = [1.0, -1.0, 0.5, 0.5, 0.0, 1.0];

        let outputs = brain.forward(inputs);

        for (i, &out) in outputs.iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&out),
                "Output {} should be in [-1, 1], got {}",
                i,
                out
            );
        }
    }

    #[test]
    fn test_brain_mutate_keeps_weights_in_range() {
        let mut brain = Brain::new_random();
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 1.0, // Always mutate
            mutation_amount: 0.5,
            drift_rate: 0.5,
            drift_amount: 0.1,
        };

        // Mutate many times
        for _ in 0..100 {
            brain.mutate_with_config(&config);
        }

        // All weights should still be in [-2, 2]
        for w in &brain.weights_ih {
            assert!(
                *w >= -2.0 && *w <= 2.0,
                "Weight should be clamped to [-2, 2]"
            );
        }
        for w in &brain.weights_ho {
            assert!(
                *w >= -2.0 && *w <= 2.0,
                "Weight should be clamped to [-2, 2]"
            );
        }
    }

    #[test]
    fn test_brain_crossover_produces_valid_child() {
        let parent1 = Brain::new_random();
        let parent2 = Brain::new_random();

        let child = Brain::crossover(&parent1, &parent2);

        // Child should have correct dimensions
        assert_eq!(child.weights_ih.len(), 36);
        assert_eq!(child.weights_ho.len(), 30);

        // Each weight should come from either parent
        for i in 0..child.weights_ih.len() {
            assert!(
                child.weights_ih[i] == parent1.weights_ih[i]
                    || child.weights_ih[i] == parent2.weights_ih[i],
                "Child weight should come from a parent"
            );
        }
    }

    #[test]
    fn test_brain_genotype_distance_self_is_zero() {
        let brain = Brain::new_random();
        let distance = brain.genotype_distance(&brain);
        assert!(
            (distance - 0.0).abs() < 0.0001,
            "Distance to self should be 0"
        );
    }

    #[test]
    fn test_brain_genotype_distance_is_symmetric() {
        let brain1 = Brain::new_random();
        let brain2 = Brain::new_random();

        let d1 = brain1.genotype_distance(&brain2);
        let d2 = brain2.genotype_distance(&brain1);

        assert!((d1 - d2).abs() < 0.0001, "Distance should be symmetric");
    }

    #[test]
    fn test_brain_hex_roundtrip() {
        let original = Brain::new_random();
        let hex = original.to_hex();
        let restored = Brain::from_hex(&hex).expect("Should decode successfully");

        assert_eq!(original.weights_ih, restored.weights_ih);
        assert_eq!(original.weights_ho, restored.weights_ho);
        assert_eq!(original.bias_h, restored.bias_h);
        assert_eq!(original.bias_o, restored.bias_o);
    }
}
