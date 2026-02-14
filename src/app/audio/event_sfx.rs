use primordium_core::systems::audio::AudioEvent;
use std::f32::consts::PI;

pub struct EventSFXGenerator {
    sample_rate: f32,
}

impl EventSFXGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    pub fn generate_waveform(&self, event: AudioEvent) -> Vec<f32> {
        match event {
            AudioEvent::Birth => self.birth_waveform(),
            AudioEvent::Death => self.death_waveform(),
            AudioEvent::ClimateShift => self.climate_waveform(),
            AudioEvent::Metamorphosis => self.metamorphosis_waveform(),
            AudioEvent::NewEra => self.new_era_waveform(),
            AudioEvent::AmbientShift => vec![],
        }
    }

    fn birth_waveform(&self) -> Vec<f32> {
        let duration = 0.3;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count as f32;
            let frequency = 440.0 + t * 880.0;

            let sample = (2.0_f32 * PI * frequency * t).sin()
                + 0.5 * (2.0_f32 * PI * frequency * 3.0 * t).sin();

            let envelope = match t {
                t if t < 0.1 => t / 0.1,
                t if t < 0.2 => 1.0 - (t - 0.1) / 0.1,
                _ => 0.8 * (1.0 - t) / 0.1,
            };

            waveform.push(sample * envelope);
        }

        waveform
    }

    fn death_waveform(&self) -> Vec<f32> {
        let duration = 0.5;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count as f32;
            let carrier_freq = 220.0 * (1.0 - t);
            let noise = (rand::random::<f32>() - 0.5) * 2.0;

            let sample = (2.0_f32 * PI * carrier_freq * t).sin() + 0.3 * noise;

            let envelope = (1.0 - t).powf(2.0);
            waveform.push(sample * envelope);
        }

        waveform
    }

    fn climate_waveform(&self) -> Vec<f32> {
        let duration = 1.0;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count as f32;
            let frequency = 100.0 + t * 900.0;

            let sample = (2.0_f32 * PI * frequency * t).sin() * (f32::sin(10.0 * t) * 0.5 + 0.5);

            let envelope = (f32::sin(PI * t)).powf(2.0);
            waveform.push(sample * envelope);
        }

        waveform
    }

    fn metamorphosis_waveform(&self) -> Vec<f32> {
        let duration = 0.4;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        let chord_frequencies = [330.0, 392.0, 494.0];

        for i in 0..sample_count {
            let t = i as f32 / sample_count as f32;
            let note_idx = ((t * 8.0) as usize) % chord_frequencies.len();
            let frequency = chord_frequencies[note_idx] * (1.0 + t * 0.5);

            let sample = (2.0_f32 * PI * frequency * t).sin();
            let envelope = match t {
                t if t < 0.05 => t / 0.05,
                t if t < 0.35 => 1.0 - (t - 0.05) / 0.3,
                _ => 0.0,
            };

            waveform.push(sample * envelope);
        }

        waveform
    }

    fn new_era_waveform(&self) -> Vec<f32> {
        let duration = 0.3;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count as f32;
            let frequency = 880.0 * (1.5 - t);

            let sample = (2.0_f32 * PI * frequency * t).sin()
                + 0.3 * (2.0_f32 * PI * frequency * 2.0 * t).sin();

            let envelope = (1.0 - t).powf(0.5);
            waveform.push(sample * envelope);
        }

        waveform
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_new() {
        let gen = EventSFXGenerator::new(44100.0);
        assert_eq!(gen.sample_rate, 44100.0);
    }

    #[test]
    fn test_birth_waveform_length() {
        let gen = EventSFXGenerator::new(44100.0);
        let waveform = gen.birth_waveform();
        assert_eq!(waveform.len(), (0.3 * 44100.0) as usize);
    }

    #[test]
    fn test_waveform_boundedness() {
        let gen = EventSFXGenerator::new(44100.0);

        for event in [
            AudioEvent::Birth,
            AudioEvent::Death,
            AudioEvent::ClimateShift,
        ] {
            let waveform = gen.generate_waveform(event);
            let max_sample = waveform
                .iter()
                .map(|s| s.abs())
                .fold(0.0f32, |a, b| a.max(b));
            assert!(
                max_sample < 10.0,
                "Max sample {} for {:?}",
                max_sample,
                event
            );
        }
    }
}
