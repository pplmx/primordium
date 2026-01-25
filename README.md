# Primordium (原初之境)

[简体中文](./docs/README_zh.md) | [Changelog](./CHANGELOG.md)

> *Evolution in silicon, powered by your machine*

**Primordium** is a hardware-coupled artificial life simulation that lives in your terminal. It bridges the gap between your physical computer and a digital ecosystem, where the laws of nature are shaped by your machine's real-time performance.

![Status](https://img.shields.io/badge/Status-Stable-brightgreen)
![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-blue)

---

## 🎯 Vision

Primordium is an experiment in **emergent complexity**. It transforms your host machine into a digital god:

- **CPU Workload** becomes the environmental climate.
- **RAM Pressure** controls resource scarcity.
- **Neural Networks** evolve as organisms adapt to your hardware's unique signature.

---

## 🌊 Core Mechanics: Hardware Resonance

The simulation environment is directly coupled to your computer's real-time metrics.

### 🌡️ Climate (CPU-Coupled)

Your CPU usage dictates metabolic speed. High machine load forces life to burn energy faster.

| CPU Usage | Climate State | Metabolism | Effect |
| ----------- | ---------------- | ------------- | --------- |
| 0-30%     | 🌡️ Temperate  | ×1.0       | Baseline survival |
| 30-60%    | 🔥 Warm       | ×1.5       | Increased energy burn |
| 60-80%    | 🌋 Hot        | ×2.0       | High metabolic stress |
| 80-100%   | ☀️ Scorching  | ×3.0       | Rapid starvation risk |

### 🌾 Resource Scarcity (RAM-Coupled)

Memory usage determines food availability. High RAM usage simulates a resource-famine environment.

---

- [User Manual (English)](./docs/MANUAL.md)
- [用户手册 (中文)](./docs/MANUAL_zh.md)
- [Web Guide](./www/README.md)

## ✨ Features

### 🧠 Neural Awakening & Selection

Each entity possesses a **dynamic Graph-based (NEAT-lite)** brain. Through natural selection, organisms learn to hunt, coordinate, and manage energy.

- **Evolvable Topology**: Brains can grow new neurons and connections to adapt to complex environments.
- **Efficiency Pressure**: Complexity carries a metabolic cost (0.02/node + 0.005/conn), preventing network bloat.
- **Kin Recognition**: Entities sense the relative centroid of their lineage members (**KX**, **KY**), enabling collective herding behaviors.
- **Semantic Language**: Active chemical signaling (**SA**, **SB**) provides a substrate for evolved social coordination.
- **Linguistic Evolution (Phase 48)**: Entities possess **Hearing** (Input) and **Vocalization** (Output) channels. Real-time acoustic ripples propagate through the `SoundGrid`, allowing for the emergence of alarm calls and swarming behaviors.
- **Lifetime Learning (Phase 47)**: Hebbian plasticity allows brains to adapt weights in real-time based on reinforcement signals (Food/Pain).
- **Massive Parallelism (Phase 41)**: Powered by **Rayon** and a "Proposal Unzipping" pipeline, supporting 10,000+ entities with zero-jitter performance on multi-core systems.

### 👥 Social & Life History

- **Specialized Castes (Phase 53)**: Entities evolve into **Soldiers** (high damage, high cost), **Engineers** (efficient terraforming), or **Providers** (efficient sharing) based on their lifetime activities and genetic bias.
- **R/K Selection Strategies**: Organisms evolve trade-offs between many weak offspring (Strategy R) or few high-investment heirs (Strategy K).
- **Metabolic Niches**: Specialized digestion for Green vs Blue food types coupled to terrain geography.
- **Interspecies Symbiosis (Phase 54)**: Mutualistic bonds can form between different lineages, granting metabolic bonuses and enabling **Interspecies Hybridization** (sexual crossover).
- **Group Defense**: Proximity to same-lineage members reduces incoming predation damage.
- **Persistent Lineages**: Success is tracked globally in the **Lineage Registry**.

### 🦠 Pathogens & Parasites

- **Dynamic Emergence**: Pathogens naturally emerge and mutate, affecting host health and energy.
- **Parasitic Manipulation (Phase 55)**: Advanced pathogens can hijack host neural circuits, forcing specific behaviors (e.g., forced aggression or vocalization) to facilitate their own spread.
- **Immunity Evolution**: Entities gain resistance after recovery and pass it to offspring with minor mutations.

### ⚡ Divine Interface v2

- **Terrain Editor**: Use **Mouse Drag** to paint Mountains, Rivers, and Walls directly onto the world.
- **Targeted Intervention**: Manually **Mutate (M)**, **Smite (K)**, or **Reincarnate (P)** selected organisms.
- **Archeology & Fossils (Phase 40)**: Persistent **Fossil Record** (`logs/fossils.json.gz`) preserves extinct legendary genotypes with high storage efficiency. Periodic **History Snapshots** enable time-travel browsing of macro-evolutionary trends.
- **God Mode Overrides**: Induce global Heat Waves, Resource Booms, or Mass Extinctions via keyboard macros.

### 🦁 Apex Predators & Sexual Reproduction

- **Predatory Dynamics**: Organisms can evolve aggression to hunt others. **Soldier castes** and **War Zones** apply lethal damage multipliers.
- **Genetic Crossover**: Sexual reproduction enables neural trait exchange. Bonded partners can reproduce inter-lineage hybrids.
- **HexDNA Protocol**: Export (`C`) and import (`V`) organism genomes as portable text files.

### 📊 The Omniscient Eye

- **Tree of Life (Phase 34)**: Real-time ancestry visualization using `petgraph`. Trace the branching history of the top 5 dominant dynasties.
- **Era System**: Population-driven narrative engine tracks world epochs (Genesis, Expansion, Decline, etc.).
- **Hall of Fame**: Real-time leaderboard of the top 3 fittest organisms.
- **Advanced Analytics**: Brain entropy, average lifespan, and dual-sparkline population dynamics.
- **Specialization Visualizer**: Real-time TUI brain view mapping 11 outputs and caste progression meters.

### 🏔️ Ecosystem Dynamics

- **Terrain System**: Mountains (▲ slow), Rivers (≈ fast, hydrating), Oases (◊ food-rich).
- **Emergent Architecture (Phase 52)**: Entities can **Dig** canals or **Build** protective **Nests** (Ω). Nests provide metabolic recovery and nursery bonuses for offspring.
- **Hydrological Coupling**: Artificial canals boost nearby soil fertility, enabling biological terraforming.
- **Season Cycle**: Spring, Summer, Fall, Winter affecting metabolism and food availability.
- **Geographic Pressure**: Migration patterns emerge from terrain-based resource distribution.

### 👥 Social Structures

- **Pheromone System**: Food trails attract foragers, danger pheromones warn of predators.
- **Tribe Formation**: Color-similar entities form protective tribes (no intra-tribe attacks).
- **Territorial Behavior**: Entities are more aggressive near their birth location.
- **Energy Sharing**: High-energy entities can share with starving neighbors (♣).
- **Advanced Hierarchy (Phase 49)**: Alphas influence movement; overcrowded tribes fracture.

### 🌌 Global Hive & Networking

- **P2P Multiverse**: Entities migrate between simulation instances via a high-performance **Axum** relay server.
- **Peer Discovery**: Automated peer awareness with real-time REST APIs for global monitoring.
- **HexDNA Protocol**: Unified genetic protocol ensuring 100% fidelity migrations across simulation versions.

### 📜 The Ledger & Blockchain

- **History Logging**: Continuous streaming of life events to JSONL.
- **Immutable Proof**: Datasets are anchored to the **Bitcoin blockchain** via OpenTimestamps for cryptographic proof of evolution.

### ⚡ Performance & Stability

- **Component-Based Entity (CBE)**: Logical grouping of attributes into Physics, Metabolism, Health, and Intel.
- **Systemic Decomposition**: Monolithic update logic split into specialized systems (Perception, Action, Biological, Social).
- **Multi-threaded Pipeline**: Fully parallel updates supporting 10,000+ entities with zero-jitter performance.
- **Strict Quality Gate**: 100% Clippy compliance and 40+ tests ensuring digital stability.
- **Spatial Hashing**: Optimized lock-free sensory lookups for high-density populations.


---

## 🚀 Quick Start

```bash
# Clone and enter
git clone https://github.com/pplmx/primordium.git
cd primordium

# Run Standard Mode
cargo run --release

# Run Screensaver Mode
cargo run --release -- --mode screensaver
```

---

## ⌨️ Controls

| Key | Action |
| ----- | --------- |
| `Q` | Quit simulation |
| `Space` | Pause / Resume |
| `A` | Toggle **Ancestry View** (Tree of Life) |
| `Shift+A` | Export Ancestry Tree (DOT format) |
| `Y` | Toggle **Archeology & Fossil Record** |
| `[` / `]` | **Time Travel** (Navigate History Snapshots) |
| `B` | Toggle Neural Brain Heatmap |
| `H` | Toggle Help Overlay |
| `X` | Trigger **Genetic Surge** (Global Mutation) |
| `M` | Mutate selected organism |
| `K` | Smite (Remove) selected organism |
| `P` | Reincarnate (Randomize DNA) selected organism |
| `! @ # $ % ^` | Select Terrain Brush (Plains, Mt, River, Oasis, Wall, Barren) |
| `Shift+K` | Toggle **Heat Wave** Disaster |
| `L` | Trigger **Mass Extinction** (90% wipe) |
| `R` | Trigger **Resource Boom** (100x Food) |
| `+` / `-`| Increase / Decrease time scale |
| `Left Click` | Select organism / **Hold & Drag to Paint Terrain** |
| `Right Click`| Inject Food Cluster |

---

## 🌱 Philosophy

Every run of Primordium is unique. Your specific hardware workload creates a one-of-a-kind evolutionary pressure. Every lineage is precious, and every extinction is a lesson in the primordial soup.
