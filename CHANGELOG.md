# Changelog - Primordium

[简体中文](./docs/CHANGELOG_zh.md)

All notable changes to the **Primordium** project will be documented in this file. This project adheres to a phase-based evolutionary development cycle.

---

## [Phase 25: Social Complexity & Defense Evolution] - 2026-01-23

### Evolutionary Leap: Collective Intelligence & Dynamic Signaling

Phase 25 introduces deeper social dynamics, focusing on mutual protection and active communication. Lineages now possess an emergent sense of "self" and "other," enabling group defense strategies and real-time visual signaling.

#### ✨ Features

- **Lineage-Aware Sensing**: Neural networks now receive real-time data on the density of same-lineage members nearby.
- **Group Defense Mechanics**: Introduced a "Social Defense" bonus. Entities receive significantly less damage (up to 60% reduction) when surrounded by lineage allies.
- **Dynamic Color Signaling**: Organisms can now modulate their color intensity via a dedicated neural output, allowing for warning displays or stealth behaviors.
- **Lineage Density Input**: Added a 13th brain input for `LineageDensity`, enabling organisms to react to the presence of their kin.

#### 🛠️ Technical Achievements

- **Expanded Brain Topology**: Upgraded the standard architecture to **13-6-6 RNN-lite** to support new social inputs and signal outputs.
- **Social Defense Engine**: Implemented a proximity-based damage multiplier ($M_{defense}$) within the `Social` system.
- **Metabolic Signaling Cost**: Integrated signaling intensity into the metabolic cost formula, adding **0.1 energy per unit** of active modulation.

---

## [Phase 24: Lineage & Macroevolution] - 2026-01-23

### Evolutionary Leap: Ancestral Tracking & Dynastic Dominance

Phase 24 shifts the focus from individual survival to the long-term success of ancestral lines. By introducing formal lineage tracking, the simulation can now visualize how specific "dynasties" rise and fall across generations and even across different simulation universes.

#### ✨ Features

- **Lineage Tracking**: Every organism is now assigned a permanent `lineage_id` inherited from its ancestors.
- **Dynastic Dominance Visualization**: The TUI now displays the most successful lineages, showing which ancestral lines are currently dominating the ecosystem.
- **Cross-Universe Ancestry**: Lineage data is preserved during inter-universe migration, allowing your "master race" to maintain its identity even in foreign simulations.
- **Macroevolutionary Stats**: New statistical tracking for lineage diversity, extinction rates of specific lines, and ancestral longevity.

---

## [Phase 23: Phenotypic Specialization] - 2026-01-23

### Evolutionary Leap: Physical Diversification & Trade-offs

Organisms are no longer physically identical. Evolution now acts on physical traits (Phenotypes) in tandem with neural intelligence, creating a more diverse and specialized ecosystem.

#### ✨ Features

- **Unified Genotype**: Integrated physical traits directly into the genetic sequence. All attributes are now mutable, inheritable, and subject to selection.
- **Variable Sensing Range**: Organisms can evolve perception radii between 3.0 and 15.0 units.
- **Variable Max Speed**: Locomotive capabilities now range from 0.5 to 3.0 units/tick.
- **Variable Max Energy**: Energy storage capacity (stomach size) is now an evolvable trait (100-500).

#### 🛠️ Technical Achievements

- **Phenotypic Trade-off Engine**: Implemented dynamic metabolic scaling. Superior traits now carry heavy costs:
    - Sensing: +0.1 range -> +2% idle cost.
    - Speed: +0.1 speed -> +5% movement cost.
- **Biomechanical Inertia**: Introduced a mass-responsiveness model where higher energy capacity reduces acceleration (steering responsiveness).
- **Inheritance Sync**: Optimized the reproduction system to ensure phenotypic genes are correctly synced between Genotype and Component states.

---

## [Phase 22: Parallel Evolution & Global Hive] - 2026-01-23

### Evolutionary Leap: Distributed Intelligence & The Multiverse

The simulation has transcended individual machines, enabling organisms to migrate across the global network and evolve in a shared digital "Hive."

#### ✨ Features

- **Global Hive Migration**: Entities hitting the world boundaries can now be serialized and transmitted to other connected simulations via a central relay.
- **Relay Server Architecture**: A new high-performance relay server built with **Axum** to manage the distribution of life across the multiverse.
- **RESTful Monitoring**: Real-time APIs to inspect global simulation health, peer counts, and migration traffic.
- **P2P Discovery**: Automated peer announcement and listing within the Global Hive.

#### 🛠️ Technical Achievements

- **Asynchronous Networking**: Implemented a non-blocking WebSocket protocol for real-time entity transfer.
- **Secure Anchoring**: Integrated Bitcoin-based history verification into the networking stack to ensure the authenticity of incoming migrants.
- **Enhanced Networking Tests**: Comprehensive integration tests covering entity serialization, WebSocket handshakes, and broadcast logic.

---

## [Phase 21: Environmental Fluidity & Disasters] - 2026-01-21

### Evolutionary Leap: Temporal Coherence & Dynamic Crises

The digital ecosystem has gained memory and faces its first environmental catastrophes, requiring more sophisticated survival strategies.

#### ✨ Features

- **Recurrent Neural Networks (RNN-lite)**: Upgraded Brain architecture to include feedback loops from the previous tick's hidden state, enabling time-coherent behavior and internal memory.
- **Dust Bowl Disaster**: Introduced dynamic terrain events where high population stress and heat waves trigger widespread soil depletion and barren transitions.
- **Physical Barriers**: Added impassable `Wall` terrain types that force organisms to evolve steering and obstacle avoidance.
- **Proximity-Based Sensing**: Migrated from global food knowledge to a realistic "Sensing Radius" (20 units) powered by a dedicated `food_hash`.

#### 🛠️ Technical Achievements

- **O(1) Sensing**: Integrated a second Spatial Hash specifically for resources, reducing sensory query complexity.
- **Buffer Pooling**: Implemented reusable heap allocations within the `World` struct, significantly reducing allocation overhead during parallel execution.
- **Stateful Intelligence**: Added persistent `last_hidden` states to the `Intel` component, supporting the new recurrent brain architecture.
- **Physics Engine Update**: Enhanced `handle_movement` to support collision detection and reflection against impassable terrain.

---

## [Phase 20: Cognitive Synthesis & Systemic Refactor] - 2026-01-21

### Evolutionary Leap: Component-Based Life & Parallel Intelligence

The simulation has undergone its most significant architectural evolution yet, transitioning to a modular, component-based system designed for peak performance and extreme scalability.

#### ✨ Features

- **Component-Based Entity (CBE)**: Organism attributes are now logically grouped into Physics, Metabolism, Health, and Intel components, improving system isolation and data locality.
- **Systemic Decomposition**: The world update loop is now a pipeline of specialized "Systems" (Perception, Action, Biological, Social), making the logic easier to extend and maintain.
- **Rayon-Powered Parallelism**: Integrated the Rayon data-parallelism library to handle heavy computational loads across all CPU cores.
- **High-Density Scaling**: Optimized for 5000+ simultaneous entities with multi-threaded neural processing.

#### 🛠️ Technical Achievements

- **Parallel Brain Inference**: Neural network forward passes are now processed in parallel using `.par_iter()`.
- **Systemic Pipeline**: Refactored `World::update` into discrete stages, resolving long-standing "monolith" technical debt.
- **Data Locality**: Component grouping allows the simulation to process only relevant data subsets per system, reducing cache misses.
- **Logic Parity**: Achieved 100% functional parity with previous versions while dramatically increasing throughput.

---

## [Phase 19: Circadian Rhythms] - 2026-01-21

### Evolutionary Leap: The Temporal Dimension

The digital ecosystem now pulses with the cycle of Day and Night, affecting both plant life and organism metabolism.

#### ✨ Features

- **Circadian Cycle**: A global world clock transitioning between Day and Night.
- **Light-Dependent Growth**: Food spawn rates are tied to the `light_level`. Midday sees peak growth.
- **Resting Metabolism**: Organisms have a resting state at night, reducing idle energy consumption by 40%.
- **TUI Visualization**: Real-time Day/Night icons (☀️/🌙) added to the dashboard.

#### 🛠️ Technical Achievements

- **Environment Ticking**: Implemented a per-tick update mechanism for the `Environment` state.
- **Physics Coupling**: Integrated light levels and circadian multipliers into the core metabolic formulas.
- **Verification**: Added `test_light_dependent_food_growth` to the integration suite.

---

## [Phase 15-18: Biological & Ecological Depth] - 2026-01-21

### Evolutionary Leap: Life Cycles, Trophic Levels, and Pathogen Evolution

The digital ecosystem has evolved into a complex web of life with developmental stages, dietary specializations, and immunological defense mechanisms.

#### ✨ Features

- **Phase 15: Life Cycles**:
    - **Juvenile State (◦)**: Entities now start as immatures and must survive 150 ticks before reaching adulthood.
    - **Maturity Gate**: Reproduction is disabled for juveniles, creating a vulnerable early-life period.
- **Phase 16: Trophic Levels**:
    - **Herbivores (H-)**: Specialized in plant consumption; inefficient hunters.
    - **Carnivores (C-)**: Obligate predators; highly efficient hunters (1.2x yield).
- **Phase 17: Ecological Succession**:
    - **Terrain Health (Fertility)**: Land fertility depletes when plants are consumed.
    - **Barren Terrain (░)**: Overgrazed land turns barren, stopping food production.
- **Phase 18: Pathogens & Immunity**:
    - **Contagion Simulation**: Pathogens spread through proximity, factoring in virulence and host immunity.
    - **Immunity Evolution**: Entities gain resistance after recovery and pass it to offspring with minor mutations.
    - **Environmental Outbreaks**: Randomly emerging pathogens add dynamic selective pressure.

#### 🛠️ Technical Achievements

- **Spatial Logic Refactor**: Fixed indexing in `World::update` to ensure spatial hashing works correctly with `std::mem::take` for entity snapshots.
- **Pathogen Model**: Introduced a new `Pathogen` module with custom transmission and lethality dynamics.
- **Comprehensive Testing**: Added `tests/pathogens.rs` and reached 41 total tests.

---

## [Phase 14: Gameplay & Polish] - 2026-01-21

### Evolutionary Leap: Divergent Realities & Digital Stability

The simulation now supports distinct game modes and has reached peak stability through a massive architectural refactor and comprehensive quality enforcement.

#### ✨ Features

- **Game Modes**:
    - **Cooperative (`--gamemode coop`)**: Global peace enforced; ideal for colony growth.
    - **Battle Royale (`--gamemode battle`)**: Shrinking world borders force conflict.
- **Enhanced TUI**:
    - **Legend Bar**: Added a dedicated reference line for entity status and terrain symbols.
    - **Help System**: New 4-tab tabbed help guide (Controls, Symbols, Concepts, Eras).
    - **Onboarding**: A 3-screen tutorial for new gods entering the Primordium.
- **Documentation**:
    - Comprehensive **User Manuals** (EN/ZH).
    - **Technical Wiki** covering Genetics, Brains, and Ecosystems.
    - **Agent Memory**: New `AGENTS.md` to assist AI pair programmers.

#### 🛠️ Technical Achievements

- **Modular Refactor**: Decomposed the 870-line `app.rs` into a clean, modular structure under `src/app/`.
- **Quality Gate**: Established a strict pre-commit pipeline using Husky (Fmt, Check, Clippy, Test).
- **Comprehensive Testing**: Added 30+ tests covering unit logic (Brain, Entity, Quadtree) and integration workflows (Life cycles, DNA flow, Era transitions).
- **Performance**:
    - Enabled **LTO** (Link Time Optimization) for release builds.
    - Replaced $O(N^2)$ bottlenecks with optimized iterators and spatial hashing.
- **Robustness**: 100% Clippy compliance and automated log directory management.

---

## [Phase 13: Multiplayer Primordium] - 2026-01-21

### Evolutionary Leap: Interstellar Migration

Primordium now supports distributed simulations where entities can migrate between different users' "universes" via a relay server.

#### ✨ Features

- **Relay Server**: New `primordium-server` binary using `axum` and WebSockets to route traffic.
- **Client Networking**: WASM client can connect to relay server and send/receive entities.
- **Inter-World Migration**: Entities hitting the world edge are serialized and sent to other connected clients.
- **Real-time Visualization**: Web UI indicates connection status and network events.

#### 🛠️ Technical Achievements

- **WebSocket Protocol**: Custom JSON-based protocol for handshake and entity transfer.
- **Multi-Crate Workspace**: Project structure updated to support `bin` (server) and `lib/wasm` (client) targets.
- **Entity Serialization**: DNA and stats are preserved during network transit.

---

## [Phase 12: WebAssembly Port] - 2026-01-21

### Evolutionary Leap: Breaking the Terminal Barrier

Primordium can now run in modern web browsers via WebAssembly and HTML5 Canvas.

#### ✨ Features

- **WASM Support**: Core simulation compiled to WebAssembly via `wasm-pack`.
- **Canvas Rendering**: New `WebRenderer` replaces TUI for 60 FPS browser visualization.
- **Web Interface**: Modern "Glassmorphism" UI with real-time stats and controls.
- **Dual Target**: Project supports both native CLI (TUI) and Web (Canvas) builds.

#### 🛠️ Technical Achievements

- **Library Refactoring**: Extracted core logic into `lib.rs` for dual-target support.
- **Conditional Compilation**: Usage of `#[cfg(target_arch = "wasm32")]` to maintaining native compatibility.
- **JS Interop**: Exposed `Simulation` struct and `draw` method to JavaScript via `wasm-bindgen`.

---

## [Phase 11: Social Structures] - 2026-01-21

### Evolutionary Leap: Pheromones, Tribes & Cooperation

This phase introduces emergent social behaviors through chemical communication, kin recognition, and cooperative behaviors.

#### ✨ Features

- **Pheromone System**: Entities leave persistent chemical trails:
    - **Food Pheromones**: Deposited when eating, attract foragers
    - **Danger Pheromones**: Deposited at kill sites, warn of predators
    - Pheromones decay over time (0.5% per tick)
- **Tribe Formation**: Color-based kin recognition:
    - Entities with similar colors (RGB distance < 60) form tribes
    - Same-tribe members never attack each other
- **Territorial Behavior**: Entities are 50% more aggressive near birth location
- **Energy Sharing**: High-energy entities can share energy with starving neighbors
    - New "Sharing" status (♣) indicates active sharing
- **Expanded Neural Network**: 6 inputs → 6 hidden → 5 outputs:
    - New inputs: Pheromone strength, Tribe density
    - New output: Share intent

#### 🛠️ Technical Achievements

- **PheromoneGrid**: Efficient grid-based pheromone storage with decay
- **Vec-based Brain**: Arrays converted to Vec for serde compatibility
- **Social Symbols**: New sharing symbol (♣) and green status color

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
