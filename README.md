# Primordium (原初之境)

> *Evolution in silicon, powered by your machine*

**Primordium** is a hardware-coupled artificial life simulation that lives in your terminal. It bridges the gap between your physical computer and a digital ecosystem, where the laws of nature are shaped by your machine's real-time performance.

![Primordium Concept](https://img.shields.io/badge/Status-Phase%203%20Implemented-green)
![Rust](https://img.shields.io/badge/Built%20with-Rust-orange)

---

## 🎯 Vision

Primordium is an experiment in **emergent complexity**. It transforms your host machine into a digital god:

- **CPU temperature** becomes the environmental climate.
- **RAM pressure** controls resource scarcity.
- **Evolution** writes the story as organisms adapt to your hardware's workload.

---

## 🌊 How it Works: Hardware Resonance

The simulation environment is directly coupled to your computer's real-time metrics using the `sysinfo` crate.

### 🌡️ Climate (CPU-Coupled)

Your CPU usage dictates the "Climate State," which directly affects the metabolism speed of all digital organisms.

| CPU Usage | Climate State | Metabolism Multiplier | Effect |
| ----------- | -------------|--------------- | ----------------------- | -------- |
| 0-30%     | 🌡️ Temperate  | ×1.0                  | Baseline energy consumption |
| 30-60%    | 🔥 Warm       | ×1.5                  | Increased energy burn |
| 60-80%    | 🌋 Hot        | ×2.0                  | High metabolic stress |
| 80-100%   | ☀️ Scorching  | ×3.0                  | Scorching heat; mass die-offs |

### 🌾 Resource Scarcity (RAM-Coupled)

Memory usage determines the availability of food in the world.

| RAM Usage | Resource State | Food Spawn Rate |
| ----------- | ------------- | ---------------- | ----------------- |
| 0-50%     | 🌾 Abundant    | ×1.0            |
| 50-70%    | ⚠️ Strained    | ×0.7            |
| 70-85%    | 🚨 Scarce      | ×0.4            |
| 85-100%   | 💀 Famine      | ×0.1            |

---

## ✨ Key Features

### 1. Genesis (Physics Foundation)

A terminal-based 2D universe where organisms move with momentum.

- **Random Walk with Momentum**: Organisms don't just move randomly; they have velocity and inertia.
- **Boundary Interaction**: Entities bounce off the edges of their terminal universe.
- **60 FPS Rendering**: Smooth, high-performance TUI powered by `ratatui`.

### 2. Metabolism & Evolution

Life is a constant struggle for energy.

- **Energy System**: Movement and existence cost energy. Running out of energy means death.
- **Feeding**: Consume food particles (`*`) to restore energy.
- **Reproduction**: Once an organism accumulates enough energy, it reproduces asexually.
- **Heredity & Mutation**: Offspring inherit their parent's traits (color, velocity) with slight mutations, allowing for natural selection.

### 3. Real-time Monitoring

- **Hardware Gauges**: Visual representations of current CPU and RAM usage.
- **CPU Sparkline**: A 60-second historical graph of your machine's load.
- **Status Dashboard**: Real-time stats on population, generations, and environmental multipliers.

---

## 🚀 Quick Start

Ensure you have [Rust and Cargo](https://rustup.rs/) installed.

```bash
# Clone the repository
git clone https://github.com/pplmx/primordium.git
cd primordium

# Build and run
cargo run --release
```

---

## ⌨️ Controls

| Key | Action |
| --- | ------ |
| `Q` | Quit simulation |
| `Space` | Pause / Resume |

---

## 🗺️ Roadmap

- [x] **Phase 1-3**: Core physics, metabolism, and hardware coupling.
- [ ] **Phase 4: Neural Awakening**: Replacing random movement with Neural Networks. Organisms will develop "senses" to find food and avoid crowds.
- [ ] **Phase 5: Historical Archives**: Detailed tracking of lineages and the preservation of "Legendary" organisms.
- [ ] **Phase 5.5: Blockchain Anchoring**: Cryptographically proving evolutionary history and minting legendary organisms as NFTs.

---

## 🌱 Philosophy

Every run of Primordium is unique. Your machine's specific hardware signature and your daily workflow create a one-of-a-kind evolutionary pressure. Every lineage is precious, and every extinction is a lesson in the primordial soup.

*Welcome to the original frontier.*
