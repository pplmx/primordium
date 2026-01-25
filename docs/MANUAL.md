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
| `c` | **Export DNA** of selected entity to `exported_dna.txt` |
| `C` | **Export Brain JSON** of selected entity to `logs/brain_<id>.json` |
| `v` | **Infuse DNA** from `dna_infuse.txt` |
| `a` | Toggle **Ancestry View** (Family Tree) |
| `Shift+A` | Export Ancestry Tree to DOT file |
| `y` | Toggle **Archeology & Fossil Record** |
| `[` / `]` | **Time Travel** (Navigate History Snapshots) |
| `+` / `-`| Increase / Decrease time scale |
| `1 2 3 4 5` | **View Modes**: Normal, Fertility, Social, Rank, Vocal |
| `j` | Toggle **Brush Mode** (Terrain / Social) |
| `! @ #` | **Social Brush**: Neutral, Peace, War |
| `$ % ^` | **Terrain Brush**: Oasis, Wall, Barren |
| `Shift+1..5` | Navigate Help Tabs (when open) |

### Mouse Controls

| Input | Action |
| ----- | --------- |
| `Left Click` | Select an entity / Change help tab |
| `Left Drag` | **Paint Terrain** with selected brush |
| `Right Click`| Inject Food Cluster |

### View Modes (Phase 50)

- **1: Normal**: Standard view.
- **2: Fertility**: Green heatmap showing soil quality.
- **3: Social Zones**: Cyan overlay showing Peace/War zones.
- **4: Rank Heatmap** 👑: Purple/Magenta gradients revealing social stratification and Alpha leadership strength.
- **5: Vocal Propagation** 🔉: Yellow ripples visualizing real-time sound wave propagation.

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
- `⚔` **Soldier**: High-rank, aggressive defender. Deals 1.5x damage.

### Specialized Castes (Phase 53)

Entities can evolve specialized roles based on their activities:

- **Soldier**: High damage dealer (1.5x). Higher metabolic cost.
- **Engineer**: Expert at terraforming. 50% lower energy cost for Dig/Build.
- **Provider**: Altruistic sharer. 50% lower energy penalty when sharing.

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
- `Ω` **Nest** (Gold): Protective structures built by entities. Grant metabolic recovery and energy boost for offspring.
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
- **Collective Reinforcement (Phase 50)**: Hebbian learning now rewards social coordination (Vocalization sync) in addition to basic survival.

### Genetics & Adaptation

When entities reproduce, their offspring inherits a mix of parents' DNA with slight mutations.

- **Attributes**: Speed, Range, Metabolism, Niche, Sexual Preference.
- **Brain**: Topology and weights are mutated.
- **Population-Aware Mutation**:
    - **Bottleneck**: In small populations, mutation rates increase (up to 3x) to find survival strategies.
    - **Stasis**: In large stable populations, mutation is halved to preserve fit genes.
- **Genetic Drift**: Tiny populations (<10) may experience random major trait flips.

### Social Hierarchy (Phase 49 & 50)

Tribes are organized into hierarchies based on a **Rank** score (Energy + Age + Offspring + Reputation).

- **Alpha Leadership**: Entities are influenced by the movement of the highest-ranking local member (Alpha). Alphas emit a **Leadership Aura** (visible in View 4) that guides nearby kin.
- **Soldier Caste**: Entities with High Rank (>0.8) and High Aggression (>0.5) become Soldiers. They are the primary defenders of the tribe and possess a **Combat Aura**.
- **Dynamic Territoriality**: Alphas can automatically claim and mark zones. High-rank presence reinforces tribal boundaries, creating implicit **Peace** or **War** zones based on collective aggression.
- **Tribal Splitting**: If a low-ranking entity (Omega) is trapped in an overcrowded area, it may initiate a **Fracture**, changing color and starting a new, rival tribe to escape the competitive pressure.

### Social Brushes (Phase 50)

Use `j` to toggle between **Terrain** and **Social** painting modes. Social brushes allow divine intervention in tribal relations:

- **Neutral** (`!`): Clears any social zone override.
- **Peace Zone** (`@`): Enforces non-aggression within the area. Entities are discouraged from attacking, regardless of neural state.
- **War Zone** (`#`): 2x Damage multiplier for Soldiers. High-aggression area that triggers predatory instincts.

---

## 🌍 Ecosystem

### Archeology & Fossils (Phase 40)

Primordium preserves the deep history of your world through two key mechanisms:

- **History Snapshots**: Every 1,000 ticks, the system captures a macro-state of the world (population, carbon, hotspots). Use the Archeology View (`y`) and Time Travel keys (`[`/`]`) to browse these snapshots.
- **Fossil Record**: When a legendary lineage goes extinct, its genetic legacy and brain architecture are "fossilized" into a persistent registry (`logs/fossils.json`). You can view these ancestral icons in the Archeology panel.

### Carbon Cycle & Atmospheric Chemistry (Phase 56)

The simulation features a global **Carbon & Oxygen Cycle**:

- **CO2 (Carbon)**: Metabolic activity increases carbon. High carbon triggers Global Warming.
- **O2 (Oxygen)**: Plant biomass and Forests produce Oxygen via photosynthesis.
- **Metabolism**: Entities consume Oxygen. High O2 increases movement efficiency; low O2 (< 8%) causes hypoxic stress (extra energy drain).
- **Global Warming**: High CO2 levels shift the climate state towards **Scorching**.

### Weather & Cycles

- **Seasons**: Change cyclically, affecting food growth rates and metabolism.
- **Circadian Rhythms**: A Day/Night cycle pulses through the world.
    - **Day**: Peak light levels drive maximum food growth.
    - **Night**: Minimal growth; entities enter a "Resting" state with 40% lower idle metabolism.

### Pathogens & Parasites (Phase 55)

Microscopic threats can emerge and spread:

- **Contagion**: Disease spreads through proximity.
- **Behavioral Hijacking**: Parasitic pathogens can force specific brain outputs (Aggression, Vocalization, Random Movement) to increase spread.
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
- [History & Archeology](../docs/wiki/HISTORY.md)

---
*Last Updated: 2026-01-24*
