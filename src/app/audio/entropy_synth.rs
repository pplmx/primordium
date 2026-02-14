use std::f32::consts::PI;

pub struct FMSynthesizer {
    carrier_phase: f32,
    modulator_phase: f32,
    sample_rate: f32,
}

impl FMSynthesizer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            carrier_phase: 0.0,
            modulator_phase: 0.0,
            sample_rate,
        }
    }

    pub fn render_sample(&mut self, entropy: f32, biomass: f32) -> f32 {
        let carrier_freq = 200.0 + entropy * 500.0;
        let modulator_freq = carrier_freq * (1.0 + entropy);
        let modulation_index = biomass * 0.001;

        self.modulator_phase += modulator_freq / self.sample_rate;
        self.modulator_phase %= 1.0;
        let modulator_signal = (self.modulator_phase * 2.0 * PI).sin();
        let modulator = modulation_index * modulator_signal;

        self.carrier_phase += carrier_freq / self.sample_rate;
        self.carrier_phase %= 1.0;
        let carrier = (self.carrier_phase * 2.0 * PI).sin();

        carrier * modulator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fm_synthesizer_new() {
        let synth = FMSynthesizer::new(44100.0);
        assert_eq!(synth.sample_rate, 44100.0);
        assert_eq!(synth.carrier_phase, 0.0);
        assert_eq!(synth.modulator_phase, 0.0);
    }

    #[test]
    fn test_fm_sample_generation() {
        let mut synth = FMSynthesizer::new(44100.0);
        let sample = synth.render_sample(0.5, 1000.0);
        assert!(sample >= -1.0 && sample <= 1.0);
    }

    #[test]
    fn test_entropy_monotonicity() {
        let mut synth = FMSynthesizer::new(44100.0);

        let sample_low = synth.render_sample(0.1, 1000.0);

        synth.carrier_phase = 0.0;
        synth.modulator_phase = 0.0;

        let sample_ref = synth.render_sample(0.5, 1000.0);

        synth.carrier_phase = 0.0;
        synth.modulator_phase = 0.0;

        let sample_high = synth.render_sample(0.9, 1000.0);

        assert_ne!(sample_low, sample_high);
        assert_ne!(sample_high, sample_ref);
    }

    #[test]
    fn test_sample_boundedness() {
        let mut synth = FMSynthesizer::new(44100.0);

        for entropy in 0..11 {
            let sample = synth.render_sample(entropy as f32 / 10.0, 1000.0);
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "Sample {} out of bounds at entropy={}",
                sample,
                entropy
            );
        }
    }
}
