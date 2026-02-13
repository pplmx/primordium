//! Audio system abstraction for Primordium.
//!
//! Provides traits and types for triggering spatial audio and bio-music
//! based on simulation events.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Categories of audio events that can be triggered.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioEvent {
    /// Entity birth event.
    Birth,
    /// Entity death/predation event.
    Death,
    /// Metamorphosis transformation.
    Metamorphosis,
    /// Global climate shift.
    ClimateShift,
    /// New era beginning.
    NewEra,
    /// Background ambient music shift.
    AmbientShift,
}

/// Parameters for spatial audio positioning.
#[derive(Debug, Clone, Copy)]
pub struct AudioSpatialParams {
    /// X coordinate (0.0 to 1.0).
    pub x: f32,
    /// Y coordinate (0.0 to 1.0).
    pub y: f32,
    /// Relative volume intensity.
    pub intensity: f32,
}

/// Hardware-independent trait for audio drivers.
#[async_trait]
pub trait AudioDriver: Send + Sync {
    /// Plays a one-shot sound effect.
    async fn play_effect(&self, event: AudioEvent, params: Option<AudioSpatialParams>);

    /// Updates the background procedural music state.
    async fn update_ambient(&self, entropy: f32, biomass: f64);

    /// Sets the master volume (0.0 to 1.0).
    fn set_volume(&self, volume: f32);
}

/// Null audio driver that discards all commands.
pub struct NullAudioDriver;

#[async_trait]
impl AudioDriver for NullAudioDriver {
    async fn play_effect(&self, _event: AudioEvent, _params: Option<AudioSpatialParams>) {}
    async fn update_ambient(&self, _entropy: f32, _biomass: f64) {}
    fn set_volume(&self, _volume: f32) {}
}
