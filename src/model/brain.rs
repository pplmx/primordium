use rand::Rng;
use serde::{Deserialize, Serialize};

/// Neural network brain with Recurrent (Memory) capability.
///
/// Architecture: 13 inputs -> 6 hidden -> 6 outputs
///
/// Inputs:
/// 0-5. Environmental sensors (Food X, Food Y, Energy, Density, Pheromone, Tribe)
/// 6. Same-Lineage Density nearby
/// 7-12. Recurrent memory (last tick's hidden layer)
///
/// Outputs:
/// 0. Movement X
/// 1. Movement Y
/// 2. Speed
/// 3. Aggression
/// 4. Share intent
/// 5. Signal (Color Modulation)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Brain {
    pub weights_ih: Vec<f32>, // 13 inputs -> 6 hidden (78 weights)
    pub weights_ho: Vec<f32>, // 6 hidden -> 6 outputs (36 weights)
    pub bias_h: Vec<f32>,     // 6 hidden biases
    pub bias_o: Vec<f32>,     // 6 output biases
}

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();

        let weights_ih: Vec<f32> = (0..78).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let weights_ho: Vec<f32> = (0..36).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let bias_h: Vec<f32> = (0..6).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let bias_o: Vec<f32> = (0..6).map(|_| rng.gen_range(-1.0..1.0)).collect();

        Self {
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
        }
    }

    /// Forward pass with recurrence. Takes environmental inputs and previous hidden state.
    /// Returns (Outputs, New Hidden State).
    pub fn forward(&self, inputs: [f32; 7], last_hidden: [f32; 6]) -> ([f32; 6], [f32; 6]) {
        // 1. Combine inputs with memory
        let mut combined_inputs = [0.0; 13];
        combined_inputs[0..7].copy_from_slice(&inputs);
        combined_inputs[7..13].copy_from_slice(&last_hidden);

        // 2. Input to Hidden (13 inputs -> 6 hidden)
        let mut hidden = [0.0; 6];
        for (i, h) in hidden.iter_mut().enumerate() {
            let mut sum = self.bias_h[i];
            for (j, &input) in combined_inputs.iter().enumerate() {
                sum += input * self.weights_ih[j * 6 + i];
            }
            *h = sum.tanh();
        }

        // 3. Hidden to Output (6 hidden -> 6 outputs)
        let mut output = [0.0; 6];
        for (i, o) in output.iter_mut().enumerate() {
            let mut sum = self.bias_o[i];
            for (j, &h) in hidden.iter().enumerate() {
                sum += h * self.weights_ho[j * 6 + i];
            }
            *o = sum.tanh();
        }
        (output, hidden)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::systems::intel;

    #[test]
    fn test_brain_new_random_has_correct_dimensions() {
        let brain = Brain::new_random();
        assert_eq!(
            brain.weights_ih.len(),
            78,
            "Should have 13x6=78 input-hidden weights"
        );
        assert_eq!(
            brain.weights_ho.len(),
            36,
            "Should have 6x6=36 hidden-output weights"
        );
        assert_eq!(brain.bias_h.len(), 6, "Should have 6 hidden biases");
        assert_eq!(brain.bias_o.len(), 6, "Should have 6 output biases");
    }

    #[test]
    fn test_brain_forward_is_deterministic() {
        let brain = Brain::new_random();
        let inputs = [0.5, -0.5, 0.3, 0.0, 0.1, 0.2, 0.1];
        let last_hidden = [0.0; 6];

        let (output1, _) = intel::brain_forward(&brain, inputs, last_hidden);
        let (output2, _) = intel::brain_forward(&brain, inputs, last_hidden);

        assert_eq!(output1, output2, "Same inputs should produce same outputs");
    }

    #[test]
    fn test_brain_forward_output_in_valid_range() {
        let brain = Brain::new_random();
        let inputs = [1.0, -1.0, 0.5, 0.5, 0.0, 1.0, 0.0];
        let last_hidden = [0.1; 6];

        let (outputs, next_hidden) = intel::brain_forward(&brain, inputs, last_hidden);

        for (i, &out) in outputs.iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&out),
                "Output {} should be in [-1, 1], got {}",
                i,
                out
            );
        }
        for (i, &h) in next_hidden.iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&h),
                "Hidden {} should be in [-1, 1], got {}",
                i,
                h
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
            speciation_rate: 0.0,
        };

        // Mutate many times
        for _ in 0..100 {
            intel::mutate_brain(&mut brain, &config);
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

        let child = intel::crossover_brains(&parent1, &parent2);

        // Child should have correct dimensions
        assert_eq!(child.weights_ih.len(), 78);
        assert_eq!(child.weights_ho.len(), 36);

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
    fn test_brain_recurrence_memory_impact() {
        let brain = Brain::new_random();
        let inputs = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];

        // Two different last states
        let state_a = [0.9; 6];
        let state_b = [-0.9; 6];

        let (out_a, next_a) = intel::brain_forward(&brain, inputs, state_a);
        let (out_b, next_b) = intel::brain_forward(&brain, inputs, state_b);

        // Outputs should differ because memory state differs
        assert_ne!(
            out_a, out_b,
            "Different memory states should produce different outputs"
        );
        assert_ne!(
            next_a, next_b,
            "Different memory states should lead to different next states"
        );
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
