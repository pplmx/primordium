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

Each entity possesses a **12-6-5 Recurrent Neural Network** brain integrated into a modular **Intel** component. Through natural selection, organisms learn to hunt, navigate, and manage energy.

- **Recurrent Memory**: 6 inputs are dedicated to the previous tick's internal state, enabling time-coherent behavior.
- **Parallel Inference**: Neural processing is multi-threaded using **Rayon**, enabling massive population scaling.
- **Interactive Selection**: Use the **Mouse** to click and inspect specific organisms.
- **Genotype Clustering**: Organisms are automatically classified into species based on neural weight similarity.

### 🦁 Apex Predators & Sexual Reproduction

- **Predatory Dynamics**: Organisms can evolve aggression to hunt and consume others for massive energy gains.
- **Genetic Crossover**: Sexual reproduction enables neural trait exchange with nearby mates.
- **HexDNA Protocol**: Export (`C`) and import (`V`) organism genomes as portable text files.

### 📊 The Omniscient Eye

- **Era System**: Population-driven narrative engine tracks world epochs (Genesis, Expansion, Decline, etc.).
- **Hall of Fame**: Real-time leaderboard of the top 3 fittest organisms.
- **Advanced Analytics**: Brain entropy, average lifespan, and dual-sparkline population dynamics.

### 🏔️ Ecosystem Dynamics

- **Terrain System**: Mountains (▲ slow), Rivers (≈ fast), Oases (◊ food-rich)
- **Season Cycle**: Spring, Summer, Fall, Winter affecting metabolism and food availability
- **Geographic Pressure**: Migration patterns emerge from terrain-based resource distribution

### 👥 Social Structures

- **Pheromone System**: Food trails attract foragers, danger pheromones warn of predators
- **Tribe Formation**: Color-similar entities form protective tribes (no intra-tribe attacks)
- **Territorial Behavior**: Entities are more aggressive near their birth location
- **Energy Sharing**: High-energy entities can share with starving neighbors (♣)

### 🌌 Global Hive & Networking (Phase 22)

- **Distributed Evolution**: Entities migrate between simulation instances via a high-performance **Axum** relay server.
- **Inter-Universe Migration**: Move off-screen to send your most successful lineages to other users.
- **Real-time Synchronization**: A shared digital multiverse where life evolves across boundaries.

### 📜 The Ledger & Blockchain

- **History Logging**: Continuous streaming of life events to JSONL.
- **Immutable Proof**: Datasets are anchored to the **Bitcoin blockchain** via OpenTimestamps for cryptographic proof of evolution.

### ⚡ Performance & Stability

- **Component-Based Entity (CBE)**: Logical grouping of attributes into Physics, Metabolism, Health, and Intel for better data locality and isolation.
- **Systemic Decomposition**: Monolithic update logic split into specialized systems (Perception, Action, Biological, Social).
- **Parallel Processing**: Multi-core acceleration via **Rayon** for perception lookups and neural decisions.
- **Strict Quality Gate**: 100% Clippy compliance and 40+ tests ensuring digital stability.
- **Spatial Hashing**: Optimized $O(N \log N)$ sensory lookups for high-density populations.

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
| `B` | Toggle Neural Brain Heatmap |
| `H` | Toggle Help Overlay |
| `X` | Trigger **Genetic Surge** (Global Mutation) |
| `C` | Export selected organism's HexDNA |
| `V` | Infuse organism from HexDNA file |
| `+` / `-`| Increase / Decrease time scale |
| `Left Click` | Select Organism |
| `Right Click`| Inject Food Cluster |

---

## 🌱 Philosophy

Every run of Primordium is unique. Your specific hardware workload creates a one-of-a-kind evolutionary pressure. Every lineage is precious, and every extinction is a lesson in the primordial soup.
