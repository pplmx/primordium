use primordium_core::systems::audio::AudioEvent;
use primordium_data::data::environment::LiveEvent;
use std::collections::VecDeque;

#[cfg(all(
    feature = "rodio",
    not(target_arch = "wasm32"),
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
mod rodio_driver {
    use super::*;
    use async_trait::async_trait;
    use primordium_core::systems::audio::{AudioDriver, AudioSpatialParams};
    use std::sync::Arc;

    pub struct RodioAudioDriver {
        _stream_handle: Arc<rodio::OutputStream>,
        sink: rodio::Sink,
        volume: f32,
    }

    impl RodioAudioDriver {
        pub fn new() -> Result<Self, String> {
            let (stream_handle, stream) = rodio::OutputStream::try_default()
                .map_err(|e| format!("Failed to create audio stream: {}", e))?;

            let sink = rodio::Sink::try_new(&stream_handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

            std::mem::forget(stream);

            Ok(Self {
                _stream_handle: Arc::new(stream_handle),
                sink,
                volume: 0.5,
            })
        }

        fn synth_wave(&self, frequency: f32, duration: f64) -> rodio::Source {
            let sample_rate = 44100;
            let samples = (duration * sample_rate as f64) as usize;
            let source = rodio::source::SineWave::new(frequency)
                .take_duration(std::time::Duration::from_secs_f64(duration));

            source
        }
    }

    #[async_trait]
    impl AudioDriver for RodioAudioDriver {
        async fn play_effect(&self, event: AudioEvent, params: Option<AudioSpatialParams>) {
            let (frequency, duration, vol) = match event {
                AudioEvent::Birth => (880.0, 0.2, 0.3),
                AudioEvent::Death => (220.0, 0.3, 0.4),
                AudioEvent::Metamorphosis => (660.0, 0.25, 0.35),
                AudioEvent::ClimateShift => (440.0, 0.5, 0.3),
                AudioEvent::NewEra => (1100.0, 0.4, 0.5),
                AudioEvent::AmbientShift => (330.0, 0.3, 0.2),
            };

            let source = self
                .synth_wave(frequency, duration)
                .amplify(vol * self.volume);

            let sink = rodio::Sink::new(&self._stream_handle);
            sink.append(source);
            sink.detach();
        }

        async fn update_ambient(&self, entropy: f32, biomass: f64) {
            let frequency = 200.0 + entropy * 300.0;
            let duration = 1.0 / (1.0 + biomass * 0.001);

            let source = self
                .synth_wave(frequency, duration)
                .amplify(0.1 * self.volume)
                .repeat_infinite();

            self.sink.try_append(source).ok();
        }

        fn set_volume(&self, volume: f32) {}
    }
}

#[cfg(all(
    feature = "rodio",
    not(target_arch = "wasm32"),
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
pub use rodio_driver::RodioAudioDriver;

pub struct AudioSystem {
    pub enabled: bool,
    pub volume: f32,
    event_queue: VecDeque<AudioEvent>,
    #[cfg(all(
        feature = "rodio",
        not(target_arch = "wasm32"),
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    ))]
    rodio_driver: Option<RodioAudioDriver>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            enabled: false,
            volume: 0.5,
            event_queue: VecDeque::with_capacity(32),
            #[cfg(all(
                feature = "rodio",
                not(target_arch = "wasm32"),
                any(target_os = "linux", target_os = "macos", target_os = "windows")
            ))]
            rodio_driver: None,
        }
    }
}

impl AudioSystem {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable audio output
    pub fn enable(&mut self) {
        if self.enabled {
            return;
        }

        #[cfg(all(
            feature = "rodio",
            not(target_arch = "wasm32"),
            any(target_os = "linux", target_os = "macos", target_os = "windows")
        ))]
        {
            if self.rodio_driver.is_none() {
                match RodioAudioDriver::new() {
                    Ok(driver) => {
                        self.rodio_driver = Some(driver);
                        tracing::info!("Audio enabled with rodio backend");
                    }
                    Err(e) => {
                        tracing::warn!("Failed to initialize audio: {}", e);
                        return;
                    }
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

    pub fn process_live_event(&mut self, event: &LiveEvent) {
        match event {
            LiveEvent::Birth { .. } => self.queue_event(AudioEvent::Birth),
            LiveEvent::Death { .. } => self.queue_event(AudioEvent::Death),
            LiveEvent::Metamorphosis { .. } => self.queue_event(AudioEvent::Metamorphosis),
            LiveEvent::ClimateShift { .. } => self.queue_event(AudioEvent::ClimateShift),
            LiveEvent::TribalSplit { .. } => self.queue_event(AudioEvent::Birth),
            LiveEvent::Snapshot { .. } | LiveEvent::Narration { .. } => {
                self.queue_event(AudioEvent::AmbientShift)
            }
            LiveEvent::Extinction { .. } | LiveEvent::EcoAlert { .. } => {
                self.queue_event(AudioEvent::AmbientShift)
            }
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
    fn play_event(&self, event: &AudioEvent) {
        #[cfg(all(
            feature = "rodio",
            not(target_arch = "wasm32"),
            any(target_os = "linux", target_os = "macos", target_os = "windows")
        ))]
        {
            if let Some(driver) = &self.rodio_driver {
                let event = *event;
                let _ = tokio::spawn(async move {
                    AudioDriver::play_effect(driver, event, None).await;
                });
            }
        }

        #[cfg(not(all(
            feature = "rodio",
            not(target_arch = "wasm32"),
            any(target_os = "linux", target_os = "macos", target_os = "windows")
        )))]
        {
            tracing::debug!("Audio event: {:?}", event);
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

        // Queue should be processed in order
        audio.process_queue();
        // No panic = success for placeholder
    }
}
