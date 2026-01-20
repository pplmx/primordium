# Changelog - Primordium

[简体中文](./docs/CHANGELOG_zh.md)

All notable changes to the **Primordium** project will be documented in this file. This project adheres to a phase-based evolutionary development cycle.

---

## [Phase 10: Ecosystem Dynamics] - 2026-01-21

### Evolutionary Leap: Terrain, Geography & Seasons

This phase introduces environmental heterogeneity through terrain systems and seasonal cycles, enabling emergent migration patterns.

#### ✨ Features

- **Terrain System**: Procedurally generated world terrain with distinct biomes:
  - **Mountains** (▲): Slows movement by 50%, no food spawns
  - **Rivers** (≈): Speeds movement by 50%
  - **Oases** (◊): 3× food spawn rate, attracts migration
- **Season Cycle**: Dynamic 4-season system affecting ecosystem balance:
  - **Spring**: Food ×1.5, Metabolism ×0.8 (growth period)
  - **Summer**: Food ×1.0, Metabolism ×1.2 (active period)
  - **Fall**: Food ×1.2, Metabolism ×1.0 (harvest period)
  - **Winter**: Food ×0.5, Metabolism ×1.5 (survival period)
- **Terrain-Aware AI**: Entities adapt movement speed based on terrain underfoot
- **Geographic Food Distribution**: Food clusters naturally around oases

#### 🛠️ Technical Achievements

- **Noise-Based Generation**: Multi-octave value noise for natural terrain distribution
- **Layered Rendering**: Terrain drawn as background layer before entities

---

## [Phase 9: The Omniscient Eye] - 2026-01-21

### Evolutionary Leap: Deep Analytics & Visual Narratives

This phase introduces comprehensive world analytics and narrative systems to bring the simulation to life.

#### ✨ Features

- **Era System**: Integrated a population-driven state machine that narrates world progression epochs (Genesis, Expansion, Golden Age, Decline, etc.).
- **Hall of Fame**: Real-time leaderboard tracking the top 3 fittest organisms across the simulation.
- **Visual Narratives**: Status-aware symbols (†♥♦●) and dynamic color mapping for physiological states.
- **Advanced Analytics**: Rolling brain entropy (Shannon entropy) and average lifespan metrics for monitoring biodiversity.
- **Population Dynamics**: Dual-sparkline system visualizing real-time population health versus hardware stress.

---

## [Phase 8: Apex Predators & Genetic Synergy] - 2026-01-20

### Evolutionary Leap: Predation, Sexual Reproduction & Data Portability

This phase elevates the simulation with predator-prey dynamics and genetic exchange mechanisms.

#### ✨ Features

- **Evolved Predation**: Added a 4th neural output 'Aggression' enabling organisms to consume others for massive energy gain (80% yield).
- **Sexual Reproduction**: Implemented genetic crossover allowing organisms to combine neural traits with local mates.
- **HexDNA Protocol**: Robust serialization format for exporting (`C` key) and infusing (`V` key) organisms via text files.
- **Advanced Senses**: Refactored the sensory system to handle multi-pass world updates without borrow checker conflicts.
- **Enhanced Chronicles**: UI event log now narrates predation events and genetic surges.

---

## [Phase 7: Divine Interface] - 2026-01-20

### Evolutionary Leap: Interactivity & Taxonomy

This phase focuses on the transition from a passive observer to an active "Digital Deity," introducing tools for intervention and sophisticated species classification.

#### ✨ Features

- **Mouse-Driven Interaction**: Full terminal mouse support enabled. Users can now click on individual organisms to inspect their neural state, lineage, and specific genetic traits.
- **Procedural Naming Engine**: Every organism is now assigned a unique, procedurally generated name (e.g., *Xylos-Tetra*, *Aether-7*) based on its genotype, moving beyond raw UUIDs for better storytelling.
- **Live UI Chronicles**: Implemented a real-time event log ("Chronicles") that narrates significant evolutionary events (e.g., "The Great Famine of Tick 5000", "Legendary Hero *Zenith* has fallen").
- **Divine Intervention Tools**:
  - **Genetic Surge**: Manually trigger a high-mutation burst to force rapid adaptation.
  - **Food Injection**: Interactively place resource clusters to steer population migration.
- **Genotype-based Species Clustering**: Implemented an L2-norm distance algorithm that groups organisms into "Species" based on neural weight similarity, allowing the UI to track biodiversity and the rise/fall of distinct biological lineages.

#### 🛠️ Technical Achievements

- **Event-Driven UI updates**: Optimized the event loop to drain the full queue per tick, ensuring zero-latency mouse interaction.
- **Spatial Hash Queries**: Integrated a grid-based spatial partition system to enable real-time mouse picking and optimized sensory queries at $O(N \log N)$ complexity.

---

## [Phase 6: Immersion] - 2026-01-15

### The Optimization & Deployment Phase

Focus on performance, flexibility, and the "Screensaver" experience.

#### ✨ Features

- **Spatial Hash Optimization**: Replaced $O(N^2)$ proximity checks with a dynamic **Spatial Hashing** grid. Enabling 500+ entities on standard hardware.
- **Multi-Mode Support**:
  - **Standard Mode**: Full TUI with all dashboards.
  - **Screensaver Mode**: Minimalist, distraction-free view of the world.
  - **Headless Mode**: High-speed background simulation for data mining.
- **Configuration System**: Externalized all simulation constants to `config.toml`.

---

## [Phase 5 & 5.5: The Ledger & Blockchain]

### Immutable History & Standalone Analysis

Ensuring that every legendary life is etched into the digital firmament.

#### ✨ Features

- **JSONL Event Logging**: Robust, low-overhead streaming of every life event to `logs/live.jsonl`.
- **Legendary Criteria**: Automatic archival of "Legendary Organisms" meeting elite fitness thresholds.
- **OpenTimestamps Anchoring**: SHA-256 hashes of session logs are anchored to the Bitcoin blockchain.
- **Standalone Tools**:
  - `primordium-ledger-analyzer`: Generates detailed markdown reports and family tree visualizations.
  - `primordium-ledger-verifier`: Validates the integrity of local logs against blockchain proofs.

---

## [Phase 4: Neural Awakening]

### The Transition to Intelligence

Replacing random motion with sensory-driven neural processing.

#### ✨ Features

- **4x6x3 Neural Network**: Implementation of a multilayer perceptron (MLP) for every organism.
- **Sensory Inputs**: Food vectors, Energy reserves, and Local crowding.
- **Real-time Brain Heatmap**: Visualizing synaptic weights of the selected organism.

---

## [Phase 1-3: Genesis & Resonance]

### Foundation & Hardware Coupling

The birth of the universe and the coupling of code to silicon.

#### ✨ Features

- **Ratatui Foundation**: High-performance TUI framework.
- **Metabolic Energy Loop**: Survival system with caloric costs for actions.
- **Hardware-Coupled Climate**: CPU/RAM load translates to environmental pressure.
