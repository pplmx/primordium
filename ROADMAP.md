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

## 🎯 Immediate Priorities (Top 10)

> **"Focus is not saying yes; it is saying no to the hundred other good ideas."**

These tasks are the critical path to the next major version of Primordium.

1.  **T1: Architectural Decoupling (Workspace Refactor)** - *Critical Pre-requisite* ✅ (Complete - Core/Data/Observer crates, Trait-based behaviors)
2.  **Phase 66: Data-Oriented Core (ECS Refactor)** - *Performance Foundation* 🚧 (Step 1 Done - Food migrated to hecs, Parallel SpatialHash implementation)
3.  **Phase 66.5: Cognitive Hygiene & Resilience** - *Long-term Stability* ✅ (Done - Renaming, Neural Pruning, Zero-Allocation Optimizations)
4.  **T2: Engineering Excellence (CI/CD + Determinism)** - *Safety Net* ✅ (CI + Release workflows, Zero-Allocation Core, Parallel Scaling verified)
5.  **Phase 65: The Silicon Scribe (LLM Integration)** - *User Engagement* ✅ (Foundation - Heuristic narration via primordium_observer)
6.  **Phase 64: Genetic Memory & Evolutionary Rewind** - *Core Simulation Depth* ✅
7.  **Phase 66 Step 2: Entity ECS Migration** - *Performance* - Next priority
8.  **Phase 67: The Creator's Interface (Plugin Architecture)** - *Extensibility*
8.  **Phase 68: The Song of Entropy (Audio)** - *Immersion*
9.  **Phase 69: Visual Synthesis (ASCII Raytracing)** - *Visual Polish*
10. **Phase 70: The Galactic Federation (Central Server)** - *Online Universe*

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

### Phase 46: Evolutionary Stable Strategy (ESS) & Social Topology ✅

- **Hamilton's Rule Integration**: Social benefits (Sharing, Defense) weighted by genetic relatedness ($r$).
- **Social Punishment**: Reputation-based mechanics where "betrayers" or "exploiters" face community retaliation.
- **Speciation Branching**: Improved visualization of the social tree, showing how groups diverge into tribes.
- **Social Interventions**: Divine tools to enforce peace zones or war zones to steer group behaviors.

### Phase 47: Lifetime Learning (Neuroplasticity) ✅

- **Hebbian Learning**: Real-time weight adjustment based on neural co-activation ($\Delta w = \eta \cdot pre \cdot post$).
- **Reinforcement Signals**: Global modulators (Food=+1, Pain=-1) guiding plasticity towards survival strategies.
- **Epigenetic Priming**: Lamarckian inheritance where offspring inherit learned weight biases.
- **Neural Dashboard**: Activity heatmap and plasticity visualization in TUI.

### Phase 48: Linguistic Evolution ✅

### Phase 49: Advanced Social Hierarchies (Tribal Warfare) ✅

- **Tribal Splits**: Mechanisms for large tribes to fracture into competing factions based on genetic drift or leadership crises.
- **Warfare Logic**: Organized aggression where "Soldier" castes attack foreign entities.
- **Leadership Roles**: Emergence of "Alpha" entities that influence the movement of their tribe.
- **Territory Claims**: Persistent memory of "Home Turf" and defense bonuses.

### Phase 50: Visualizing the Invisible (Collective Intelligence) ✅

- **Rank Heatmaps**: Visualize social stratification and Alpha-centric tribal organization in real-time.
- **Vocal Propagation**: Yellow sound-density overlays revealing coordination signals and alarm ripples.
- **Dynamic Sovereignty**: Alpha-driven territoriality where leaders claim local zones as Peace/War regions.
- **Leadership Auras**: Visual highlights for Soldiers and Alphas in specialized view modes.
- **Collective Reinforcement**: Socially-aware Hebbian learning loop that associates vocal signals with survival rewards.

### Phase 51: Symbiosis (The Bond) ✅

- **Biological Bonding**: Implementation of physical attachment between entities via the `Bond` neural output.
- **Kinematic Coupling**: Bonded pairs move as a unified physics body (Spring-mass damper logic).
- **Metabolic Fusion**: Bidirectional energy equalization (not just one-way donation) to create true shared organisms.
- **Bond Maintenance**: Distance-based bond integrity checks (Break if dist > 20.0).
- **Specialized Roles**: Emergence of "Pilot" (Movement specialist) and "Turret" (Defense specialist) pairs.

### Phase 52: Emergent Architecture (Terraforming) ✅

- **Active Terrain Modification**: Added `Dig` and `Build` neural outputs allowing entities to reshape the world.
- **Hydrological Engineering**: Construction of canals (River expansion) that boost local soil fertility via hydration coupling.
- **Nest Construction (Ω)**: Protective structures that provide metabolic idle reduction and energy bonuses for offspring.
- **Ecological Feedback**: Biological terraforming directly influencing biome succession (e.g., turning desert to plains via canal irrigation).

### Phase 53: Specialized Castes & Behavioral Metering ✅

- **Specialization Meters**: Entities evolve specialized roles—**Soldier**, **Engineer**, or **Provider**—based on their lifetime neural activity.
- **Role Bonuses**: Engineers have 50% lower terraforming costs; Soldiers inflict 1.5x damage; Providers share energy with 50% less metabolic loss.
- **Genetic Bias**: Inheritable predispositions towards specific castes, allowing lineages to evolve stable social structures.

### Phase 54: Interspecies Symbiosis & Hybridization ✅

- **Mutualistic Bonds**: Extended bonding to support inter-lineage partnerships with shared metabolic bonuses.
- **Interspecies Hybridization**: Bonded partners of different lineages can reproduce sexually, enabling horizontal gene transfer and hybrid vigor.
- **River Dynamics**: Implemented evaporation in low-fertility zones to balance biological canal engineering.

### Phase 55: Parasitic Manipulation & Behavioral Hijacking ✅

- **Neural Hijacking**: Advanced pathogens can force specific brain outputs (e.g., forced aggression, vocalization) to facilitate their spread.
- **Pathogen Evolution**: Viral strains mutate their manipulation targets, creating dynamic behavioral epidemics.
- **Compressed Fossil Record**: Transitioned to Gzip-compressed fossil storage (`fossils.json.gz`) for 60% disk savings.

### Phase 56: Atmospheric Chemistry (Gas Exchange) ✅

- **Oxygen Cycle**: Implemented Oxygen level tracking coupled to photosynthesis (Forests) and metabolism (Entities).
- **Hypoxic Stress**: Low oxygen levels (< 8%) induce metabolic energy drain.
- **Aerobic Efficiency**: High oxygen levels boost movement speed and efficiency.
- **Atmospheric Displacement**: High CO2 levels slightly displace Oxygen, linking climate change to respiratory stress.

### Phase 57: Neural Archiving (Brain Export) ✅

- **JSON Brain Export**: Added `Shift+C` command to export the full neural graph of the selected entity to `logs/brain_<id>.json`.
- **Archival Compatibility**: Brain exports include all node topologies, connection weights, and recurrence states for external analysis.

### Phase 58: Complex Life Cycles (Metamorphosis) ✅

- **Larval Stage**: Juvenile organisms with restricted behavioral outputs.
- **Metamorphosis Trigger**: Physical transformation at 80% maturity.
- **Neural Remodeling**: Automated connection of adult behavioral nodes.
- **Physical Leap**: One-time somatic buffs to energy, speed, and sensing.

### Phase 59: Divine Research & Multiverse Trade ✅

- **Genetic Engineering UI**: Real-time genotype editing for selected entities.
- **Multiverse Market**: P2P resource exchange (Energy, Oxygen, Biomass, Soil).
- **Synaptic Plasticity Tools**: Visualizing Hebbian learning deltas in real-time.
- **Unified Trade Engine**: Centralized resource management across all simulation tiers.

### Phase 60: Macro-Evolutionary Intelligence & Global Cooperation ✅

- **Lineage-Wide Coordination**:
    - Functional: Implement `LineageGoal` registry to synchronize behavioral biases (e.g., "Expand West") across distributed clusters.
    - Technical: Neural input for `ClusterCentroid` and `GoalVector`.
- **Global Altruism Networks**:
    - Functional: P2P lineage-based energy relief protocols.
    - Technical: `TradeMessage::Relief` for non-reciprocal energy transfer to struggling kin in other universes.
- **Biological Irrigation**:
    - Functional: Emergent canal networks for global fertility stabilization.
    - Technical: Entities with Engineer caste prioritize connecting isolated `River` cells to `Desert` biomes.
- **Civilization Seeds**:
    - Functional: Transition from individual survival to collective engineering.
    - Technical: Implement `Structure::Outpost` which acts as a permanent pheromone relay and energy capacitor.

### Phase 61: Evolutionary Ecology & Civilizational Tiers ✅

- **Ancestral Traits & Epigenetics**:
    - Functional: High-fitness lineages accumulate "Ancestral Traits" that persist through mass extinctions.
    - Technical: Implement trait persistence in `LineageRecord` with metabolic cost scaling.
- **Global Peer Events**:
    - Functional: Real-time environmental crises synchronized across the Hive network (e.g., "Solar Flare").
    - Technical: `NetMessage::GlobalEvent` propagation with deterministic seed synchronization.
- **Civilization Leveling**:
    - Functional: Tribes that build connected Outpost networks gain civilization bonuses (e.g., shared energy pool).
    - Technical: Graph-based connectivity check for Outposts in `World::update`.
- **Neural Specialization (Phase 2)**:
    - Functional: Castes evolve distinct neural sub-modules (e.g., Soldier-only hidden layer paths).
    - Technical: Topology-restricted mutations based on `Specialization`.

### Phase 62: Planetary Engineering & Hive Mind Synergy ✅

- **Atmospheric Engineering**:
    - Functional: Dominant lineages influence global climate via forest management.
    - Technical: Owned `Forest` cells near `Outposts` sequestrate CO2 at 2.5x rate.
- **Outpost Power Grid (Civ Level 2)**:
    - Functional: Connected outposts share energy stores across the network.
    - Technical: BFS-based connectivity check for Outposts linked by `River` (Canal) cells.
- **Functional Neural Modules**:
    - Functional: Castes develop "Protected Clusters" in their brain that resist destructive mutation.
    - Technical: Implementation of mutation-resistant weight sets based on specialization-driven activity.
- **Hive Perception**:
    - Functional: Entities sense the macro-state of their entire lineage.
    - Technical: Neural inputs for `LineageGlobalPop` and `LineageGlobalEnergy`.

### Phase 63: Civilizational Specialization & Resource Pipelining ✅

- **Outpost Specialization**:
    - Functional: Outposts can evolve into **Silos** (high energy cap) or **Nurseries** (birth energy bonus).
    - Technical: Specialization state in `TerrainCell` influenced by nearby entity activity.
- **Resource Pipelining**:
    - Functional: Long-distance energy transfer through the Power Grid.
    - Technical: Implemented "Flow" logic between outposts in the same connected component.
- **Hive Overmind Broadcast**:
    - Functional: High-rank Alphas can broadcast a "Goal Pheromone" that overrides kin movement.
    - Technical: 12th neural output for `OvermindSignal` and 28th input for `BroadcastVector`.
- **Ecosystem Dominance**:
    - Functional: Tribes with level 3 civilizations gain global albedo control (cooling).
    - Technical: Global climate forcing based on total owned forest area.

### Phase 64: Genetic Memory & Evolutionary Rewind ✅

**Goal:** Deepen biological realism through temporal genetic mechanisms.

- **Genotype Checkpointing**:
    - Functional: Lineages automatically archive the "All-Time Best" genotype in their shared memory.
    - Technical: Track `max_fitness_genotype` in `LineageRecord`.
- **Atavistic Recall**:
    - Functional: Struggling entities have a small chance to revert to an ancestral successful genotype (Rewind).
    - Technical: Mutation variant that replaces current brain with the checkpointed brain.

### T1: Architectural Decoupling & Foundation Refactor 🏗️

- **Goal**: Achieve a "Perfect" separation of Data, Logic, IO, and Presentation.
- **Progress**:
    - ✅ **Shared Definitions**: Created `defs.rs` to break circular dependencies between Entity, Intel, and LineageRegistry.
    - ✅ **Deterministic Foundation**: Implemented seeded RNG and parallel determinism for robust simulation replay.
    - 🚧 **Data-Logic Split**: Moving towards ECS (Phase 66).

### Phase 65: The Silicon Scribe (LLM Integration) 🚀

**Goal:** Ultimate Observability regarding "Why did this happen?".

- **Narrator System**:
    - Functional: Natural language event logs describing epic moments (e.g., "The Red Tribe migrated south due to famine").
    - Technical: Async Rust bindings to local LLM (e.g., Llama 3) encapsulated in **`primordium_observer`** to prevent core bloat.
- **Analyst Agent**:
    - Functional: RAG system allowing users to query simulation history.
    - Technical: Vector database integration for `logs/history.jsonl`.
- **Interactive Query**:
    - Functional: "Show me the lineage that survived the Great Drought."
    - Technical: Natural Language to SQL/Filter converter for `primordium-analyze`.

### Phase 66: Data-Oriented Core (ECS Refactor) ⚡

**Goal:** Maximize CPU cache localization and parallelism.

- **Step 1: The Component Split**:
    - Functional: Decompose the monolithic `Entity` struct.
    - Technical: Create atomic components: `Position`, `Velocity`, `Brain`, `Metabolism`, `Genotype`.
- **Step 2: The Archetype Migration**:
    - Functional: Optimize memory layout for different entity types (e.g. `Food` vs `Organism`).
    - Technical: Adopt `hecs` or `bevy_ecs` to manage SoA (Structure of Arrays) storage efficiently.
- **Step 3: System Parallelism**:
    - Functional: Fearless concurrency for massive scale.
    - Technical: Use explicit queries like `Query<(&mut Position, &Velocity)>` to remove `RwLock` contention.
- **Step 4: Zero-Copy Serialization**:
    - Functional: Instant simulation saves and network transfers.
    - Technical: Implement `rkyv` for memory-mapped persistence of **component tables** (Archetypes).

### Phase 67: The Creator's Interface (Plugin Architecture) 🧩

**Goal:** Community extensibility and modding support.

- **WASM Plugin Host**:
    - Functional: Users can write custom `Systems` (e.g., new disease logic) in Rust/WASM.
    - Technical: `wasmer` or `wasmtime` integration to run sandboxed systems during `World::update`.
- **Lua Scripting**:
    - Functional: Lightweight scripting for "Disaster Scenarios" or level design.
    - Technical: `mlua` integration for triggering events based on world state conditions.
- **Mod Loader**:
    - Functional: Simple CLI to load/unload mods.
    - Technical: `mods/` directory scanner and dependency resolution.

### 🎨 Creative Construction

*Focus on the artistic and sensory experience.*

- **Phase 68: The Song of Entropy (Procedural Audio)** 🎵
    - **Goal**: Hear the state of the world.
    - **Features**:
        - `Entropy Synth`: Sound generation driven by global system entropy.
        - `Event SFX`: Spatial audio for predation and birth.
        - `Bio-Music`: Dominant lineage genomes converted to melody.

- **Phase 69: Visual Synthesis (ASCII Raytracing)** 👁️
    - **Goal**: Push the limits of the terminal.
    - **Features**:
        - `SDF Rendering`: Signed Distance Field rendering for "blobs" in TUI.
        - `Glow Effects`: Simulated CRT bloom using RGB colors.

### 🌐 Ecosystem Expansion

*Focus on platform reach and developer integration.*

- **Phase 70: The Galactic Federation (Central Server)** 🏛️
    - **Goal**: A persistent, shared multiverse.
    - **Features**:
        - `Global Registry`: Permanent storage of "Hall of Fame" genomes.
        - `Marketplace`: Exchange "Seeds" (Simulation Configs) and "Specimens".

---

## 🏗️ Technical Evolution

> **"Code is not just functionality; it is the literature of logic."**

These parallel workstreams focus on the long-term health, stability, and developer experience of the Primordium engine.

### T1: Architectural Decoupling (The Hexagonal Refactor) 🧱
- **Goal**: Achieve a "Perfect" separation of Data, Logic, IO, and Presentation.
- **Tasks**:
    - **`primordium_data`** (The Atom):
        - *Role*: Pure Data Structs (POD) shared by Tools, SDKs, and Engine.
        - *Content*: `EntityData`, `Genotype`, `PhysicsState`.
        - *Dependencies*: `serde`, `uuid`. NO Logic.
    - **`primordium_core`** (The Engine):
        - *Role*: Deterministic Simulation Logic.
        - *Content*: `Systems`, `World::update()`.
        - *Constraints*: `no_std` compatible, WASM-pure. NO Disk/Net I/O.
    - **`primordium_io`** (The Scribe):
        - *Role*: Persistence and Logging.
        - *Content*: `HistoryLogger`, `FossilRegistry`, `SaveManager`.
        - *Why*: Isolates heavy I/O from the lightweight Core.
    - **`primordium_driver`** (The Contract):
        - *Role*: Trait definitions for Hardware Abstraction (`Renderer`, `Input`).
        - *Why*: Enables swapping TUI for WebCanvas or Headless without touching App logic.
    - **`primordium_net`** (The Voice):
        - *Role*: P2P Networking implementation.
        - *Dependencies*: `primordium_data`, `tokio`.
    - **`primordium_tui`** (The Lens):
        - *Role*: TUI implementation of `primordium_driver`.
    - **`primordium_app`** (The Glue):
        - *Role*: Composition Root (`main.rs`) that wires Drivers to Core.
    - **`primordium_tools`** (The Toolkit):
        - *Role*: CLI utilities for data analysis and verification.
        - *Binaries*: `analyze` (from `src/bin/analyze.rs`), `verify` (from `src/bin/verify.rs`).
        - *Dependencies*: `primordium_data`, `primordium_io`.
    - **`primordium_server`** (The Nexus):
        - *Role*: Dedicated backend binary for the Galactic Federation.
        - *Content*: Current `src/server/main.rs` logic.
        - *Dependencies*: `primordium_net`, `axum`, `tokio`.

### T2: Continuous Evolution (CI/CD Pipeline) 🔄
- **Goal**: Automate quality assurance.
- **Tasks**:
    - `test.yml`: Run `cargo test` on every push.
    - `release.yml`: Auto-build binaries for Linux/MacOS/Windows on semantic tags.
    - `audit.yml`: Weekly security scan with `cargo audit`.
    - `clippy.yml`: Enforce strict linting rules on PRs.

### T3: The Testing Gauntlet 🛡️
- **Goal**: Catch distinct edge cases and ensure deterministic simulation.
- **Tasks**:
    - **Property Testing**: Integrate `proptest` to fuzz `physics` and `collision` logic with millions of inputs.
    - **Determinism Check**: Regression test suite ensuring that `seed_A` always produces `history_A` across platforms.
    - **Long-Haul Tests**: 24-hour runtime verification to check for memory leaks and numerical drift.

### T4: Knowledge Preservation (Documentation) 📚
- **Goal**: Move beyond markdown files to a searchable knowledge base.
- **Tasks**:
    - Setup `mdBook` framework for documentation.
    - Compile `ARCHITECTURE.md`, `AGENTS.md`, and `CHANGELOG.md` into a static site.
    - Generate `cargo doc` pages for the internal API.
    - Deploy to GitHub Pages.

---

## 🌱 Philosophy

Primordium is an experiment in **emergent complexity**. You provide the rules, the hardware provides the pressure, and evolution writes the story.

Every run is unique. Every lineage is precious. Every extinction teaches us something.

*Last updated: 2026-01-27*
*Version: 0.0.1*
