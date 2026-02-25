use crate::app::audio::bio_music::BioMusicGenerator;
use crate::app::audio::entropy_synth::FMSynthesizer;
use crate::app::audio::event_sfx::EventSFXGenerator;
use primordium_core::systems::audio::AudioEvent;
use std::sync::mpsc::{self, Receiver, Sender};

pub trait AudioOutput: Send + Sync {
    fn write_samples(&mut self, samples: &[f32]) -> Result<(), AudioError>;
    fn set_sample_rate(&mut self, rate: f32);
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("Audio initialization failed")]
    InitializationFailed,
    #[error("Audio output error: {0}")]
    OutputError(String),
    #[error("Sample rate not supported")]
    UnsupportedSampleRate,
}

#[allow(dead_code)] // Fields are used in tests and reserved for future audio configuration
pub struct AudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    entropy_synth: FMSynthesizer,
    bio_music: BioMusicGenerator,
    event_sfx: EventSFXGenerator,
    event_queue: Sender<AudioEvent>,
    event_receiver: Receiver<AudioEvent>,
    current_entropy: f32,
    current_biomass: f32,
    background_running: bool,
    active_event_sfx: Option<Vec<f32>>,
    event_sfx_index: usize,
    spatial_sfx_left_gain: f32,
    spatial_sfx_right_gain: f32,
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        let (tx, rx) = mpsc::channel();

        Ok(Self {
            sample_rate: 44100.0,
            buffer_size: 256,
            entropy_synth: FMSynthesizer::new(44100.0),
            bio_music: BioMusicGenerator::new(44100.0),
            event_sfx: EventSFXGenerator::new(44100.0),
            event_queue: tx,
            event_receiver: rx,
            current_entropy: 0.5,
            current_biomass: 1000.0_f32,
            background_running: false,
            active_event_sfx: None,
            event_sfx_index: 0,
            spatial_sfx_left_gain: 1.0,
            spatial_sfx_right_gain: 1.0,
        })
    }

    pub fn render_block(&mut self, out_buffer: &mut [f32]) {
        for sample in out_buffer.iter_mut() {
            let mut mix = 0.0;

            if self.background_running {
                mix += 0.3
                    * self
                        .entropy_synth
                        .render_sample(self.current_entropy, self.current_biomass);
            }

            if self.bio_music.is_playing() {
                if let Some(s) = self.bio_music.next_sample() {
                    mix += 0.5 * s;
                }
            }

            if let Some(ref sfx) = self.active_event_sfx {
                if self.event_sfx_index < sfx.len() {
                    mix += 0.8 * sfx[self.event_sfx_index];
                    self.event_sfx_index += 1;
                } else {
                    self.active_event_sfx = None;
                    self.event_sfx_index = 0;
                }
            }

            *sample = mix;
        }

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event);
        }
    }

    pub fn render_block_stereo(&mut self, out_buffer: &mut [f32; 2]) {
        let mut mix_left = 0.0;
        let mut mix_right = 0.0;

        if self.background_running {
            let sample = self
                .entropy_synth
                .render_sample(self.current_entropy, self.current_biomass);
            mix_left += 0.3 * sample;
            mix_right += 0.3 * sample;
        }

        if self.bio_music.is_playing() {
            if let Some(s) = self.bio_music.next_sample() {
                mix_left += 0.5 * s;
                mix_right += 0.5 * s;
            }
        }

        if let Some(ref sfx) = self.active_event_sfx {
            if self.event_sfx_index < sfx.len() {
                mix_left += 0.8 * self.spatial_sfx_left_gain * sfx[self.event_sfx_index];
                mix_right += 0.8 * self.spatial_sfx_right_gain * sfx[self.event_sfx_index];
                self.event_sfx_index += 1;
            } else {
                self.active_event_sfx = None;
                self.event_sfx_index = 0;
            }
        }

        out_buffer[0] = mix_left;
        out_buffer[1] = mix_right;

        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: AudioEvent) {
        let waveform = self.event_sfx.generate_waveform(event);
        if !waveform.is_empty() {
            self.active_event_sfx = Some(waveform);
            self.event_sfx_index = 0;
        }
    }

    pub fn queue_event(&self, event: AudioEvent) {
        let _ = self.event_queue.send(event);
    }

    pub fn set_spatial_sfx_gain(&mut self, left: f32, right: f32) {
        self.spatial_sfx_left_gain = left;
        self.spatial_sfx_right_gain = right;
    }

    pub fn set_entropy(&mut self, entropy: f32) {
        self.current_entropy = entropy.clamp(0.0, 1.0);
    }

    pub fn set_biomass(&mut self, biomass: f32) {
        self.current_biomass = biomass;
    }

    pub fn enable_background(&mut self, enabled: bool) {
        self.background_running = enabled;
    }

    pub fn load_bio_music(&mut self, notes: &[crate::app::audio::bio_music::Note], tempo_bpm: u8) {
        self.bio_music.load_melody(notes, tempo_bpm);
    }

    pub fn event_queue_sender(&self) -> Sender<AudioEvent> {
        self.event_queue.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_new() {
        let engine = AudioEngine::new();
        assert!(engine.is_ok());
        let engine = engine.unwrap();
        assert_eq!(engine.sample_rate, 44100.0);
        assert_eq!(engine.buffer_size, 256);
    }

    #[test]
    fn test_render_block() {
        let mut engine = AudioEngine::new().unwrap();
        let mut buffer = vec![0.0f32; 256];

        engine.render_block(&mut buffer);

        for sample in buffer {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_set_entropy() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_entropy(1.5);
        assert_eq!(engine.current_entropy, 1.0);

        engine.set_entropy(-0.5);
        assert_eq!(engine.current_entropy, 0.0);

        engine.set_entropy(0.6);
        assert_eq!(engine.current_entropy, 0.6);
    }

    #[test]
    fn test_queue_event() {
        let engine = AudioEngine::new().unwrap();

        engine.queue_event(AudioEvent::Birth);

        drop(engine);
    }
}
