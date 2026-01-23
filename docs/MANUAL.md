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
| ----- | --------- |
| `q` | **Quit** the simulation |
| `Space` | **Pause/Resume** simulation |
| `r` | **Reset** world state (kills all entities) |
| `s` | Toggle **Statistics Overlay** (TUI) |
| `h` | Toggle **Help/Controls** |
| `w` | **Save** Simulation State |
| `l` | **Load** Simulation State |

### TUI Specific

| Key | Action |
| ----- | --------- |
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
- `♣` **Sharing**: Altruistic state, giving energy to nearby tribe members.
- `☣` **Infected**: Carrying a pathogen, loses energy and spreads disease.
- `◦` **Juvenile**: Immature state, unable to reproduce.

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
- `░` **Barren** (Brown): Overgrazed or disaster-struck land. Very low food growth and 0.7x movement speed.
- `█` **Wall** (Dark Gray): Impassable physical barrier. Entities reflect off walls.
- `*` **Food** (Green): Energy source spawned based on RAM usage.

---

## 🧬 Evolution Mechanics

### The Brain (Recurrent Architecture)

Each entity possesses a **Recurrent Neural Network** (RNN-lite) that evolves over generations.

- **Inputs (Sensors)**:
    - Environmental (Vision, Energy, Pheromones, Tribe density)
    - **Memory**: 6 inputs are reserved for the previous tick's internal state.
- **Outputs (Actions)**:
    - Move X / Y, Boost, Attack, Share.

### Genetics

When entities reproduce, their offspring inherits a mix of parents' DNA with slight mutations.

- **Attributes**: Speed, Strength, Metabolism, Color.
- **Brain**: Weights and biases are mutated.

### Pheromones

Entities leave chemical trails:

- **Food Trail**: "I found food here" (Attracts others).
- **Danger Trail**: "I died here" (Warns others).

---

## 🌍 Ecosystem

### Weather & Cycles

- **Seasons**: Change cyclically, affecting food growth rates and metabolism.
- **Circadian Rhythms**: A Day/Night cycle pulses through the world.
    - **Day**: Peak light levels drive maximum food growth.
    - **Night**: Minimal growth; entities enter a "Resting" state with 40% lower idle metabolism.

### Pathogens & Immunity

Microscopic threats can emerge and spread:

- **Contagion**: Disease spreads through proximity.
- **Evolution**: Surviving an infection boosts `Immunity`.

### Disasters

- **Dust Bowl**: Occurs during heat waves under high population stress, turning plains into barren wasteland.

---

## ⚔️ Game Modes

Launch Primordium with different rulesets using `--gamemode`: `standard`, `coop`, `battle`.

---

## 🌌 Multiplayer

Primordium supports **Interstellar Migration**. Travel off the edge while "Online" to migrate to other users' universes.

---

## 📚 Technical Wiki

- [Genetics & HexDNA](../docs/wiki/GENETICS.md)
- [Neural Network Architecture](../docs/wiki/BRAIN.md)
- [Ecosystem Formulas](../docs/wiki/ECOSYSTEM.md)

---
*Last Updated: 2026-01-21*
