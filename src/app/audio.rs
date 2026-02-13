use primordium_core::systems::audio::{AudioDriver, AudioEvent, NullAudioDriver};
use std::collections::VecDeque;

/// Audio system for Primordium TUI
///
/// This is a placeholder for Phase 68 Audio implementation.
/// Currently provides the infrastructure; actual audio playback
/// will be implemented in Phase 68.
#[allow(dead_code)]
pub struct AudioSystem {
    pub enabled: bool,
    pub volume: f32,
    driver: Option<Box<dyn AudioDriver>>,
    /// Queue of pending audio events
    event_queue: VecDeque<AudioEvent>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.5,
            driver: Some(Box::new(NullAudioDriver)),
            event_queue: VecDeque::with_capacity(32),
        }
    }
}

impl AudioSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable audio output
    pub fn enable(&mut self) {
        self.enabled = true;
        tracing::info!("Audio enabled");
    }

    /// Disable audio output
    pub fn disable(&mut self) {
        self.enabled = false;
        tracing::info!("Audio disabled");
    }

    /// Toggle audio on/off
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        tracing::info!(
            "Audio {}",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    /// Set volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Queue an audio event for playback
    pub fn queue_event(&mut self, event: AudioEvent) {
        if self.enabled {
            self.event_queue.push_back(event);
        }
    }

    /// Process all queued audio events
    ///
    /// Call this once per frame to play queued sounds
    pub fn process_queue(&mut self) {
        if !self.enabled {
            self.event_queue.clear();
            return;
        }

        while let Some(event) = self.event_queue.pop_front() {
            self.play_event(&event);
        }
    }

    /// Play a single audio event immediately
    fn play_event(&self, _event: &AudioEvent) {
        // Placeholder: In Phase 68, this will actually play sound
        // For now, just log the event
        tracing::debug!("Audio event: {:?}", _event);
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

        // Queue should be processed in order
        audio.process_queue();
        // No panic = success for placeholder
    }
}
