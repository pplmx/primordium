# Primordium (原初之境)

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

The simulation environment is directly coupled to your computer's real-time metrics using the `sysinfo` crate.

### 🌡️ Climate (CPU-Coupled)

Your CPU usage dictates the metabolic speed of all digital organisms. High machine load forces life to burn energy faster.

| CPU Usage | Climate State | Metabolism | Effect |
| --------- | ------------- | ---------- | ------ |
| 0-30%     | 🌡️ Temperate  | ×1.0       | Baseline survival |
| 30-60%    | 🔥 Warm       | ×1.5       | Increased energy burn |
| 60-80%    | 🌋 Hot        | ×2.0       | High metabolic stress |
| 80-100%   | ☀️ Scorching  | ×3.0       | Rapid starvation risk |

### 🌾 Resource Scarcity (RAM-Coupled)

Memory usage determines the availability of food. High RAM usage simulates a resource-famine environment.

| RAM Usage | Resource State | Food Spawn Rate |
| --------- | ------------- | -------------- | --------------- |
| 0-50%     | 🌾 Abundant    | ×1.0            |
| 50-70%    | ⚠️ Strained    | ×0.7            |
| 70-85%    | 🚨 Scarce      | ×0.4            |
| 85-100%   | 💀 Famine      | ×0.1            |

---

## ✨ Features

### 🧠 Neural Awakening

Organisms are no longer random. Each entity possesses a **4-layer MLP Neural Network** (42 genes) that processes sensory data:

- **Inputs**: Nearest food vector, energy reserves, and local crowding.
- **Outputs**: Movement steering and metabolic speed boosting.
- **Mutation**: Hereditary DNA mutation and rare genetic drift enable natural selection.

### 📜 The Ledger & Blockchain

- **History Logging**: Every birth, death, and climate shift is recorded in JSONL format.
- **Legends Archive**: High-fitness organisms (survivors and prolific parents) are archived forever.
- **Immutable Proof**: Evolution datasets are hashed (SHA-256) and anchored to the **Bitcoin blockchain** via OpenTimestamps.

### ⚡ Performance & Modes

- **Spatial Hashing**: Optimized $O(N \log N)$ sensory lookups allow for 500+ entities at 60 FPS.
- **Flexible Modes**:
  - **Standard**: Interactive TUI with full stats and brain visualizer.
  - **Screensaver**: Minimalist, high-efficiency world view.
  - **Headless**: Pure simulation for background research and logging.

---

## 🚀 Quick Start

Ensure you have [Rust](https://rustup.rs/) installed.

```bash
# Clone and enter
git clone https://github.com/pplmx/primordium.git
cd primordium

# Run Standard Mode
cargo run --release

# Run Screensaver Mode
cargo run --release -- --mode screensaver

# Run Headless Mode (CLI only)
cargo run --release -- --mode headless
```

---

## 🛠️ Analysis & Verification

### Generate Evolution Report

Rebuild family trees and analyze population dynamics from your session logs.

```bash
cargo run --release --bin analyze -- --live-log logs/live.jsonl --output my_report.md
```

### Validate Blockchain Proof

Verify that your evolutionary history hasn't been tampered with since anchoring.

```bash
cargo run --release --bin verify
```

---

## ⌨️ Controls

| Key | Action |
| --- | ------ |
| `Q` | Quit simulation |
| `Space` | Pause / Resume |
| `B` | Toggle Neural Brain Heatmap |
| `H` | Toggle Help Overlay |
| `+` / `-` | Increase / Decrease time scale |

---

## 🌱 Philosophy

Every run of Primordium is unique. Your specific hardware workload creates a one-of-a-kind evolutionary pressure. Every lineage is precious, and every extinction is a lesson in the primordial soup.

*Last updated: 2026-01-20*
