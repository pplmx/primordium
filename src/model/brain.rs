use rand::Rng;

#[derive(Clone, Debug)]
pub struct Brain {
    pub weights_ih: [f32; 24], // 4 inputs -> 6 hidden
    pub weights_ho: [f32; 18], // 6 hidden -> 3 outputs
    pub bias_h: [f32; 6],
    pub bias_o: [f32; 3],
}

impl Brain {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let mut weights_ih = [0.0; 24];
        let mut weights_ho = [0.0; 18];
        let mut bias_h = [0.0; 6];
        let mut bias_o = [0.0; 3];

        for w in weights_ih.iter_mut() {
            *w = rng.gen_range(-1.0..1.0);
        }
        for w in weights_ho.iter_mut() {
            *w = rng.gen_range(-1.0..1.0);
        }
        for b in bias_h.iter_mut() {
            *b = rng.gen_range(-1.0..1.0);
        }
        for b in bias_o.iter_mut() {
            *b = rng.gen_range(-1.0..1.0);
        }

        Self {
            weights_ih,
            weights_ho,
            bias_h,
            bias_o,
        }
    }

    pub fn forward(&self, inputs: [f32; 4]) -> [f32; 3] {
        // Input to Hidden
        let mut hidden = [0.0; 6];
        for i in 0..6 {
            let mut sum = self.bias_h[i];
            for j in 0..4 {
                sum += inputs[j] * self.weights_ih[j * 6 + i];
            }
            hidden[i] = sum.tanh();
        }

        // Hidden to Output
        let mut output = [0.0; 3];
        for i in 0..3 {
            let mut sum = self.bias_o[i];
            for j in 0..6 {
                sum += hidden[j] * self.weights_ho[j * 3 + i];
            }
            output[i] = sum.tanh();
        }
        output
    }

    pub fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        let mutation_rate = 0.1;
        let mutation_amount = 0.2;
        let drift_rate = 0.01;
        let drift_amount = 0.5;

        let mut mutate_val = |v: &mut f32| {
            let r = rng.gen::<f32>();
            if r < drift_rate {
                *v += rng.gen_range(-drift_amount..drift_amount);
            } else if r < mutation_rate {
                *v += rng.gen_range(-mutation_amount..mutation_amount);
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
}
