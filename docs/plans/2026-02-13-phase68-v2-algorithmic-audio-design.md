# Phase 68 v2: The Song of Entropy - Algorithmic Audio Design

**Date**: 2026-02-13
**Version**: 2.0
**Status**: Design
**Goal**: Replace simple sine wave placeholder with sophisticated procedural audio generation

---

## Overview

Phase 68 v2 implements three advanced audio systems that transform the Primordium simulation state into immersive sonic landscapes:

1. **Entropy Synth**: FM synthesis driven by global neural entropy
2. **Bio-Music**: Genotype-to-melody encoding for dominant lineages
3. **Event SFX**: Procedural sound effects with spatial positioning

All systems use pure Rust implementation with no external audio dependencies (cpal/ALSA removed).

---

## Architecture

### Module Structure

```
src/app/audio/
├── mod.rs              # Public API, AudioSystem orchestrator
├── engine.rs           # Pure Rust audio engine (wavetable + FM)
├── entropy_synth.rs    # Entropy-driven ambient synthesis
├── bio_music.rs        # Genome-to-melody translation
├── event_sfx.rs        # Procedural event sound effects
└── spatial.rs          # 3D audio positioning for events
```

### Data Flow

```
World::update() -> Vec<LiveEvent>
    ├─> EntropyPopulator (background): avg_brain_entropy, biomass
    ├─> BioMusicGenerator (background): Top1 lineage genotype
    └─> EventSFXGenerator (real-time): Birth/Death/Predation

AudioSystem::process_queue()
    ├─> EntropySynth::update_parameters(entropy, biomass)
    ├─> BioMusic::queue_sequence(lineage_genotype)
    └─> EventSFX::play(event, spatial_params)

AudioEngine::render_next_block()
    ├─> Mix EntropySynth (ambient)
    ├─> Mix BioMusic (melody, when active)
    └─> Mix EventSFX (one-shot)
```

---

## Algorithm 1: Entropy Synth

### Purpose

Generate continuous ambient soundscape that reflects the stability/chaos of the ecosystem through neural network entropy.

### Input Parameters

- `avg_brain_entropy`: Shannon entropy of all neural weight distributions (0.0-1.0)
- `biomass`: Total biological mass (0.0-∞)
- `carbon_level`: Atmospheric carbon (0.0-∞)
- `population`: Number of living entities

### FM Synthesis Algorithm

```rust
pub struct FMSynthesizer {
    carrier_phase: f32,
    modulator_phase: f32,
    sample_rate: f32,
}

impl FMSynthesizer {
    pub fn new(sample_rate: f32) -> Self;

    /// Generate next sample with current entropy parameters
    pub fn render_sample(&mut self, entropy: f32, biomass: f32) -> f32 {
        // Carrier: base frequency modulated by entropy
        let carrier_freq = 200.0 + (entropy * 500.0);  // 200-700 Hz
        let carrier = self.sine_wave(&mut self.carrier_phase, carrier_freq);

        // Modulator: richness controlled by biomass
        let modulator_freq = carrier_freq * (1.0 + entropy);
        let modulation_index = biomass * 0.001;
        let modulator = modulation_index * self.sine_wave(&mut self.modulator_phase, modulator_freq);

        // FM synthesis
        carrier * modulator.sin()
    }

    fn sine_wave(&mut self, phase: &mut f32, frequency: f32) -> f32 {
        *phase += frequency / self.sample_rate;
        *phase %= 1.0;
        (*phase * 2.0 * std::f32::consts::PI).sin()
    }
}
```

### Entropy-to-Sound Mapping

| Entropy Range | Carrier Freq | Modulation Index | Character |
|--------------|-------------|------------------|-----------|
| 0.0 - 0.2 | 200-300 Hz | Low (0.1-0.3) | Pure, stable, harmonic |
| 0.2 - 0.5 | 300-400 Hz | Medium (0.3-0.6) | Balanced, evolving |
| 0.5 - 0.8 | 400-600 Hz | High (0.6-0.9) | Complex, rich harmonics |
| 0.8 - 1.0 | 600-700 Hz | Very High (0.9-1.2) | Chaotic, unstable |

---

## Algorithm 2: Bio-Music

### Purpose

Convert dominant lineage genotypes into recognizable melodic motifs, creating "theme songs" for each civilization.

### Input Parameters

- `genotype`: Genotype of Top 1 fitness entity
  - Physical genes: `sensing_range`, `max_speed`, `max_energy`, `metabolic_niche`, `trophic_potential`
  - Neural genes: `Brain` (graph with weights)

### Encoding Algorithm

#### Step 1: Physical Genes → Musical Foundation

```rust
pub struct BioMusicGenerator {
    scale: Scale,           // Major/Minor/Chromatic
    base_pitch: u8,        // MIDI pitch 0-127
    tempo_bpm: u8,         // 60-120 BPM
}

impl BioMusicGenerator {
    pub fn from_genotype(genotype: &Genotype) -> Self {
        // Sensing range determines tonal center
        let base_pitch = (genotype.sensing_range * 10.0) as u8 % 12;

        // Speed determines tempo
        let tempo_bpm = 60 + (genotype.max_speed * 60.0) as u8;

        // Niche determines scale type
        let scale = if genotype.metabolic_niche < 0.5 {
            Scale::Major
        } else {
            Scale::Minor
        };

        Self { scale, base_pitch, tempo_bpm }
    }
}
```

#### Step 2: Neural Weights → Melodic Phrase

```rust
pub fn encode_genotype_to_melody(genotype: &Genotype) -> Vec<Note> {
    let mut melody = Vec::with_capacity(16);
    let scale = Scale::from_genotype(genotype);

    // Sample representative weights from each layer
    for layer in genotype.brain.layers() {
        let representative_weight = extract_representative_weight(layer);

        // Map weight to pitch (2-octave range, 24 semitones)
        let pitch_offset = (representative_weight.abs() * 24.0) as i8 - 12;

        // Map layer complexity to note duration
        let duration = match layer.complexity() {
            Complexity::Low => NoteDuration::Quarter,
            Complexity::Medium => NoteDuration::Eighth,
            Complexity::High => NoteDuration::Sixteenth,
        };

        melody.push(Note {
            pitch: scale.quantize(base_pitch + pitch_offset),
            duration,
            velocity: 0.7,
        });
    }

    melody
}
```

#### Step 3: Connection Weights → Harmonic Progression

```rust
pub fn derive_chords_from_connections(genotype: &Genotype) -> Vec<Chord> {
    let mut chords = Vec::new();
    let scale = Scale::from_genotype(genotype);

    // Analyze inter-layer connection strength patterns
    for connection_group in genotype.brain.connections().chunks(4) {
        let avg_strength = connection_group.iter()
            .map(|c| c.strength.abs())
            .sum::<f32>() / 4.0;

        // Map strength to chord quality (0-7 diatonic chords)
        let chord_degree = (avg_strength * 7.0) as u8 % 7;
        chords.push(scale.get_chord(chord_degree));
    }

    chords
}
```

### Playback Strategy

- **Trigger**: When a lineage becomes Top 1 in Hall of Fame
- **Duration**: 8 ~ 16小节的主题旋律 (~15-30秒)
- **Timbre**: Sine wave with ADSR envelope (soft attack, medium decay)
- **Volume**: 0.5 (blended with EntropySynth at 0.3)

---

## Algorithm 3: Event SFX (Procedural Sound Effects)

### Purpose

Generate non-repetitive, context-aware sounds for birth, death, predation, and climate events.

### Waveform Generation

```rust
pub struct EventSFXGenerator {
    sample_rate: f32,
}

impl EventSFXGenerator {
    /// Generate procedural sound for an event
    pub fn generate_waveform(&self, event: AudioEvent) -> Vec<f32> {
        match event {
            AudioEvent::Birth => self.birth_waveform(),
            AudioEvent::Death => self.death_waveform(),
            AudioEvent::Predation => self.predation_waveform(),
            AudioEvent::ClimateShift => self.climate_waveform(),
            _ => vec![],
        }
    }

    fn birth_waveform(&self) -> Vec<f32> {
        // Rising sine sweep with harmonics
        let duration = 0.3;  // seconds
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count;
            let frequency = 440.0 + (t * 880.0);  // 440-1320 Hz sweep

            // Fundamental + 3rd harmonic
            let sample = (2.0 * std::f32::consts::PI * frequency * t).sin()
                       + 0.5 * (2.0 * std::f32::consts::PI * frequency * 3.0 * t).sin();

            // ADSR envelope
            let envelope = match t {
                t if t < 0.1 => t / 0.1,                    // Attack
                t if t < 0.2 => 1.0 - (t - 0.1) / 0.1,      // Decay
                _ => 0.8 * (1.0 - t) / 0.1,                // Release
            };

            waveform.push(sample * envelope);
        }

        waveform
    }

    fn death_waveform(&self) -> Vec<f32> {
        // Falling FM-noise hybrid
        let duration = 0.5;
        let sample_count = (duration * self.sample_rate) as usize;
        let mut waveform = Vec::with_capacity(sample_count);

        for i in 0..sample_count {
            let t = i as f32 / sample_count;
            let carrier_freq = 220.0 * (1.0 - t);  // 220->0 Hz fall
            let noise = (rand::random::<f32>() - 0.5) * 2.0;

            let sample = (2.0 * std::f32::consts::PI * carrier_freq * t).sin()
                       + 0.3 * noise;

            let envelope = (1.0 - t).powf(2.0);
            waveform.push(sample * envelope);
        }

        waveform
    }
}
```

### Event-Sound Mapping

| Event | Waveform | Frequency Range | Duration | Character |
|-------|----------|----------------|----------|-----------|
| Birth | Sine sweep ↑ | 440-1320 Hz | 0.3s | Bright, ascendant |
| Death | FM+Noise ↓ | 220-0 Hz | 0.5s | Low, dissonant |
| Predation | Pluck+Impulse | 880-1760 Hz | 0.15s | Sharp, percussive |
| ClimateShift | Filter sweep | 100-1000 Hz | 1.0s | Brooding, evolving |
| Metamorphosis | Chord arpeggio ↑ | 330-660 Hz | 0.4s | Ascending, magical |

---

## Spatial Audio (3D Positioning)

### Purpose

Position event sounds in 3D space based on entity location on the 2D world grid.

### Simple Stereo Panning

```rust
pub fn calculate_stereo_panning(world_x: f64, world_y: f64, world_width: u16, world_height: u16) -> (f32, f32) {
    // Normalize world position to 0-1
    let norm_x = world_x / world_width as f64;
    let norm_y = world_y / world_height as f64;

    // X-axis controls left/right balance (0=left, 1=right)
    let pan = (norm_x - 0.5) * 2.0;  // -1.0 to 1.0

    // Constant power panning
    let angle = pan * std::f32::consts::FRAC_PI_4;  // ±45 degrees
    let left = (angle.cos() - angle.sin()) * 0.707;
    let right = (angle.cos() + angle.sin()) * 0.707;

    (left, right)
}
```

### Distance Attenuation

```rust
pub fn apply_distance_attenuation(sample: f32, distance: f32, max_distance: f64) -> f32 {
    let normalized_distance = (distance / max_distance).min(1.0);
    // Inverse square law with soft knee
    let attenuation = 1.0 / (1.0 + normalized_distance * normalized_distance);
    sample * attenuation
}
```

---

## Pure Rust Audio Engine

### Design Principles

- **No external C dependencies**: Eliminate cpal/ALSA/portaudio requirement
- **Cross-platform platform**: Linux/macOS/Windows via std::audio (custom implementation)
- **Low latency**: 64-256 sample buffers
- **Efficient**: SIMD-friendly generation

### Engine Architecture

```rust
pub struct AudioEngine {
    sample_rate: f32,
    buffer_size: usize,
    entropy_synth: FMSynthesizer,
    bio_music: WavetablePlayer,
    event_sfx: EventSFXGenerator,
    current_entropy: f32,
    current_biomass: f64,
    background_running: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, AudioError> {
        Ok(Self {
            sample_rate: 44100.0,
            buffer_size: 256,
            entropy_synth: FMSynthesizer::new(44100.0),
            bio_music: WavetablePlayer::new(),
            event_sfx: EventSFXGenerator::new(44100.0),
            current_entropy: 0.0,
            current_biomass: 0.0,
            background_running: false,
        })
    }

    /// Render next audio block (64-256 samples)
    pub fn render_block(&mut self, out_buffer: &mut [f32]) {
        for sample in out_buffer.iter_mut() {
            // Mix all sources
            let mut mix = 0.0;

            // 1. Entropy synth ambient (always running if enabled)
            if self.background_running {
                mix += 0.3 * self.entropy_synth.render_sample(
                    self.current_entropy,
                    self.current_biomass as f32
                );
            }

            // 2. Bio-music melody (when active)
            match self.bio_music.next_sample() {
                Some(s) => mix += 0.5 * s,
                None => {}
            }

            // 3. Event SFX (one-shot)
            match self.event_sfx.next_sample() {
                Some(s) => mix += 0.8 * s,
                None => {}
            }

            *sample = mix;
        }
    }
}
```

### Platform-Specific Output

#### Platform Abstraction

```rust
pub trait AudioOutput: Send + Sync {
    fn write_samples(&mut self, samples: &[f32]) -> Result<(), AudioError>;
    fn set_sample_rate(&mut self, rate: f32);
}

// Linux: PulseAudio via std::process (pipe to pacat)
pub struct PulseAudioOutput {
    process: Option<std::process::Child>,
    rate: f32,
}

impl AudioOutput for PulseAudioOutput {
    fn write_samples(&mut self, samples: &[f32]) -> Result<(), AudioError> {
        // Convert f32 to i16 PCM and write to stdin
        // Implementation details omitted for brevity
    }
}
```

---

## Integration with AudioSystem

### Updated AudioSystem API

```rust
pub struct AudioSystem {
    enabled: bool,
    volume: f32,
    event_queue: VecDeque<AudioEvent>,

    // Pure Rust engine (no Box<dyn AudioDriver>)
    engine: Option<AudioEngine>,

    // Background parameters for entropy synth
    current_entropy: f32,
    current_biomass: f64,

    // Bio-music caching
    top_lineage_melody: Option<Vec<Note>>,
}

impl AudioSystem {
    pub fn process_live_event(&mut self, event: &LiveEvent) {
        if !self.enabled {
            return;
        }

        match event {
            LiveEvent::Birth { .. } => self.queue_event(AudioEvent::Birth),
            LiveEvent::Death { .. } => self.queue_event(AudioEvent::Death),
            // ... other mappings

            // Update entropy parameters from PopulationStats
            LiveEvent::Snapshot { stats, .. } => {
                self.current_entropy = stats.avg_brain_entropy;
                self.current_biomass = stats.biomass_h + stats.biomass_c;

                // Check if new top lineage -> generate Bio-Music
                if self.check_new_top_lineage(stats) {
                    self.generate_bio_music();
                }
            }
            _ => {}
        }
    }

    pub fn process_queue(&mut self) {
        if !self.enabled {
            self.event_queue.clear();
            return;
        }

        if let Some(engine) = &mut self.engine {
            // Update entropy synth parameters
            engine.set_entropy(self.current_entropy);
            engine.set_biomass(self.current_biomass);
            engine.enable_background(true);

            // Queue event SFX
            for event in self.event_queue.drain(..) {
                engine.queue_event_sfx(event);
            }
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

1. **Entropy Synth**: Verify entropy-to-frequency mapping
2. **Bio-Music**: Test deterministic genotype → melody encoding
3. **Event SFX**: Validate waveform generation characteristics
4. **Spatial Audio**: Test panning and attenuation formulas

### Integration Tests

1. **End-to-End**: LiveEvent → AudioEvent → Sample generation
2. **Performance**: Benchmark sample generation < 100μs per frame
3. **Memory**: Leak detection with long-running simulations

### Property-Based Tests

1. **Monotonicity**: Higher entropy always results in higher carrier frequency
2. **Boundedness**: All generated samples within [-1.0, 1.0]
3. **Determinism**: Same genotype always produces same melody

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Render block (256 samples) | < 50 μs | 44.1kHz / 256 ≈ 172 Hz update rate |
| Memory per AudioEngine | < 1 MB | Minimal state machine |
| Event SFX generation | < 10 μs | Pre-cached waveforms where possible |
| Bio-music melody gen | < 1 ms | Cached for lineage lifetime |

---

## Implementation Plan

### Phase 1: Core Audio Engine (2-3 commits)
- [ ] Implement pure Rust `AudioEngine` with output abstraction
- [ ] Implement `FMSynthesizer` for Entropy Synth
- [ ] Implement `WavetablePlayer` for Bio-Music
- [ ] Implement `EventSFXGenerator` for waveforms
- [ ] Tests for all synthesis algorithms

### Phase 2: Algorithm Implementation (3-4 commits)
- [ ] Implement `EntropySynth` component (entropy → FM parameters)
- [ ] Implement `BioMusicGenerator` (genotype → melody)
- [ ] Implement spatial audio (panning + distance)
- [ ] Integration with `AudioSystem`

### Phase 3: Integration & Polish (2 commits)
- [ ] Update `update_world()` to pass PopulationStats
- [ ] Add Bio-Music trigger on lineage change
- [ ] Performance benchmarking and optimization
- [ ] Documentation and examples

---

## Future Enhancements (Phase 68.5+)

1. **Reverb Algorithm**: Simple Schroeder reverbfor space
2. **Dynamic EQ**: Filter frequencies based on ecosystem state
3. **Generative Rhythm**: Heartbeat-like rhythms from population dynamics
4. **Doppler Effect**: Pitch shift for moving entities

---

## References

- FM Synthesis Theory: Chowning, J. (1973). "The Synthesis of Complex Audio Spectra"
- Bio-inspired Sound: Miranda, E. R. (2002). "Composing Music with Computers"
- Neural Entropy: Tishby, N. et al. (1999). "The Information Bottleneck Method"

---

**Next Steps**: User feedback → Modify design → Begin Phase 1 implementation
