#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteDuration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl NoteDuration {
    pub fn duration_ms(&self, tempo_bpm: u8) -> u32 {
        let beat_ms = 60000 / tempo_bpm as u32;
        match self {
            NoteDuration::Whole => beat_ms * 4,
            NoteDuration::Half => beat_ms * 2,
            NoteDuration::Quarter => beat_ms,
            NoteDuration::Eighth => beat_ms / 2,
            NoteDuration::Sixteenth => beat_ms / 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: u8,
    pub duration: NoteDuration,
    pub velocity: f32,
}

pub struct BioMusicGenerator {
    samples: Vec<f32>,
    index: usize,
    sample_rate: f32,
}

impl BioMusicGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            samples: Vec::new(),
            index: 0,
            sample_rate,
        }
    }

    pub fn load_melody(&mut self, notes: &[Note], tempo_bpm: u8) {
        self.samples.clear();
        self.index = 0;

        for note in notes {
            let duration_samples =
                note.duration.duration_ms(tempo_bpm) as f32 / 1000.0 * self.sample_rate;
            let pitch_freq = 440.0 * 2.0_f32.powf((note.pitch as i32 - 69) as f32 / 12.0);

            for i in 0..duration_samples as usize {
                let t = i as f32 / self.sample_rate;
                let envelope = self.adsr_envelope(t, duration_samples / self.sample_rate);

                let sample = (2.0 * std::f32::consts::PI * pitch_freq * t).sin();
                self.samples.push(sample * envelope * note.velocity);
            }
        }
    }

    pub fn next_sample(&mut self) -> Option<f32> {
        if self.index >= self.samples.len() {
            None
        } else {
            let sample = self.samples[self.index];
            self.index += 1;
            Some(sample)
        }
    }

    pub fn is_playing(&self) -> bool {
        self.index < self.samples.len()
    }

    fn adsr_envelope(&self, t: f32, duration: f32) -> f32 {
        let attack = 0.1;
        let decay = 0.1;
        let sustain = 0.6;
        let release = 0.2;

        if t < attack * duration {
            t / (attack * duration)
        } else if t < (attack + decay) * duration {
            1.0 - (t - attack * duration) / (decay * duration) * (1.0 - sustain)
        } else if t < (1.0 - release) * duration {
            sustain
        } else {
            sustain * (1.0 - (t - (1.0 - release) * duration) / (release * duration))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_duration_ms() {
        assert_eq!(NoteDuration::Quarter.duration_ms(60), 1000);
    }

    #[test]
    fn test_bio_music_generator_new() {
        let gen = BioMusicGenerator::new(44100.0);
        assert_eq!(gen.sample_rate, 44100.0);
        assert_eq!(gen.index, 0);
        assert!(gen.samples.is_empty());
    }

    #[test]
    fn test_load_melody() {
        let mut gen = BioMusicGenerator::new(44100.0);
        let note = Note {
            pitch: 60,
            duration: NoteDuration::Quarter,
            velocity: 0.7,
        };
        gen.load_melody(&[note], 60);
        assert!(!gen.samples.is_empty());
        assert!(gen.is_playing());
    }

    #[test]
    fn test_next_sample() {
        let mut gen = BioMusicGenerator::new(44100.0);
        let note = Note {
            pitch: 69,
            duration: NoteDuration::Eighth,
            velocity: 0.5,
        };
        gen.load_melody(&[note], 120);
        assert!(gen.next_sample().is_some());
        assert!(gen.is_playing());
    }
}
