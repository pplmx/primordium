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
| `b` | Toggle **Neural Brain Visualization** |
| `h` | Toggle **Help Overlay** |
| `x` | Trigger **Genetic Surge** (Global Mutation) |
| `m` | **Mutate** selected entity |
| `k` | **Smite** (Kill) selected entity |
| `p` | **Reincarnate** (Reset DNA) selected entity |
| `! @ # $ % ^` | Select **Terrain Brush** (Plains, Mt, River, Oasis, Wall, Barren) |
| `Shift+K` | Toggle **Heat Wave** Disaster |
| `l` | Trigger **Mass Extinction** (90% wipe) |
| `r` | Trigger **Resource Boom** (Spawn Food) |
| `w` | **Save** Simulation State to `save.json` |
| `o` | **Load** Simulation State from `save.json` |
| `a` | Toggle **Ancestry View** (Family Tree) |
| `Shift+A` | Export Ancestry Tree to DOT file |
| `+` / `-`| Increase / Decrease time scale |
| `1 2 3 4` | Navigate Help Tabs (when open) |

### Mouse Controls

| Input | Action |
| ----- | --------- |
| `Left Click` | Select an entity / Change help tab |
| `Left Drag` | **Paint Terrain** with selected brush |
| `Right Click`| Inject Food Cluster |

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

### Terrain & Succession

- ` ` **Plains**: Standard movement speed.
- `≈` **River** (Blue): Faster movement (1.5x), represents water currents.
- `▲` **Mountain** (Gray): Slow movement (0.5x), no food growth.
- `◊` **Oasis** (Green): Prime real estate with 3x food spawn rate.
- `♠` **Forest** (Dark Green): Carbon sink with high food yield (2.0x). Plains transition to Forest under high fertility and plant biomass.
- `▒` **Desert** (Tan): Resource-poor, high heat stress land. Plains degrade to Desert under low fertility.
- `░` **Barren** (Brown): Overgrazed or disaster-struck land. Very low food growth.
- `█` **Wall** (Dark Gray): Impassable physical barrier.
- `*` **Food** (Green): Energy source spawned based on RAM usage.

---

## 🧬 Evolution Mechanics

### The Brain (Recurrent Architecture)

Each entity possesses a **Recurrent Neural Network** (RNN-lite) that evolves over generations.

- **Inputs (Sensors)**:
    - Environmental (Vision, Energy, Pheromones, Tribe density)
    - **Memory**: 6 inputs are reserved for the previous tick's internal state.
- **Outputs (Actions)**:
    - Move X / Y, Boost, Attack, Share, Signal.

### Genetics & Adaptation

When entities reproduce, their offspring inherits a mix of parents' DNA with slight mutations.

- **Attributes**: Speed, Range, Metabolism, Niche, Sexual Preference.
- **Brain**: Topology and weights are mutated.
- **Population-Aware Mutation**: 
    - **Bottleneck**: In small populations, mutation rates increase (up to 3x) to find survival strategies.
    - **Stasis**: In large stable populations, mutation is halved to preserve fit genes.
- **Genetic Drift**: Tiny populations (<10) may experience random major trait flips.

---

## 🌍 Ecosystem

### Carbon Cycle & Warming

The simulation features a global **Carbon Cycle**:
- **Emissions**: Metabolic activity from all entities increases atmospheric CO2.
- **Sequestration**: Plant biomass and Forests act as carbon sinks.
- **Global Warming**: High CO2 levels shift the climate state towards **Scorching**, increasing metabolic drain for all life.

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
