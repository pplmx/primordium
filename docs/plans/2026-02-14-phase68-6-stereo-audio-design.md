# Phase 68.6: Full Stereo Audio Output Implementation

**Date**: 2026-02-14
**Status**: Design
**Priority**: HIGH - Enables spatial audio stereo panning

---

## Problem Statement

Current spatial audio infrastructure (Phase 68 v2.5 + 68.5) implements coordinate tracking and spatial calculation algorithms, but **audio output remains mono**:

- `AudioEngine::render_block(&mut [f32])` generates single-channel samples
- Spatial panning (left/right) and distance attenuation are **calculated but not applied**
- Birth/Death events have position data (x, y) but cannot use it for stereo audio

---

## Current Architecture Analysis

### AudioEvent Definition

```rust
// crates/primordium_core/src/systems/audio.rs
pub enum AudioEvent {
    Birth,           // Entity birth - needs spatial positioning
    Death,           // Entity death - needs spatial positioning
    Metamorphosis,   // Metamorphosis - no spatial needed
    ClimateShift,    // Climate shift - no spatial needed
    NewEra,          // Era change - no spatial needed
    AmbientShift,    // Ambient music shift - no spatial needed
}
```

**Issue**: AudioEvent is a simple C-like enum, cannot carry spatial parameters directly.

### AudioSystem Structure

```rust
// src/app/audio.rs
pub struct AudioSystem {
    pub enabled: bool,
    pub volume: float,
    event_queue: VecDeque<AudioEvent>,  // Mono queue - no spatial params
    engine: Option<engine::AudioEngine>,
    world_width: u16,
    world_height: u16,
    // ...
}
```

### Current Spatial Queue (broken)

```rust
// src/app/audio.rs:171-207
fn queue_event_spatial(&mut self, event: AudioEvent, x: Option<f64>, y: Option<f64>) {
    // Calculates left/right panning
    let (left, right) = SpatialAudio::calculate_stereo_panning(...);

    // Calculates distance attenuation
    let distance = self.calculate_distance(x, y);
    let left_sample = SpatialAudio::apply_distance_attenuation(...);
    let right_sample = SpatialAudio::apply_distance_attenuation(...);

    // BUT: discards spatial params! Only pushes non-spatial event
    self.event_queue.push_back(event);

    // TODO noted in code:
    // TODO: Stereo spatial audio requires engine refactor to support stereo output
    // Current mono audio engine doesn't use spatial samples yet
}
```

**Issue**: Spatial parameters are calculated but immediately discarded because:
1. `event_queue: VecDeque<AudioEvent>` cannot store spatial params
2. `AudioEngine::render_block()` only generates mono samples
3. No mechanism to pass spatial params to the renderer

---

## Design Solution

### Approach: Dual Queue Architecture

Create separate queues for spatial and non-spatial events:

```rust
pub struct AudioSystem {
    // Existing mono queue (for events without spatial positioning)
    event_queue: VecDeque<AudioEvent>,

    // NEW: Spatial queue for events with positioning
    spatial_queue: VecDeque<SpatialAudioEvent>,
    // ...
}
```

#### SpatialAudioEvent Structure

```rust
// New struct to bundle AudioEvent with spatial parameters
pub struct SpatialAudioEvent {
    pub event: AudioEvent,
    pub left_pan: f32,          // 0.0-1.0 (left channel gain)
    pub right_pan: f32,         // 0.0-1.0 (right channel gain)
    pub distance: f64,          // Distance from world center
    pub max_distance: f64,      // For attenuation normalization
}

impl SpatialAudioEvent {
    pub fn apply_stereo(&self, mono_sample: f32) -> (f32, f32) {
        let distance_attenuation = 1.0 / (1.0 + (self.distance / self.max_distance).powi(2));
        let attenuated = mono_sample * distance_attenuation as f32;

        let left = attenuated * self.left_pan;
        let right = attenuated * self.right_pan;
        (left, right)
    }
}
```

### AudioEngine Stereo Output

```rust
// src/app/audio/engine.rs
impl AudioEngine {
    // NEW: Stereo render method
    pub fn render_block_stereo(&mut self, out_buffer: &mut [f32; 2]) {
        let mut mix_left = 0.0;
        let mut mix_right = 0.0;

        // Entropy synth (center-panned, no attenuation)
        if self.background_running {
            let sample = self.entropy_synth.render_sample(...);
            mix_left += 0.3 * sample;
            mix_right += 0.3 * sample;
        }

        // Bio-music (center-panned, no attenuation)
        if self.bio_music.is_playing() {
            if let Some(s) = self.bio_music.next_sample() {
                mix_left += 0.5 * s;
                mix_right += 0.5 * s;
            }
        }

        // Event SFX (mono, applies spatial panning externally)
        if let Some(ref sfx) = self.active_event_sfx {
            if self.event_sfx_index < sfx.len() {
                mix_left += 0.8 * sfx[self.event_sfx_index];
                mix_right += 0.8 * sfx[self.event_sfx_index];
                self.event_sfx_index += 1;
            } else {
                self.active_event_sfx = None;
                self.event_sfx_index = 0;
            }
        }

        out_buffer[0] = mix_left;
        out_buffer[1] = mix_right;
    }

    // KEEP: Existing mono render for backward compatibility
    pub fn render_block(&mut self, out_buffer: &mut [f32]) {
        for sample in out_buffer.iter_mut() {
            let mut mix = 0.0;
            // ... existing mono logic ...
            *sample = mix;
        }
    }
}
```

### AudioSystem Integration

```rust
// src/app/audio.rs
impl AudioSystem {
    fn queue_event_spatial(&mut self, event: AudioEvent, x: Option<f64>, y: Option<f64>) {
        if let (Some(x), Some(y)) = (x, y) {
            let (left_pan, right_pan) = SpatialAudio::calculate_stereo_panning(...);
            let distance = self.calculate_distance(x, y);
            let max_distance = self.max_distance();

            let spatial_event = SpatialAudioEvent {
                event,
                left_pan,
                right_pan,
                distance,
                max_distance,
            };

            if self.enabled {
                self.spatial_queue.push_back(spatial_event);
            }
        } else {
            self.queue_event(event);  // Non-spatial path
        }
    }

    pub fn process_queue(&mut self) {
        if !self.enabled {
            self.event_queue.clear();
            self.spatial_queue.clear();
            return;
        }

        if let Some(engine) = &mut self.engine {
            // Process spatial events with stereo output
            while let Some(spatial_ev) = self.spatial_queue.pop_front() {
                let mut buffer = [0.0_f32; 2];
                engine.render_block_stereo(&mut buffer);

                // Apply spatial panning to Event SFX only
                if matches!(spatial_ev.event, AudioEvent::Birth | AudioEvent::Death) {
                    engine.queue_event_sfx(spatial_ev.event);

                    // Wait for SFX to be generated, then apply panning
                    // ... integration point for spatial SFX rendering
                }
            }

            // Non-spatial events (center-panned)
            for event in self.event_queue.drain(..) {
                engine.handle_event(event);
            }

            // Update entropy params
            engine.set_entropy(self.current_entropy);
            engine.set_biomass(self.current_biomass);
            engine.enable_background(true);
        }
    }
}
```

---

## Implementation Phases

### Phase 68.6.1: Create SpatialAudioEvent struct (1 commit)

**File**: `src/app/audio.rs` (add after AudioSystem definition)

```rust
pub struct SpatialAudioEvent {
    pub event: AudioEvent,
    pub left_pan: f32,
    pub right_pan: f32,
    pub distance: f64,
    pub max_distance: f64,
}
```

Add `spatial_queue: VecDeque<SpatialAudioEvent>` to AudioSystem struct.

### Phase 68.6.2: Refactor queue_event_spatial (1 commit)

**File**: `src/app/audio.rs:171-207`

Remove broken spatial sample generation code. Create SpatialAudioEvent and push to spatial_queue.

### Phase 68.6.3: Add stereo render to AudioEngine (1 commit)

**File**: `src/app/audio/engine.rs:59-92`

Add `render_block_stereo(&mut self, out_buffer: &mut [f32; 2])` method.
Keep existing `render_block(&mut self, &mut [f32])` for backward compatibility.

Add `active_spatial_sfx: Option<(AudioEvent, SpatialAudioParams)>` to track spatial Event SFX.

### Phase 68.6.4: Update process_queue to handle spatial events (1 commit)

**File**: `src/app/audio.rs:218-237`

Process spatial_queue with stereo rendering, apply panning to Birth/Death events.

---

## Risk Assessment

### Low Risk

- **Backward Compatibility**: Keep mono `render_block()` for non-spatial use cases
- **Gradual Migration**: Two-tier architecture allows incremental adoption
- **Pattern Matching**: All existing code uses mono render_block initially

### Medium Risk

- **Audio Complexity**: Stereo rendering requires careful mixing to prevent clipping
  - **Mitigation**: Use soft clipping (tanh) and master volume control
- **Queue Coordination**: Two queues must be processed in correct order
  - **Mitigation**: Process spatial_queue first, then event_queue

### Testing Strategy

1. **Unit Tests**: Verify SpatialAudioEvent::apply_stereo() calculations
2. **Integration Tests**: Confirm stereo output has correct channel separation
3. **Performance Test**: Benchmark stereo vs mono overhead (<50% expected)
4. **Regression Tests**: All existing audio tests still pass

---

## Success Criteria

- ✅ SpatialAudioEvent struct created and used
- ✅ Stere

o output generated with correct channel separation
- ✅ Birth events have left/right panning based on x position
- ✅ Death events have left/right panning and distance attenuation
- ✅ All existing mono audio still works (center-panned)
- ✅ Performance overhead <50% compared to mono
- ✅ All tests pass, no clipping in output

---

## Future Enhancements (Post-68.6)

1. **Doppler Effect**: Pitch shift for moving entities based on velocity
2. **Reverb**: Schroeder reverb for environmental depth
3. **Dynamic EQ**: Filter frequencies based on ecosystem state
4. **Head-Related Transfer Function (HRTF)**: True 3D audio positioning

---

## Estimated Effort

| Phase | Description | Estimate |
|-------|-------------|----------|
| 68.6.1 | Create SpatialAudioEvent, add spatial_queue | 30 min |
| 68.6.2 | Refactor queue_event_spatial | 15 min |
| 68.6.3 | Add render_block_stereo to AudioEngine | 45 min |
| 68.6.4 | Update process_queue for stereo rendering | 45 min |
| Testing & Verification | | 30 min |
| **Total** | | **~2.5 hours, 4 commits** |

---

## References

- Phase 68 v2 Design: `docs/plans/2026-02-13-phase68-v2-algorithmic-audio-design.md`
- Phase 68.5 Design: `docs/plans/2026-02-14-phase68-5-entity-coordinates-design.md`
- Spatial Audio Implementation: `src/app/audio/spatial.rs`
- Audio Engine: `src/app/audio/engine.rs`
