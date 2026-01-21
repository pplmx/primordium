# Primordium User Manual

Welcome to **Primordium**, an advanced Artificial Life simulation where entities evolve, form societies, and struggle for survival in a dynamic ecosystem. This manual guides you through the simulation mechanics and controls.

---

## 🚀 Getting Started

### Modes
Primordium runs in two environments:
1. **Terminal (TUI)**: The classic, high-performance experience.
   ```bash
   primordium
   ```
2. **Web Browser (WASM)**: A modern graphical interface via WebAssembly.
   (See [Web Guide](../www/README.md) for setup)

---

## 🎮 Controls

### Global Keys
| Key | Action |
|-----|--------|
| `q` | **Quit** the simulation |
| `p` | **Pause/Resume** simulation |
| `r` | **Reset** world state (kills all entities) |
| `s` | Toggle **Statistics Overlay** (TUI) |
| `h` | Toggle **Help/Controls** |
| `w` | **Save** Simulation State |
| `l` | **Load** Simulation State |

### TUI Specific
| Key | Action |
|-----|--------|
| `Click` | Select an entity to view its brain/genome |
| `Arrows` | Move inspection cursor |

---

## 👁️ Interface Guide

### Entities & Status
Entities are represented by symbols indicating their current physiological state:
- `●` **Foraging**: Standard state, searching for resources.
- `♦` **Hunting**: Aggressive state, attempting to consume other entities.
- `♥` **Mating**: High-energy state, ready to reproduce.
- `†` **Starving**: Critical energy state (< 20%), high risk of death.
- `◦` **Juvenile**: Immature state, unable to reproduce. [NEW]
- `☣` **Infected**: Carrying a pathogen, loses energy and spreads disease. [NEW]
- `♣` **Sharing**: Altruistic state, giving energy to nearby tribe members.

### Colors (Tribes)
Entities are colored based on their **genetic tribe**.
- Entities with similar colors (RGB distance < 60) belong to the same **Tribe**.
- Tribe members do **not** attack each other.
- Tribe members may share energy if their neural "Share" output is high.

### Terrain
- ` ` **Plains**: Standard movement speed.
- `≈` **River** (Blue): Faster movement (1.5x), represents water currents.
- `▲` **Mountain** (Gray): Slow movement (0.5x), no food growth.
- `◊` **Oasis** (Green): Prime real estate with 3x food spawn rate.
- `*` **Food** (Green): Energy source spawned based on RAM usage.

---

## 🧬 Evolution Mechanics

### The Brain
Each entity possesses a neural network (Brain) that evolves over generations.
- **Inputs (Sensors)**:
  - Vision (Food/Entities nearby)
  - Internal Energy Level
  - Pheromone Strength (Smell)
  - Tribe Density
- **Outputs (Actions)**:
  - Move X / Y
  - Boost (Sprint)
  - Attack / Aggressiveness
  - Share (Altruism)

### Genetics
When entities reproduce (Mating `M`), their offspring inherits a mix of parents' DNA with slight mutations.
- **Attributes**: Speed, Strength, Metabolism, Color.
- **Brain**: Weights and biases are mutated.

### Pheromones
Entities leave chemical trails:
- **Food Trail**: "I found food here" (Attracts others).
- **Danger Trail**: "I died here" (Warns others).

---

## 🌍 Ecosystem

### Weather
Seasons change cyclically, affecting food growth rates and metabolism.
- **Spring/Summer**: Abundant food.
- **Winter**: Scarcity, higher metabolism cost.

### Disaster (The Divine Interface)
Occasional random events or user-triggered catastrophes (if Divine Mode enabled) can reshape the population.

---

## ⚔️ Game Modes

Launch Primordium with different rulesets using `--gamemode`:

### Standard
`--gamemode standard`
The classic Darwinian struggle. Entities fight, cooperate, and evolve naturally.

### Cooperative
`--gamemode coop`
**Global Peace**. Attack behaviors are disabled. The goal is to maximize population and efficiency.
- Ideal for studying swarm dynamics and altruism.

### Battle Royale
`--gamemode battle`
**Survival of the Fittest Tribe**.
- The "Safe Zone" shrinks over time.
- Entities outside the safe zone take massive damage.
- Forces conflict in the center.

---

## 🌌 Multiplayer (New!)
Primordium supports **Interstellar Migration**.
- If an entity travels off the edge of the screen while "Online", it migrates to the server.
- The server routes the entity to another connected user's simulation.
- Look for incoming migrants appearing at your world borders!

---

## 📚 Technical Wiki
For advanced users and modders:
- [Genetics & HexDNA](../docs/wiki/GENETICS.md)
- [Neural Network Architecture](../docs/wiki/BRAIN.md)
- [Ecosystem Formulas](../docs/wiki/ECOSYSTEM.md)

---
*Generated: 2026-01-21*
