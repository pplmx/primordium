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
Entities are represented by symbols indicating their current state:
- `@` **Idle/Alive**: Healthy entity roaming the world.
- `F` **Fighting**: Currently engaged in combat.
- `E` **Eating**: Consuming food or prey.
- `M` **Mating**: Reproducing with another entity.
- `♣` **Sharing**: Cooperating/Sharing energy with tribe members.
- `†` **Dead**: A corpse that will decompose into organic matter.

### Colors (Tribes)
Entities are colored based on their **genetic tribe**.
- Entities with similar colors (RGB distance < 60) belong to the same **Tribe**.
- Tribe members do **not** attack each other.
- Tribe members may share food if altruism gene is active.

### Terrain
- `.` **Plains**: Standard movement speed.
- `~` **River** (Blue): Faster movement (currents), higher energy cost.
- `▲` **Mountain** (Gray): Slow movement, defense bonus.
- `*` **Food** (Green): Plant matter to be eaten.

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
