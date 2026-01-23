# Primordium (原初之境) - Development Roadmap

> *Evolution in silicon, powered by your machine*

A hardware-coupled artificial life simulation where digital organisms evolve neural networks in your terminal, with their world shaped by your computer's real-time performance.

---

## 🎯 Project Vision

Primordium is not just a screensaver—it's a **living laboratory** where:

- CPU temperature becomes environmental climate
- RAM pressure controls resource scarcity
- Neural networks emerge through natural selection
- Every legendary organism's DNA is preserved on blockchain
- Your machine becomes a god, and you become the observer

---

## 📦 Technology Stack

```toml
[dependencies]
# Core rendering
ratatui = "0.26"
crossterm = "0.27"

# System monitoring
sysinfo = "0.30"

# Simulation
rand = "0.8"

# Data persistence
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

# Blockchain & Crypto
sha2 = "0.10"
hex = "0.4"
reqwest = "0.11"
tokio = "1.0"
async-trait = "0.1"

# CLI & Analysis
clap = "4.0"
petgraph = "0.6"
toml = "0.8"
```

---

## 🗺️ Development Phases

### Phase 1: Genesis - Physics Foundation ✅

**Goal:** Build the terminal universe and basic physics

- Initialize Ratatui TUI framework with crossterm backend
- Implement World grid system
- Create Entity system with position and velocity vectors
- Basic physics: random walk with momentum
- Boundary collision detection (bounce)
- 60 FPS rendering loop with smooth updates

### Phase 2: The Breath of Life - Metabolism & Evolution ✅

**Goal:** Introduce life, death, and heredity

- Energy system: Movement and idle costs
- Food chain: Dynamic green food particles `*`
- Collision detection: Consumption restores energy
- Asexual Reproduction: Energy split with offspring
- Genetic Inheritance: Velocity and color mutation

### Phase 3: Hardware Resonance - Environmental Coupling ✅

**Goal:** Bridge virtual and physical worlds

- Real-time system monitoring using `sysinfo`
- CPU-Coupled Climate: Affects metabolic energy burn (1.0x to 3.0x)
- RAM-Coupled Resources: Affects food spawn frequency (1.0x to 0.1x)
- Visual Feedback: Hardware gauges and CPU historical sparkline
- Environmental Events: Heat waves, ice ages, and abundance cycles

### Phase 4: Neural Awakening - Intelligent Behavior ✅

**Goal:** Replace random walk with learned behavior

- Sensory Inputs: Food proximity, energy ratio, and local crowding
- Neural Network: 4x6x3 MLP architecture (42 genes)
- Activation: Tanh for hidden and output layers
- Brain Visualization: Real-time weight heatmap mode (`B` key)
- Fitness Landscape: Emergent survival behaviors via natural selection

### Phase 5: The Ledger - Historical Archives ✅

**Goal:** Preserve evolutionary history for analysis

- Identity System: Unique UUIDs and lineage tracking (parent/child)
- Live Event Stream: `logs/live.jsonl` (JSON Lines format)
- Legends Archive: `logs/legends.json` for high-fitness organisms
- Analysis Tool: `primordium-analyze` binary for family tree reconstruction and reporting

### Phase 5.5: Blockchain Anchoring - Immutable Proof ✅

**Goal:** Cryptographically prove evolutionary history

- Hash Timestamping: SHA-256 integrity hashing of legendary datasets
- Blockchain Submission: Modular provider architecture
- OpenTimestamps Integration: Anchoring hashes to the Bitcoin network
- Verification Utility: `verify` binary to validate local data against blockchain proofs

### Phase 6: Immersion - Polish & Deployment ✅

**Goal:** Production-ready experience and optimization

- Multi-Mode Support: Standard, Screensaver, and Headless modes
- Performance Optimization: Grid-based Spatial Hashing (O(N log N) queries)
- Configuration System: External `config.toml` for simulation tuning
- UI Polish: Interactive help overlay, time scaling, and resize handling
- Release Preparation: Optimized builds and comprehensive documentation

### Phase 7: Divine Interface - Interactive Observation ✅

**Goal:** Transform observer into active "Digital Deity"

- Mouse-Driven Interaction: Click to select and track organisms
- Procedural Naming Engine: Unique names based on genotype
- Live UI Chronicles: Real-time event log narrating evolutionary milestones
- Divine Intervention: Food injection (Right Click) and Genetic Surge (X key)
- Genotype-based Species Clustering: L2-norm distance classification

### Phase 8: Apex Predators & Genetic Synergy ✅

**Goal:** Introduce predation and sexual reproduction

- Evolved Predation: 4th neural output 'Aggression' for hunting (80% energy yield)
- Sexual Reproduction: Genetic crossover with nearby mates
- HexDNA Protocol: Export (C) and Import (V) organism genomes as text files
- Advanced Senses: Multi-pass world updates without borrow conflicts
- Enhanced Chronicles: Predation events and genetic surge narration

### Phase 9: The Omniscient Eye ✅

**Goal:** Deep analytics and visual narratives

- Era System: Population-driven state machine for world epochs
- Hall of Fame: Top 3 fittest organisms leaderboard
- Visual Narratives: Status-aware symbols (†♥♦●) and dynamic coloring
- Advanced Analytics: Brain entropy, average lifespan metrics
- Population Dynamics: Dual-sparkline health vs hardware stress visualization

### Phase 10: Ecosystem Dynamics ✅

- Terrain & Geography: Mountains (slow), Rivers (fast), Oases (food)
- Environmental Heterogeneity for emergent migration patterns
- Weather systems: Seasons, storms, and climate shifts

### Phase 11: Social Structures ✅

- Pheromone system: Entities leave chemical trails
- Food sharing: High-energy entities donate to neighbors
- Territorial behavior: Aggressive entities drive others away
- Tribe formation: Color-based group identity

### Phase 12: WebAssembly Port ✅

- Compile to WASM with wasm-pack
- Canvas-based rendering (no terminal)
- Share simulations via URL

### Phase 13: Multiplayer Primordium ✅

- Network protocol for synchronized worlds
- Cross-machine organism migration
- Competitive and cooperative modes

### Phase 14: Gameplay & Polish ✅

- Performance Tuning (LTO)
- User Manuals (EN/ZH)
- Detailed Wiki

### Phase 15: Life Cycles & Maturity ✅

- Juvenile state for new offspring
- Maturity age requirement for reproduction
- Age-based visual differentiation

### Phase 16: Trophic Levels & Dietary Niche ✅

- Herbivores (plant-eaters) vs Carnivores (predators)
- Energy gain multipliers based on role
- Speciation mechanism for role evolution

### Phase 17: Ecological Succession & Terrain Health ✅

- Dynamic soil fertility (depletes when overgrazed)
- Barren terrain state with recovery cycles
- Forced migration patterns due to resource depletion

### Phase 18: Pathogens & Immunity Evolution ✅

- Proximity-based contagion system
- Adaptive immunity through survival
- Transgenerational resistance inheritance

### Phase 19: Circadian Rhythms & Temporal Ecology ✅

- Day/Night cycle affecting light and metabolism
- Light-dependent plant growth
- Rest-state energy conservation

### Phase 20: Cognitive Synthesis & Systemic Refactor ✅

- **Component grouping**: Refactored `Entity` struct into Physics, Metabolism, Health, and Intel.
- **Systemic Decomposition**: Decomposed monolithic `World::update` into modular Perception, Action, Biological, and Social systems.
- **Rayon Integration**: Multi-threaded brain processing and perception lookups for 5000+ entities.

### Phase 21: Environmental Fluidity & Disasters ✅

- **Memory Neurons**: Upgraded Brain architecture to RNN-lite (Recurrent Neural Network) for temporal coherence.
- **Dynamic Terrain**: Implemented "Dust Bowl" disasters triggered by high heat and population stress.
- **Physical Barriers**: Added impassable `Wall` terrain types for steering challenges.
- **Performance Tuning**: Integrated `food_hash` for $O(1)$ proximity sensing and buffer pooling for zero-jitter allocation.

### Phase 22: Parallel Evolution & Global Hive ✅

- **Rayon Integration**: Multi-threaded brain processing for 5000+ entities. *(Completed in Phase 20)*
- **P2P Peer Discovery**: WebSocket relay with peer tracking and REST APIs (`/api/peers`, `/api/stats`).
- **Network Protocol**: Extended `NetMessage` with `PeerInfo`, `PeerAnnounce`, and `PeerList` types.
- **WASM Client Enhancement**: Network state tracking, migration stats, peer awareness.
- **Bug Fixes**: Entity DNA serialization for cross-universe migration, WebRenderer terrain completeness.

---

## 📊 Development Timeline

| Phase | Milestone | Status |
| ------- | ------------ | --------- |
| Phase 1 | First moving pixels | ✅ Complete |
| Phase 2 | Natural selection visible | ✅ Complete |
| Phase 3 | Hardware coupling working | ✅ Complete |
| Phase 4 | Intelligent behavior emerges | ✅ Complete |
| Phase 5 | Historical records complete | ✅ Complete |
| Phase 5.5 | Blockchain integration | ✅ Complete |
| Phase 6 | Production release | ✅ Complete |
| Phase 7 | Divine Interface | ✅ Complete |
| Phase 8 | Apex Predators & Genetic Synergy | ✅ Complete |
| Phase 9 | The Omniscient Eye | ✅ Complete |
| Phase 10 | Ecosystem Dynamics | ✅ Complete |
| Phase 11 | Social Structures | ✅ Complete |
| Phase 12 | WebAssembly Port | ✅ Complete |
| Phase 13 | Multiplayer Primordium | ✅ Complete |
| Phase 14 | Gameplay & Polish | ✅ Complete |
| Phase 15 | Life Cycles | ✅ Complete |
| Phase 16 | Trophic Levels | ✅ Complete |
| Phase 17 | Ecological Succession | ✅ Complete |
| Phase 18 | Pathogens & Immunity | ✅ Complete |
| Phase 19 | Circadian Rhythms | ✅ Complete |
| Phase 20 | Cognitive Synthesis | ✅ Complete |
| Phase 21 | Environmental Fluidity | ✅ Complete |
| Phase 22 | Parallel Evolution | ✅ Complete |

---

## 🌱 Philosophy

Primordium is an experiment in **emergent complexity**. You provide the rules, the hardware provides the pressure, and evolution writes the story.

Every run is unique. Every lineage is precious. Every extinction teaches us something.

*Last updated: 2026-01-21*
*Version: 0.0.1*
