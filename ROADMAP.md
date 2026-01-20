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

---

## 📊 Development Timeline

| Phase | Milestone | Status |
|-------|-----------|--------|
| Phase 1 | First moving pixels | ✅ Complete |
| Phase 2 | Natural selection visible | ✅ Complete |
| Phase 3 | Hardware coupling working | ✅ Complete |
| Phase 4 | Intelligent behavior emerges | ✅ Complete |
| Phase 5 | Historical records complete | ✅ Complete |
| Phase 5.5| Blockchain integration | ✅ Complete |
| Phase 6 | Production release | ✅ Complete |

**Core Development Completed in ~1 day (Ultra Mode Enabled).**

---

## 🚀 Extended Roadmap (Phase 7+)

### Phase 7.1: Predator-Prey Dynamics
- New Entity Type: Red predators that hunt blue herbivores
- Predators gain energy by "eating" herbivores
- Herbivores eat green food
- Three-tier food chain creates stable oscillations
- Co-evolution: prey evolve evasion, predators evolve pursuit

### Phase 7.2: Social Behavior
- Pheromone system: entities leave chemical trails
- Food sharing: high-energy entities can donate to neighbors
- Territorial behavior: aggressive entities drive others away
- Tribe formation: color-based group identity

### Phase 7.3: Terrain & Geography
- Environmental Heterogeneity: Mountains (slow), Rivers (fast), Oases (food)
- Migration patterns emerge naturally

### Phase 7.4: WebAssembly Port
- Compile to WASM with `wasm-pack`
- Canvas-based rendering (no terminal)
- Share simulations via URL

---

## 🌱 Philosophy

Primordium is an experiment in **emergent complexity**. You provide the rules, the hardware provides the pressure, and evolution writes the story.

Every run is unique. Every lineage is precious. Every extinction teaches us something.

*Last updated: 2026-01-20*
*Version: 1.0.0-stable*
