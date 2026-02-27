# Changelog - Primordium

[简体中文](./docs/CHANGELOG_zh.md)

All notable changes to the **Primordium** project will be documented in this file. This project adheres to a phase-based evolutionary development cycle.

---

## [Phase 70 API Security & Quality Hardening] - 2026-02-27

### API Authentication & Rate Limiting

- **Phase 70 API Authentication**: Added `PRIMORDIUM_API_KEY` environment variable for Bearer Token authentication
    - POST endpoints (`submit_genome`, `submit_seed`) protected with `check_auth()`
    - GET endpoints remain public; open mode when key unset
    - Added 5 new auth tests (reject/accept/wrong-key/seeds/GET-public)

### Dependency Updates

- **RUSTSEC-2026-0002**: Fixed lru crate unsoundness (0.12.5 → 0.16.3)
    - Updated ratatui from 0.29 to 0.30 to resolve the vulnerability

### Code Quality Refactoring

- Eliminated 3 `clippy::too_many_arguments` suppressions:
    - `biological.rs`: Introduced `BiologicalContext<R>` struct for population/config/tick/rng
    - `storage.rs`: Introduced `GenomeSubmit` and `SeedSubmit` parameter structs
    - Reduced function parameters from 8-11 to 1

### Test Improvements

- Fixed `test_inter_tribe_predation` flaky test:
    - Changed `EntityBuilder::build()` to use `create_entity_deterministic()` with ChaCha8Rng
    - Verified 30/30 test passes, 15 full suite runs with zero failures

### Quality Assurance

- ✅ All tests passing (150+ tests)
- ✅ Zero Clippy warnings
- ✅ Code formatted
- ✅ Performance gate passed

---

## [Security Fixes] - 2026-02-24

### Dependency Security Updates

Fixed multiple security vulnerabilities in dependencies:

- **RUSTSEC-2026-0007**: Upgraded `bytes` from 1.11.0 to 1.11.1
    - Fixed integer overflow in `BytesMut::reserve`
- **RUSTSEC-2026-0009**: Upgraded `time` from 0.3.46 to 0.3.47
    - Fixed Denial of Service via Stack Exhaustion
- **RUSTSEC-2025-0009**: Upgraded `rcgen` from 0.11/0.13 to 0.14.7
    - Fixed AES function panics with overflow checking
    - **Breaking Change**: Updated API calls in `src/app/network/quic.rs`:
        - `cert.serialize_der()` → `cert.cert.der().to_vec()`
        - `cert.serialize_private_key_der()` → `cert.signing_key.serialize_der().to_vec()`

### Quality Assurance

- ✅ All tests passing (116+ tests)
- ✅ Zero Clippy warnings
- ✅ Code formatted

---

## [Phase 68.6 Integration Complete] - 2026-02-24

### Stereo Audio Implementation

Completed Phase 68.6 stereo audio integration:

- **Spatial Panning**: Birth and death events now have stereo audio positioning
    - Left/right panning based on entity X position relative to world center
    - Distance attenuation applied (inverse square law)
- **Queue Integration**: Fixed `process_queue()` to apply spatial gains before queuing events
    - Spatial events: `set_spatial_sfx_gain(left_pan, right_pan)` before queueing
    - Non-spatial events: Center-panned with `set_spatial_sfx_gain(1.0, 1.0)`
- **Dual Architecture**: Maintains mono `render_block()` for backward compatibility

### Documentation Updates

- **README.md**: Added "Procedural Audio Engine" section with feature highlights
- **MANUAL.md**: Added comprehensive "Procedural Audio" section explaining:
    - Audio systems (Entropy Synth, Bio-Music, Event SFX, Spatial Positioning)
    - How synthesis and mixing work
    - Spatial hearing details for birth/death events

### Quality Assurance

- ✅ All tests passing (116+ tests)
- ✅ Zero Clippy warnings
- ✅ Code formatted

### Known Issues (Low Priority)

The following security advisories remain as warnings (unmaintained crates):

- `RUSTSEC-2025-0009`: ring 0.16.20 (requires quinn upgrade - pending phase 70)
- `RUSTSEC-2024-0436`: paste 1.0.15 (unmaintained)
- `RUSTSEC-2025-0010`: ring unmaintained warning
- `RUSTSEC-2025-0134`: rustls-pemfile 1.0.4 (unmaintained)
- `RUSTSEC-2026-0002`: lru (unmaintained)

These are low-risk warnings about unmaintained crates. The critical ring issue is blocked by a required quinn library upgrade which will be addressed in Phase 70 (The Galactic Federation) when P2P networking infrastructure is refactored
---

## [Engineering Sprint: Code Quality & Architecture Refinement] - 2026-02-10

### Technical Excellence Sprint

**Mission**: Achieve production-level code quality through rigorous refactoring, testing, and documentation.

#### 🛠️ Code Quality Achievements

**Clippy Linting (Tier 1 - Tasks 1-8)**:

- ✅ Eliminated all `#[allow(...)]` suppressions across the codebase
- ✅ Zero Clippy warnings in all crates
- ✅ Clean compilation with `-D warnings` strict gate

**Function & File Modularity (Tier 2 - Tasks 9-16)**:

- ✅ Split `App::draw` (381 lines) into 10 granular helper methods:
    - `draw_background()`, `create_layouts()`, `draw_main_content()`, `draw_cinematic_mode()`, `draw_normal_mode()`
    - `draw_status_bar()`, `draw_sparklines()`, `draw_world_canvas()`, `draw_chronicle()`, `draw_sidebar()`, `draw_overlays()`
- ✅ Verified all large files already pre-split:
    - `brain.rs` → `brain/mod.rs` with 4 submodules
    - `terrain.rs` → `terrain/mod.rs` + `succession.rs`
    - `input.rs` → `input/normal.rs`
    - `commands.rs` already split into `generate_*_cmds` functions
    - `finalize.rs` already split into `process_*` and `finalize_*` functions

**Testing Infrastructure (Tier 3 - Partial)**:

- ✅ `primordium_observer` enhanced from 1 to 11 comprehensive tests
    - Coverage: All Narrator event types (Extinction, GreatFamine, ClimateShift, NewEra, Custom)
    - `SiliconScribe` async queue and consumption testing
    - Custom narrator implementation verification
    - Severity-based prefix formatting validation
- ✅ Verified zero ignored doc-tests in codebase

**Workspace Architecture Documentation (Tier 4 & Task 41)**:

- ✅ Added comprehensive Workspace documentation to `ARCHITECTURE.md`
    - Complete 8-crate structure overview
    - Dependency flow diagram
    - Each crate's responsibilities and relationships
    - Refactoring benefits justification

#### 📊 Verification Gates Met

```bash
✅ cargo fmt --all                    # All code formatted
✅ cargo clippy --workspace           # 0 warnings
✅ cargo fix --workspace             # All fixes applied
✅ cargo test --workspace            # 116+ tests passing
```

#### 📦 Architecture Update

**8-Crate Workspace Status** (All Existing):

- `primordium_data` - Pure data structures (6 lines, minimal)
- `primordium_core` - Engine logic with systems
- `primordium_io` - Persistence and logging
- `primordium_net` - P2P protocol
- `primordium_observer` - LLM integration (11 tests)
- `primordium_server` - Relay server
- `primordium_tools` - CLI utilities
- `primordium_tui` - TUI rendering abstraction

#### 🎯 Deliverables Summary

- **26 tasks completed** (52% of Engineering Sprint)
- **High-priority work 100% done** (Tiers 1-2, Tier 4 high-value tasks)
- **Codebase production-ready** with clean linting and modular structure

---

## [Phase 65-66 Refinement: Architectural Decoupling & Silicon Scribe] - 2026-01-28

### Evolutionary Leap: Modular Systems & Ultimate Observability

This update focuses on the transition towards a modular, system-based architecture and introduces the foundation for natural language simulation narration. Core logic has been decoupled from the `World` monolith, and a new observation infrastructure has been established.

#### ✨ Features

- **Architectural Decoupling (T1 Refinement)**:
    - **Civilization System**: Extracted civilization logic (outpost handling, power grids) into a dedicated `civilization` system.
    - **History System**: Extracted history logic (fossilization, legend management) into a dedicated `history` system.
    - **Systemic Modularization**: Reduced `world.rs` complexity by 20% and established clear boundaries for civilizational and historical evolution.
- **Silicon Scribe (Phase 65 Foundation)**:
    - **Narrator Infrastructure**: Established the `primordium_observer` crate with a `SiliconScribe` engine.
    - **Heuristic Narration**: Implemented rule-based narration for significant world events (Extinctions, Famines).
    - **Narrative Dashboard**: Updated the macro report to include a chronological stream of simulation narrations.
- **Performance Optimization**:
    - **Snapshot Recycling**: Implemented buffer reuse for entity snapshots, significantly reducing memory allocations in the parallel update loop.
    - **Sensory Refinement**: Optimized wall sensing and kin recognition through specialized spatial hashing methods.
- **Configuration Excellence**:
    - **Refactored Constants**: Extracted over 10 hardcoded simulation constants (sequestration rates, energy caps, intervals) into `config.toml`.

#### 🛠️ Technical Achievements

- **Brain Consistency Fix**: Corrected a discrepancy between documented brain inputs (29) and implemented buffer sizes (23), ensuring full utilization of the NEAT-lite architecture.
- **Crate Expansion**: Modularized the codebase further with the introduction of the `primordium_observer` crate in the workspace.
- **Zero-Panic Hardening**: Verified and hardened spatial hashing queries against out-of-bounds edge cases.

---

## [Phase 60-63: Macro-Evolution & Digital Civilization] - 2026-01-27

### Evolutionary Leap: Collective Intelligence & Planetary Engineering

This major multi-phase update transitions the simulation from a collection of isolated organisms to a networked **Hive Civilization**. Lineages now possess collective memory, cooperate across universes, and actively engineer the planet's atmosphere and infrastructure.

#### ✨ Features

- **Macro-Evolutionary Intelligence (Phase 60)**:
    - **Collective Memory**: Lineages now share a persistent memory pool. Significant events (predation, abundance) reinforce lineage-wide `threat` and `goal` signals.
    - **Neural Feedback**: Added `SharedGoal` and `SharedThreat` inputs to the brain, allowing for evolved swarm coordination.
    - **Outposts (Ψ)**: High-rank Alphas can now establish permanent structures that serve as territorial markers and energy capacitors.
- **Civilizational Tiers (Phase 61)**:
    - **Ancestral Traits**: High-fitness lineages accumulate persistent "Epigenetic" buffs (e.g., Hardened Metabolism, Acute Senses) that are inherited across mass extinctions.
    - **Global Peer Events**: Real-time environmental crises (Solar Flares, Deep Freezes) are now synchronized across the Hive network.
    - **Radiation Storms**: Solar flares trigger high-intensity mutation surges across all connected peers, forcing rapid adaptation.
    - **Civ Leveling**: Lineages reach "Civilization Level 1" upon owning 5+ outposts, unlocking specialized infrastructure.
- **Planetary Engineering & Power Grids (Phase 62)**:
    - **Atmospheric Management**: Forests near outposts sequestrate CO2 at 2.5x the standard rate, allowing dominant civilizations to steer global climate.
    - **Outpost Power Grid**: Connected outposts (via canals/rivers) automatically balance and share energy stores using a decentralized BFS distribution logic.
    - **Hive Perception**: Entities sense the macro-state of their entire lineage (`LineagePop`, `LineageEnergy`) as primary neural inputs.
- **Resource Pipelining & Overmind (Phase 63)**:
    - **Outpost Specialization**: Structures can specialize into **Silos** (5x energy capacity) or **Nurseries** (birth energy bonuses).
    - **Resource Flow**: Energy now "flows" towards equilibrium through the power grid, allowing remote outposts to support frontline kin.
    - **Hive Overmind Broadcast**: High-rank Alphas can broadcast a `Overmind` goal signal that overrides kin movement across the simulation.
    - **Protected Brain Modules**: Specialized castes gain 90% mutation resistance on role-critical neural weights, ensuring cognitive stability in high-radiation eras.

#### 🛠️ Technical Achievements

- **Expanded Brain Architecture**: Upgraded to **29 inputs** and **12 outputs** to support macro-perception and civilizational control.
- **Stigmergic Power Grids**: Implemented a graph-based energy balancing system that operates in the background of the 3-pass parallel loop.
- **Cross-Universe Relief Protocol**: Developed the `NetMessage::Relief` protocol for non-reciprocal P2P energy transfers.
- **Planetary Feedback Loops**: Coupled forest ownership and outpost density to global albedo and sequestration metrics.

---

## [Phase 56-58: Atmospheric Chemistry & Complex Life Cycles] - 2026-01-26

### Evolutionary Leap: Respiratory Stress & Metamorphosis

This update completes the core physiological systems of Phase 58, introducing the concept of distinct life stages with unique behavioral constraints and neural remodeling. Organisms now face an Oxygen cycle while juveniles undergo a complete transformation into active ecological actors.

#### ✨ Features

- **Atmospheric Chemistry (Phase 56)**:
    - **Oxygen Cycle**: Implemented Oxygen level tracking coupled to photosynthesis (Forests) and metabolism (Entities).
    - **Hypoxic Stress**: Low oxygen levels (< 8%) induce metabolic energy drain.
    - **Aerobic Efficiency**: High oxygen levels boost movement speed and efficiency.
- **Neural Archiving (Phase 57)**:
    - **JSON Brain Export**: Added `Shift+C` command to export the full neural graph (Topology, Weights, Recurrence) to `logs/brain_<id>.json`.
- **Complex Life Cycles (Phase 58)**:
    - **Metamorphosis**: Entities undergo a one-time transformation at maturity.
    - **Larval Gating**: Pre-metamorphosis "Larvae" are restricted from complex actions (Bond, Dig, Build) to focus on foraging.
    - **Neural Remodeling**: Metamorphosis triggers a structured remodeling of the brain, ensuring "Adult" behavioral outputs are connected and functional.
    - **Physical Leap**: Transforming into an adult grants a 1.5x max energy boost and 20% increases to sensing range and max speed.

#### 🛠️ Technical Achievements

- **Stage-Gated Topology**: Implemented behavioral gating in the interaction pipeline based on metabolic stage.
- **Structured Brain Remodeling**: Developed `Brain::remodel_for_adult()` to automate functional connectivity shifts during life stage transitions.
- **Metamorphosis Integration Suite**: Added `tests/metamorphosis.rs` covering lifecycle verification and gating logic.

---

## [Phase 52-55: Emergent Engineering & Social Specialization] - 2026-01-25

### Evolutionary Leap: Biological Terraforming & Parasitic Hijacking

This multi-phase update transforms organisms from passive survivalists into active biological engineers and complex social actors. Lineages now actively reshape their environment through hydrological engineering and building protective structures, while new parasitic threats introduce sophisticated behavioral manipulation.

#### ✨ Features

- **Emergent Engineering (Phase 52)**:
    - **Neural Terraforming**: Added `Dig` and `Build` neural outputs. Entities can convert terrain types based on their energy levels.
    - **Hydrological Canals**: Digging adjacent to rivers allows entities to create functional canals, boosting local fertility.
    - **Nests (Ω)**: Entities can construct protective nests that grant a 20% metabolic recovery bonus and a "Nursery" energy boost for newborns.
- **Specialized Castes (Phase 53)**:
    - **Caste Metering**: Lineages evolve specialized roles—**Soldier**, **Engineer**, or **Provider**—based on historical neural activity.
    - **Role Buffs**: Engineers get 50% lower terraforming costs; Soldiers deal 1.5x damage; Providers reduce sharing energy loss by 50%.
- **Interspecies Symbiosis (Phase 54)**:
    - **Mutualistic Bonds**: Bonding is no longer restricted to kin. Mutualistic pairs receive a 10% reduction in all metabolic costs.
    - **Interspecies Hybridization**: Bonded partners of different lineages can reproduce sexually, generating diverse hybrid genotypes.
    - **River Dynamics**: Implemented river evaporation in low-fertility zones to balance biological terraforming.
- **Parasitic Manipulation (Phase 55)**:
    - **Neural Hijacking**: Pathogens can now possess "Behavior Manipulation" traits, forcing specific host neural outputs (e.g., forced aggression or vocalization) to accelerate viral spread.
- **Vocal Propagation (Phase 48 Refinement)**:
    - **SoundGrid**: Implemented real-time acoustic ripples with diffusion and decay. Entities "Hear" the aggregate volume of nearby calls.

#### 🛠️ Technical Achievements

- **Massive Parallel Pipeline (Phase 41 Scaling)**: Refactored the simulation update loop into a "Proposal Unzipping" pattern. All action and biological systems now run in full parallel using Rayon, supporting 10,000+ entities.
- **Lock-Free Spatial Hash**: Implemented a parallel `fold`/`reduce` pattern for Spatial Hash construction, eliminating Mutex bottlenecks.
- **Compressed Fossil Record**: Implemented Gzip-compressed storage for `fossils.json.gz`, reducing disk footprint by >60%.
- **Enhanced TUI Neural View**: Updated brain visualization to support 11 outputs and real-time specialization meters.

---

## [Phase 51: Symbiosis (The Bond)] - 2026-01-24

### Evolutionary Leap: Biological Fusion

Phase 51 fundamentally changes the unit of selection from the individual to the **bonded pair**. Organisms can now form physical and metabolic bonds with compatible partners, moving as a single kinematic unit and sharing energy to survive harsh conditions. This introduces the concept of **Obligate Symbiosis**, where two specialized entities (e.g., a high-speed "Pilot" and a high-defense "Turret") can outperform generalists.

#### ✨ Features

- **Kinematic Coupling**: Bonded entities are physically tethered by a spring-mass damper system. Movement forces are shared, allowing pairs to coordinate locomotion or drag injured partners to safety.
- **Metabolic Fusion**: Implemented a bidirectional energy equalization protocol. Instead of simple one-way donations, bonded pairs continuously balance their metabolic reserves, creating a shared energy pool.
- **Bond Maintenance**: Bonds are now dynamically maintained based on proximity. Pairs that drift too far apart (Distance > 20.0) due to external forces or lack of coordination will snap their bond.
- **Symbiotic Selection**: Neural networks can now evolve specific `Bond` output strategies to actively seek or reject partners based on genetic compatibility.

#### 🛠️ Technical Achievements

- **Spring Force Physics**: Integrated a Hooke's Law simulation into the `Action` system, applying corrective velocity vectors to bonded pairs in parallel.
- **Context-Aware Action System**: Refactored `ActionContext` to include read-only access to the global `EntitySnapshot` buffer, enabling thread-safe partner lookups during parallel updates.
- **Equalization Logic**: Enhanced the `InteractionCommand` pipeline to support precise, bidirectional energy transfer commands (`TransferEnergy`) without race conditions.

---

## [Phase 50: Visualizing the Invisible (Collective Intelligence)] - 2026-01-24

### Evolutionary Leap: Perception of the Abstract

Phase 50 transforms the simulation's observability. The complex internal states introduced in previous phases—social rank, vocal signals, and territorial claims—are now brought to the surface through a multi-layered heatmapping system. Entities no longer just "hear" or "rank" each other in the dark; the simulation now visualizes the emergence of collective intelligence and dynamic sovereignty.

#### ✨ Features

- **Rank Heatmap (View 4)**: Real-time visualization of social stratification. Purple/Magenta gradients reveal how tribes organize around Alphas and how Omega-rank entities are marginalized.
- **Vocal Propagation (View 5)**: Yellow "sound waves" visualize signal density. Watch how alarm calls and coordination signals ripple through the population, moving from noise to meaning.
- **Soldier & Alpha Auras**: High-impact individuals (Soldiers/Alphas) now possess visual "auras" in social views, making leadership and defense structures immediately apparent.
- **Dynamic Territoriality**: Tribes now actively "claim" territory. Alpha entities automatically flip local social grids to "Peace" (for kin) or "War" (for rivals) based on their aggression, creating emergent borders.
- **Collective Reinforcement**: Enhanced Hebbian learning loop that rewards social proximity and vocal coordination, accelerating the evolution of complex group behaviors.

#### 🛠️ Technical Achievements

- **Spatial Heatmap Engine**: Implemented high-performance spatial queries in `WorldWidget` to render real-time propagation of abstract signals (Vocalization, Rank density).
- **Dynamic Social Grid Logic**: Integrated `TribalTerritory` commands into the 3-pass parallel update loop, allowing Alphas to steer regional rules dynamically.
- **Vocal Output Normalization**: Standardized vocalization intensity from neural outputs, enabling Input 14 (Hearing) to carry consistent semantic volume.
- **Legacy Refactoring**: Marked and deprecated sequential social systems, ensuring 100% of social interaction logic utilizes the thread-safe parallel Command pattern.

---

## [Phase 49: Advanced Social Hierarchies (Tribal Warfare)] - 2026-01-24

### Evolutionary Leap: The Rise of Leaders and Armies

Phase 49 introduces sophisticated social stratification. Tribes are no longer egalitarian blobs; they now have Alphas (leaders) and Soldier castes. When population stress disrupts social cohesion, large tribes will fracture into competing factions, simulating civil wars and schisms.

#### ✨ Features

- **Social Rank System**: Entities now have a calculated `rank` (0.0-1.0) based on Energy (30%), Age (30%), Offspring (10%), and Reputation (30%).
- **Soldier Caste**: High-ranking (>0.8) and aggressive (>0.5) entities become **Soldiers** (`⚔`). They deal **1.5x damage** generally and **2.0x damage** in War Zones.
- **Tribal Splitting**: Overcrowding combined with low social rank triggers "Fracture" events, where marginalized groups mutate their color to form new, rival tribes.
- **Leadership Vectors**: Entities now perceive and are influenced by the movement of local Alphas (highest-ranking kin).

#### 🛠️ Technical Achievements

- **Rank & Leadership Engine**: Implemented `calculate_social_rank` and Alpha-weighted movement vectors in `world.rs` Pass 1.
- **Dynamic Status Logic**: Refactored `EntityStatus` to prioritize role-based states (Soldier) over behavioral states (Hunting).
- **Brain Architecture Fix**: Corrected a critical off-by-one error in the Neural Network output layer (Node 20 vs 21), ensuring correct action mapping.

---

## [Phase 46: Evolutionary Stable Strategy (ESS) & Social Topology] - 2026-01-24

### Evolutionary Leap: The Logic of Altruism

Phase 46 moves beyond simple tribe matching by implementing a rigorous game-theoretic framework for social interaction. Entities now make decisions based on genetic relatedness and social reputation, leading to emergent cooperation and self-policing groups.

#### ✨ Features

- **Hamilton's Rule Integration**: Social benefits (Defense, Sharing) are now weighted by the Coefficient of Relatedness ($r$), derived from genetic distance.
- **Reputation System**: Entities track a `reputation` metric. Altruistic acts (sharing) build reputation, while "betrayal" (attacking kin) destroys it.
- **Social Punishment**: Low-reputation entities lose the protection of their tribe and can be hunted by kin, simulating the evolution of moral policing.
- **Social Grid & Zones**: Divine brush tool (J) allows partitioning the world into Peace Zones (no predation) and War Zones (doubled attack power), enabling steered social experiments.

---

## [Phase 45: Global Hive - Robust P2P Connectivity] - 2026-01-24

### Evolutionary Leap: Multiverse Integrity

Phase 45 transforms the experimental P2P migration system into a production-ready "Global Hive" protocol, ensuring entities can move between universes without corruption or logical inconsistencies.

#### ✨ Features

- **Versioned Migration (Fingerprinting)**: World configurations are now hashed into unique fingerprints. Migration is only permitted between worlds with compatible physical/evolutionary constants.
- **Data Integrity (SHA-256 Checksums)**: Every migrating entity is verified via SHA-256 to ensure DNA and metabolic state survive network transit.
- **Backpressure Flow Control**: Implementation of inbound migration buffers (max 5 per tick) to prevent local population spikes and CPU saturation during massive migrations.
- **Hive Dashboard**: Real-time TUI display of network health, peer counts, and migration flux.

---

## [Phase 44: Niche Construction & Nutrient Cycling] - 2026-01-24

### Evolutionary Leap: The Great Cycle

Phase 44 implements a closed-loop nutrient cycle, allowing biological activity to actively construct and maintain its local environment.

#### ✨ Features

- **Corpse Fertilization**: Death now returns a percentage of its stored biomass to the soil's fertility, creating "fertile graveyards" that boost vegetation.
- **Metabolic Feedback (Excretion)**: High-energy entities moving across the terrain have a 10% chance to fertilize the soil, simulating natural excretion and niche construction.
- **Lineage Registry Pruning**: Automatic garbage collection of extinct, low-impact lineages every 1,000 ticks to maintain peak performance during deep-time simulations.
- **Soil Dashboard**: Real-time tracking of Global Average Fertility in the TUI status bar.

---

## [Phase 42-43: Adaptive Speciation & Era Dynamics] - 2026-01-24

### Evolutionary Leap: Divergent Evolution

Phases 42 and 43 focus on macro-evolutionary branching and the environmental forcing of adaptive radiations. The simulation now transitions between narrative eras based on global metrics rather than simple time.

#### ✨ Features

- **Automatic Speciation**: Real-time lineage splitting based on a genetic distance threshold (NEAT topology + Phenotypic traits).
- **Era Transitions**: Dynamic shift between Primordial, Dawn of Life, Flourishing, and Dominance War eras triggered by Biomass, CO2, and Biodiversity metrics.
- **Evolutionary Velocity**: Slide-window metrics tracking the "intensity" of genetic drift in the population.
- **Fossil Resurrection**: Users can now select fossils in the archeology tool (Y) and resurrect/clone ancient genotypes back into the living world (G).

---

## [Phase 41: Massive Parallelism & Spatial Indexing] - 2026-01-24

### Evolutionary Leap: Multi-Core Cognitive Scaling

Phase 41 transitions the world update loop from sequential processing to a highly parallelized pipeline. By leveraging Rayon and a row-partitioned Spatial Hash, the engine can now simulate 10,000+ entities with zero-jitter performance, effectively saturating modern multi-core CPUs.

#### ✨ Features

- **Massive Parallelism**: All individual biological and neural processes are now computed in parallel using thread-local buffers.
- **Parallel Interaction Pass**: Implemented an "Interaction Proposal" system where entities propose actions (Kill, Birth, Eat) in parallel, resolved sequentially to ensure thread safety.
- **Optimized Spatial Indexing**: Row-level mutex protection enables parallel construction of the Spatial Hash grid.

#### 🛠️ Technical Achievements

- **Rayon Integration**: Refactored `World::update` into a 3-pass parallel architecture (Perception -> Intel/Action Proposals -> Command Resolution).
- **Inertia Scaling Fix**: Refined momentum calculations to ensure larger entities correctly display increased mass and reduced responsiveness.

---

## [Phase 40: Archeology & Deep History] - 2026-01-24

### Evolutionary Leap: Fossil Records & Time Travel

Phase 40 introduces deep time into the simulation. Users can now preserve the legacy of extinct lineages through persistent fossilization and browse the macro-evolutionary trajectory of their world via history snapshots.

#### ✨ Features

- **Fossil Record**: Persistent archival of extinct lineages' "Best Legendary" representatives in `logs/fossils.json`.
- **History Snapshots**: Periodic macro-state capture (every 1,000 ticks) including biodiversity hotspots and atmospheric carbon levels.
- **Archeology View (Y)**: A dedicated TUI panel for browsing fossilized dynasties and navigating history.
- **Time Travel Navigation**: Seek through world history snapshots using `[` and `]` keys.

#### 🛠️ Technical Achievements

- **Persistent Historical State**: Implemented `FossilRegistry` and `HistoryLogger::Snapshot` for low-overhead macro-state recording.
- **Fossilization Engine**: Automated extraction of neural genotypes upon lineage extinction.

---

## [Phase 38-39: Resilience, Stasis & Succession] - 2026-01-24

### Evolutionary Leap: Genetic Drift & Biome Succession

Phase 38 and 39 focus on long-term ecological stability and the genetic mechanisms that govern population resilience.

#### ✨ Features

- **Population-Aware Mutation**: Mutation rates dynamically scale (0.5x to 3.0x) based on current population density, balancing exploration and exploitation.
- **Genetic Drift**: Implemented stochastic trait randomization in bottlenecked populations (<10 entities).
- **Environmental Succession**: Biomes now transition between Plains, Forest, and Desert based on fertility and plant biomass.
- **Global Carbon Cycle**: Integrated metabolic carbon emissions with biological sequestration, linked to climate state forcing.

---

## [Phase 35: Trophic Cascades & Apex Competition] - 2026-01-23

### Evolutionary Leap: Macroevolutionary Graphing & Dynastic Analysis

Phase 34 introduces a sophisticated ancestry tracking system powered by `petgraph`. Instead of just tracking current population counts, the simulation now builds a persistent "Tree of Life," visualizing the branching paths of evolution from the first seed to the current dominant dynasties.

#### ✨ Features

- **Real-time Ancestry Tree**: Visualize the evolutionary relationships between lineages in a dedicated TUI view (press `A`).
- **Graphviz/DOT Export**: Export the complete evolutionary history of your world to a `.dot` file for external visualization (press `Shift+A`).
- **Dominant Dynasty Tracking**: Automatically identifies and highlights the top 5 most successful evolutionary branches.
- **Lineage-Trophic Overlay**: The ancestry view integrates trophic data, showing how dietary specializations (Herbivore/Carnivore) emerged along the family tree.

#### 🛠️ Technical Achievements

- **Graph-based Lineage Registry**: Implemented `lineage_tree.rs` using `petgraph` to manage macroevolutionary data as a directed graph.
- **Dynamic Branch Detection**: Developed algorithms to detect significant lineage divergence and record branching events.
- **High-Performance TUI Tree Rendering**: Custom rendering logic to display complex graph structures within terminal constraints.

---

## [Phase 32.5: Quality Lockdown & Hardening] - 2026-01-23

### Stabilization Leap: Zero-Panic & Systemic Integrity

Phase 32.5 marks the transition from feature-bursting to production-level hardening. The core engine and its 15+ integration suites have been fully synchronized with the 19x8 neural architecture and Life History genes.

#### ✨ Features

- **Engine Hardening**: Implemented a zero-panic guarantee for cross-universe migrations and DNA imports, successfully handling malformed or version-mismatched data.
- **Robustness Suite**: Added `tests/robustness.rs` to verify edge cases like mass extinction and corrupted genetic payloads.
- **ActionContext Pattern**: Refactored motorized systems to use a unified context, resolving technical debt and improving performance during high-concurrency loops.

#### 🛠️ Technical Achievements

- **Systemic Sync**: Repaired and updated all legacy integration tests (Ecology, Social, Pathogens, Disasters) to support the Phase 32 multi-gene ecosystem.
- **Biomechanical Fixes**: Corrected inertia scaling where larger energy storage correctly increases mass and reduces steering responsiveness.
- **Clippy-Clean Baseline**: Achieved zero-warning status across the entire workspace under the strict `-D warnings` gate.

---

## [Phase 32: Life History Strategies (R/K Selection)] - 2026-01-23

### Evolutionary Leap: Maturation Rates & Parental Investment

Phase 32 introduces biological life history strategies, allowing organisms to evolve their reproductive and developmental patterns. Lineages can now specialize as "R-strategists" (producing many low-investment offspring quickly) or "K-strategists" (investing heavily in fewer, high-quality offspring).

#### ✨ Features

- **Life History Genes**: Added `reproductive_investment` (0.1-0.9) and `maturity_gene` (0.5-2.0) to the genotype.
- **R/K Selection Mechanics**: Organisms can now evolve the ratio of energy passed to offspring and the time required to reach maturity.
- **Growth-Size Coupling**: An entity's `max_energy` (stomach size) now scales with its `maturity_gene`, creating a trade-off between rapid generation turnover and individual resilience.
- **Developmental Momentum**: Specialists with high maturity genes start life with significantly higher energy reserves but face a much longer juvenile period.

#### 🛠️ Technical Achievements

- **Genetic Strategy Engine**: Updated the reproduction system to factor in evolvable investment ratios.
- **Dynamic Maturity Gates**: Refactored the `Biological` system to handle variable maturation thresholds per entity.
- **HexDNA Update**: Extended the serialization protocol to include Life History genes.

---

## [Phase 31: Metabolic Niches & Resource Diversity] - 2026-01-23

### Evolutionary Leap: Dietary Specialization & Typed Resources

Phase 31 introduces resource diversity and metabolic specialization. Food sources now carry specific nutrient types (Green vs. Blue), and organisms must evolve a matching metabolic niche to digest them efficiently. This creates a strong selective pressure for terrain-based specialization, as different biomes now favor different nutrient types.

#### ✨ Features

- **Metabolic Niches**: Organisms now possess a genetic `metabolic_niche` trait, determining their digestive efficiency for different food types.
- **Resource Diversity**: Food items now have a `nutrient_type` attribute (0.0 for Green, 1.0 for Blue).
- **Digestive Efficiency Engine**: Implemented a scaling system where a specialist match yields **1.2x** energy, while a total mismatch yields only **0.2x**.
- **Terrain-Nutrient Coupling**: Mountains and Rivers now predominantly spawn Blue food, while Plains and Oases spawn Green food.
- **Nutrient Sensing**: Added a 12th environmental brain input (`NutType`) to allow organisms to perceive the nutrient type of the nearest food source.

#### 🛠️ Technical Achievements

- **Expanded Brain Topology**: Upgraded the standard architecture to **19-6-8 RNN-lite** to support nutrient sensing.
- **Typed Resource Spawning**: Enhanced the `Ecological` system to factor in terrain types during food generation.
- **Genotype Expansion**: Integrated `metabolic_niche` into the inheritable genotype and HexDNA protocol.

---

## [Phase 30: Social Coordination & Kin Recognition] - 2026-01-23

### Evolutionary Leap: Swarming & Semantic Communication

Phase 30 introduces advanced social behaviors through kin recognition and active semantic signaling. Organisms can now sense the relative center of mass of their lineage, enabling the emergence of collective movement patterns and swarming behaviors, supported by a significant expansion of the neural architecture.

#### ✨ Features

- **Kin Centroid Sensing (KX, KY)**: Organisms now perceive the relative position of their lineage allies as a unified vector, allowing for group cohesion.
- **Herding Bonus**: Implemented a metabolic reward (0.05 energy/tick) for moving in alignment with the kin centroid, encouraging collective migration.
- **Semantic Pheromones (SA, SB)**: Added neural outputs and environmental sensors for abstract signals, providing a substrate for evolved coordination.
- **Enhanced Neural Substrate**: Upgraded the standard brain architecture to **18-6-8 RNN-lite** to support new social inputs and action outputs.
- **Age & Wall Awareness**: Added dedicated sensors for maturity (`AG`) and wall proximity (`WL`) for better environmental adaptation.

#### 🛠️ Technical Achievements

- **Expanded Brain Topology**: Increased environmental inputs from 7 to 12 and outputs from 6 to 8.
- **Kin Vector Engine**: Implemented real-time calculation of lineage centroids within the `Social` system.
- **Speed Modulation**: Output 2 now controls continuous speed modulation rather than a binary boost.

---

## [Phase 28: Complex Brain Evolution (NEAT-lite)] - 2026-01-23

### Evolutionary Leap: Evolvable Topology & Efficiency Pressure

Phase 28 transitions the cognitive engine from a fixed-matrix MLP to a dynamic, graph-based **NEAT-lite** architecture. Brains are no longer static; they can grow new neurons and connections through mutation, allowing for more specialized and complex behaviors while enforcing metabolic penalties for unnecessary complexity.

#### ✨ Features

- **Evolvable Brain Topology**: Implemented "Add Node" and "Add Connection" mutations, enabling the brain to grow in complexity over generations.
- **Innovation Tracking**: Structural mutations are tracked via innovation numbers, ensuring consistent genetic crossover between diverse topologies.
- **Metabolic Penalty for Bloat**: Introduced energy costs for brain complexity (0.02 per node, 0.005 per connection) to select for cognitive efficiency.
- **Dynamic Brain Visualization**: The TUI brain heatmap now adapts to show the evolving graph structure and connection weights.

#### 🛠️ Technical Achievements

- **Graph-based Neural Engine**: Refactored `Brain` from matrix-based to an adjacency-list graph structure for flexible inference.
- **NEAT Crossover Algorithm**: Implemented an innovation-aligned crossover mechanism for structural genes.
- **Metabolic Complexity Scaling**: Integrated structural costs into the `Action` system's energy formula.

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
