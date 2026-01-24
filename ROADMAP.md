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

### Phase 23: Phenotypic Specialization & Unified Genotype ✅

- **Unified Genotype**: Integrated neural weights and physical traits into a single genetic sequence.
- **Evolvable Morphology**: Mutable Sensing Range (3-15), Max Speed (0.5-3.0), and Max Energy (100-500).
- **Metabolic Trade-offs**: Sensing and Speed capability increase base idle/move costs.
- **Biomechanical Inertia**: Energy storage capacity affects mass and steering responsiveness.
- **HexDNA 2.0**: Upgraded protocol for 100% fidelity cross-universe migrations.

### Phase 24: Lineage & Macroevolution ✅

- **Ancestral Tracking**: Every entity assigned a `lineage_id` descending from original founders.
- **Inheritance Engine**: Preservation of lineage during crossover and mutation.
- **Dynastic Dominance**: TUI visualization of top 3 dominant ancestral lines.
- **Hive Ancestry**: Lineage preservation across global migrations.

### Phase 25: Social Complexity & Defense Evolution ✅

- **Group Defense**: Proximity to same-lineage members reduces damage from predation.
- **Dynamic Signaling**: 6th neural output for real-time color modulation (stealth/warning).
- **Lineage Sensor**: 13th neural input detects nearby same-lineage density for evolved herding.
- **Social Pheromones**: Integrated presence-based herding behavior.

### Phase 26: Divine Interface v2 - Interactive Deity ✅

- **Real-time Terrain Editing**: Mouse-driven brush for placing Walls, Oasis, and Rivers.
- **Genetic Engineering**: Targeted Divine Intervention (Mutate, Smite, Reincarnate) for selected entities.
- **Disaster Dispatcher**: Manually trigger Heat Waves (K), Mass Extinctions (L), or Resource Booms (R).

### Phase 27: Persistent Legends & High-Performance Analytics ✅

- **Lineage Registry**: Persistent tracking of ancestral success metrics in `logs/lineages.json`.
- **Deeper Metrics**: Track "Total Entities Produced" and "Total Energy Consumed" per lineage across sessions.
- **Dynastic Hall of Fame**: UI visualization for all-time successful ancestral lines.
- **Macro-Analytics**: Population stats now include living lineage distribution.

### Phase 28: Complex Brain Evolution (NEAT-lite) ✅

- **Dynamic Topology**: Brains evolved from fixed MLP to graph-based NEAT-lite architecture.
- **Topological Mutation**: Neurons can be added (split connections) and new connections formed during mutation.
- **Structural Crossover**: Innovation-aware genetic exchange preserves cognitive structures.
- **Efficiency Pressure**: Metabolic costs added for hidden nodes (0.02) and enabled connections (0.005).

### Phase 29: Semantic Pheromones & Language Evolution ✅

- **Chemical Channels**: Expanded pheromone grid to support `SignalA` and `SignalB` abstract channels.
- **Dynamic Emission**: 2 new neural outputs for active chemical signaling.
- **Semantic Sensing**: 2 new neural inputs for detecting nearby signal concentrations.
- **Coordinated Foraging**: Substrate for evolved "Food Alert" or "Rally" chemical behaviors.

### Phase 30: Social Coordination & Kin Recognition ✅

- **Kin Perception**: Entities perceive the relative center of mass (Centroid) of their own lineage members.
- **Herding Bonus**: Metabolic reward (+0.05 energy) for moving in alignment with kin vectors.
- **Cognitive Expansion**: Brain upgraded to 18-input / 8-output architecture.
- **Spatial Awareness**: Added Wall Proximity and Biological Age sensors.

### Phase 31: Metabolic Niches & Resource Diversity ✅

- **Nutrient Variability**: Food now has a `nutrient_type` (Green/Blue) coupled to terrain (Plains/Mountains).
- **Digestive Genes**: Added `metabolic_niche` gene to Genotype for dietary specialization.
- **Digestive Efficiency**: Energy gain scales from 0.2x (mismatch) to 1.2x (specialist match).
- **Brain Sync**: 19th neural input for perceiving nutrient types of nearest resources.

### Phase 32: Life History Strategies (R/K Selection) ✅

- **Reproductive Investment**: New genes for maturity age and energy transfer ratio.
- **Offspring Quality**: Trade-off between many weak offspring (R-strategy) vs. few strong ones (K-strategy).
- **Developmental Scaling**: Max energy capacity scales with maturation time (Growth vs Size).
- **Strategy Inheritance**: Crossover and mutation of life history traits.

### Phase 32.5: Hardening & Survival Validation ✅

- **Engine Hardening**: Zero-panic guarantee on malformed DNA or version mismatches during migration.
- **Survival Stress Tests**: Verified metabolic sinks (bloated brains, high speed) cause starvation as intended.
- **Selection Validation**: Proven R-strategy dominance in boom cycles and K-strategy stability.
- **Architecture Cleanup**: Unified system parameters into `ActionContext` for clean scalability.

### Phase 33: Trophic Continuum & Dynamic Diets ✅

- **Predatory Potential Gene**: Replaced binary roles with a continuous trophic scale (0.0 to 1.0).
- **Digestive Versatility**: Implemented efficiency scaling where herbivores (0.0) eat plants efficiently and carnivores (1.0) extract maximum energy from predation.
- **Omnivory Emergence**: Generalists (0.3-0.7) can now consume both resources at reduced efficiency, enabling survival in fluctuating environments.
- **Trophic Sync**: Updated brain sensors and status naming to reflect the new diet spectrum.

### Phase 34: The Tree of Life (Ancestry Visualization) ✅

- **Lineage Tree Logic**: Implemented `AncestryTree` builder using `petgraph` to track parent-child relationships.
- **TUI Tree View**: Added real-time "Ancestry" panel (A key) showing the top 5 living dynasties and their representatives.
- **Trophic Mapping**: Visualized dietary branching (Herbivore/Carnivore/Omnivore icons) within the lineage view.
- **Tree Exporter**: Added Shift+A command to export the entire simulation's ancestry as a Graphviz DOT file.
- **Analytics Tool**: Updated `analyze` binary to generate family tree visualizations from historical logs.

### Phase 35: Trophic Cascades & Ecosystem Stability ✅

- **Self-Regulating Population**: Implemented feedback loops where herbivore over-population reduces soil recovery.
- **Hunter Competition**: Predatory energy gain now scales inversely with global predator biomass.
- **Eco-Stability Alerts**: Added real-time detection and warnings for Trophic Collapse and Overgrazing.
- **Trophic Naming**: Enhanced lineage naming with prefixes (H-, O-, C-) based on genetic dietary potential.

### Phase 36: World State Persistence (The Living Map) ✅

- **Manual Save/Load**: Added 'W' to save and 'O' to load the entire world state (terrain, food, and organisms).
- **Auto-Resume**: Simulation automatically attempts to load `save.json` on startup for persistent sessions.
- **Cross-Session Analytics**: `LineageRegistry` is now loaded on startup, preserving all-time statistics.

### Phase 37: Sexual Selection & Mate Preference ✅

- **Mate Choice Logic**: Entities evaluate nearby mates based on physical and cognitive traits.
- **Preference Genes**: Added `mate_preference` gene determining attractiveness based on trophic potential matching.
- **Selective Breeding**: Natural emergence of specialized clusters due to assortative mating patterns.
- **Runaway Simulation**: Proved that sexual selection can drive traits faster than survival alone in integration tests.

### Phase 38: Environmental Succession (The Living World) ✅

- **Dynamic Biomes**: Implemented terrain transitions (Plains -> Forest, Plains -> Desert) based on long-term biomass and water proximity.
- **Carbon Sequestration**: Entities impact atmospheric state (Climate) through cumulative metabolic activity.
- **Soil Exhaustion**: Permanent fertility damage from extreme over-grazing requiring intentional "fallow" periods.
- **Biodiversity Hotspots**: Emergence of hyper-diverse regions based on environmental edge-effects.

### Phase 39.5: Performance & Observability (Foundation Refinement) ✅

- **Parallel Terrain Updates**: Refactored `TerrainGrid::update` to use `Rayon` for row-parallel processing, reducing $O(W \times H)$ bottleneck.
- **Eco-Observability**: Added real-time tracking for Carbon (CO2), Climate state, and Mutation scaling in the TUI status bar.
- **God Mode Hard Reboot**: Enhanced 'Mass Extinction' (L) to reset atmospheric CO2, providing a clean slate for new simulations.
- **Visual Feedback**: Implemented fertility-based terrain dimming to visually represent soil exhaustion.
- **Quality Gates**: Achieved zero-warning baseline across all 56+ integration tests and Clippy.

### Phase 40: Archeology & Deep History ✅

- **Fossil Record**: Persistent storage of extinct "Legendary" genotypes (`logs/fossils.json`) for retrospective analysis.
- **Deep History View**: TUI-based timeline browser (Shortcut: `Y`) allowing users to navigate through periodic world snapshots.
- **Playback Infrastructure**: Implemented `Snapshot` events in `HistoryLogger` to track macro-evolutionary state over time.
- **Time Travel Navigation**: Added keyboard controls (`[` and `]`) to seek through historical snapshots within the Archeology Tool.

### Phase 41: Massive Parallelism & Spatial Indexing ✅

- **Rayon Integration**: Multi-threaded brain processing and perception lookups for 10,000+ entities.
- **3-Pass Update Strategy**: Parallelized world update pipeline (Snapshot -> Interaction Proposals -> Sequential Resolution).
- **Spatial Scaling**: Row-partitioned Spatial Hash with parallel construction.
- **Performance**: Zero-jitter simulation scaling across all CPU cores.

### Phase 42: Adaptive Radiations & Macro-Environmental Pressures ✅

- **Dynamic Era Transitions**: Automated shifts in world epochs based on global biomass, carbon levels, and biodiversity indices.
- **Evolutionary Forcing**: Eras impact global mutation rates, resource spawn patterns, and metabolic costs to force "Adaptive Radiations".
- **Ecological Indicators**: TUI visualization of "World Stability" and "Evolutionary Velocity".
- **Feedback Loops**: Carbon levels impacting climate state (Global Warming) and biome succession rates.

### Phase 43: Adaptive Speciation & Deep Evolutionary Insights ✅

- **Automatic Speciation**: Real-time lineage splitting based on genetic distance (NEAT topology + Phenotypic traits).
- **Evolutionary Velocity**: Slide-window metrics tracking the "speed" of genetic drift in the population.
- **Enhanced Archeology**: Direct interaction with fossils (Resurrection/Cloning) to reintroduce extinct genotypes.
- **TUI Dashboard v3**: Integrated Era Selection Pressure indicators and detailed Fossil Record browser.

### Phase 44: Niche Construction & Nutrient Cycling ✅

- **Corpse Fertilization**: Death returns a percentage of metabolic energy to the terrain's soil fertility.
- **Metabolic Feedback**: Entities "excrete" nutrients during movement, favoring plant growth in highly populated areas.
- **Registry Pruning**: Automated cleanup of extinct, low-impact lineages to ensure long-term performance.
- **Eco-Dashboard**: Global Fertility and Matter Recycling Rate metrics for ecosystem health monitoring.

### Phase 45: Global Hive - Robust P2P Connectivity ✅

- **Enhanced Migration Protocol**: Versioned entity transfer with checksums to prevent cross-universe corruption.
- **Backpressure & Flux Control**: Inbound/Outbound buffers to prevent population spikes during massive migrations.
- **Universal Lineage Tracking**: Stable ID preservation for lineages moving between multiple world instances.
- **Hive-Aware UI**: Real-time network health, peer counts, and migration traffic monitoring.

### Phase 46: Evolutionary Stable Strategy (ESS) & Social Topology 🚧

- **Hamilton's Rule Integration**: Social benefits (Sharing, Defense) weighted by genetic relatedness ($r$).
- **Social Punishment**: Reputation-based mechanics where "betrayers" or "exploiters" face community retaliation.
- **Speciation Branching**: Improved visualization of the social tree, showing how groups diverge into tribes.
- **Social Interventions**: Divine tools to enforce peace zones or war zones to steer group behaviors.

## 📊 Development Timeline

| Phase | Milestone | Status |
| ------- | ------------ | --------- |
| Phase 41 | Massive Parallelism | ✅ Complete |
| Phase 42 | Adaptive Radiations | ✅ Complete |
| Phase 43 | Adaptive Speciation | ✅ Complete |
| Phase 44 | Nutrient Cycling | ✅ Complete |
| Phase 45 | Global Hive | ✅ Complete |
| Phase 46 | Social Strategy | 🚧 In Progress |

---

## 🌱 Philosophy

Primordium is an experiment in **emergent complexity**. You provide the rules, the hardware provides the pressure, and evolution writes the story.

Every run is unique. Every lineage is precious. Every extinction teaches us something.

*Last updated: 2026-01-21*
*Version: 0.0.1*
