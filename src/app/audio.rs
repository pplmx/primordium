pub mod bio_music;
pub mod bio_music_algorithm;
pub mod engine;
pub mod entropy_synth;
pub mod event_sfx;
pub mod spatial;

use primordium_core::systems::audio::AudioEvent;
use primordium_data::data::environment::LiveEvent;
use primordium_data::data::genotype::Genotype;
use std::collections::VecDeque;
use uuid::Uuid;

pub struct AudioSystem {
    pub enabled: bool,
    pub volume: f32,
    event_queue: VecDeque<AudioEvent>,
    engine: Option<engine::AudioEngine>,
    current_entropy: f32,
    current_biomass: f32,
    top_lineage_id: Option<Uuid>,
    top_lineage_genotype: Option<Genotype>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.5,
            event_queue: VecDeque::with_capacity(32),
            engine: None,
            current_entropy: 0.5,
            current_biomass: 1000.0_f32,
            top_lineage_id: None,
            top_lineage_genotype: None,
        }
    }
}

impl AudioSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable(&mut self) {
        if self.enabled {
            return;
        }

        if self.engine.is_none() {
            match engine::AudioEngine::new() {
                Ok(engine) => {
                    self.engine.replace(engine);
                    tracing::info!("Audio enabled with pure Rust engine");
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize audio: {}", e);
                    return;
                }
            }
        }

        self.enabled = true;
        tracing::info!("Audio enabled");
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        tracing::info!("Audio disabled");
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        tracing::info!(
            "Audio {}",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn queue_event(&mut self, event: AudioEvent) {
        if self.enabled {
            self.event_queue.push_back(event);
        }
    }

    pub fn process_live_event_with_position(
        &mut self,
        event: &LiveEvent,
        x: Option<f64>,
        y: Option<f64>,
    ) {
        #[allow(unused_variables)]
        let _ = (x, y);

        match event {
            LiveEvent::Birth { .. } => self.queue_event(AudioEvent::Birth),
            LiveEvent::Death { .. } => self.queue_event(AudioEvent::Death),
            LiveEvent::Metamorphosis { .. } => self.queue_event(AudioEvent::Metamorphosis),
            LiveEvent::ClimateShift { .. } => self.queue_event(AudioEvent::ClimateShift),
            LiveEvent::TribalSplit { .. } => self.queue_event(AudioEvent::Birth),
            LiveEvent::Snapshot { stats, .. } => {
                self.queue_event(AudioEvent::AmbientShift);
                self.update_entropy_parameters(
                    stats.avg_brain_entropy,
                    stats.biomass_h + stats.biomass_c,
                );
            }
            LiveEvent::Narration { .. } => {
                self.queue_event(AudioEvent::AmbientShift);
            }
            LiveEvent::Extinction { .. } | LiveEvent::EcoAlert { .. } => {
                self.queue_event(AudioEvent::AmbientShift)
            }
        }
    }

    pub fn process_live_event(&mut self, event: &LiveEvent) {
        self.process_live_event_with_position(event, None, None);
    }

    fn update_entropy_parameters(&mut self, avg_brain_entropy: f64, total_biomass: f64) {
        self.current_entropy = avg_brain_entropy.clamp(0.0, 1.0) as f32;
        self.current_biomass = total_biomass as f32;
    }

    pub fn update_top_lineage_genotype(
        &mut self,
        lineage_id: Uuid,
        genotype: &Genotype,
        #[allow(unused_variables)] world_tick: u64,
    ) {
        let is_new_top = match self.top_lineage_id {
            Some(id) => id != lineage_id,
            None => true,
        };

        if is_new_top {
            self.top_lineage_id = Some(lineage_id);
            self.top_lineage_genotype = Some(genotype.clone());
            self.generate_bio_music(genotype);
        }
    }

    fn generate_bio_music(&mut self, genotype: &Genotype) {
        if !self.enabled {
            return;
        }

        let melody = self::bio_music_algorithm::BioMusicAlgorithm::genotype_to_melody(genotype);
        let tempo_bpm = self::bio_music_algorithm::BioMusicAlgorithm::extract_tempo(genotype);

        if let Some(engine) = &mut self.engine {
            engine.load_bio_music(&melody, tempo_bpm);
        }
    }

    pub fn process_queue(&mut self) {
        if !self.enabled {
            self.event_queue.clear();
            return;
        }

        if let Some(engine) = &mut self.engine {
            engine.set_entropy(self.current_entropy);
            engine.set_biomass(self.current_biomass);
            engine.enable_background(true);

            while let Some(event) = self.event_queue.pop_front() {
                engine.queue_event(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_system_default() {
        let audio = AudioSystem::new();
        assert!(!audio.enabled);
        assert_eq!(audio.volume, 0.5);
    }

    #[test]
    fn test_audio_system_toggle() {
        let mut audio = AudioSystem::new();
        assert!(!audio.enabled);

        audio.toggle();
        assert!(audio.enabled);

        audio.toggle();
        assert!(!audio.enabled);
    }

    #[test]
    fn test_audio_system_volume_clamping() {
        let mut audio = AudioSystem::new();

        audio.set_volume(1.5);
        assert_eq!(audio.volume, 1.0);

        audio.set_volume(-0.5);
        assert_eq!(audio.volume, 0.0);

        audio.set_volume(0.75);
        assert_eq!(audio.volume, 0.75);
    }

    #[test]
    fn test_audio_event_queue() {
        let mut audio = AudioSystem::new();
        audio.enable();

        audio.queue_event(AudioEvent::Birth);
        audio.queue_event(AudioEvent::Death);

        audio.process_queue();
    }
}
