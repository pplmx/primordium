# Agent Project Memory: Primordium

This file contains critical project-specific knowledge for AI agents working on Primordium.

## 🏗️ Architecture & Layout

### TUI Structure (ratatui)
- **Status Bar (Top)**: 4 lines.
  1. CPU Gauge + Era Icon/Name.
  2. RAM Gauge + Resource State Icon.
  3. World Stats (Pop, Species, Gen, AvgLife, Entropy).
  4. **Legend Bar**: Displays symbols for Entity Status (●♦♥†♣☣◦) and Terrain (▲≈◊░█*).
- **Sparklines (Middle-Top)**: 2 panes for CPU Load and Population Health.
- **World (Center)**: The main simulation grid.
- **Live Chronicle (Bottom)**: Scrolling log of events (Birth, Death, Climate Shift, etc.).
- **Brain Panel (Right)**: Neural network visualization (toggled with 'B').

### Project Modularization
- `src/app/`: Split from the original monolithic `app.rs`.
- `src/model/`: Simulation core.
  - `world.rs`: Systemic decomposition (Perception, Action, Biological, Social).
  - `entity.rs`: Component-based entity structure (Physics, Metabolism, Health, Intel).
  - `brain.rs`: Recurrent neural network (RNN-lite) with Rayon-parallelized inference.
  - `terrain.rs`: Dynamic terrain grid with fertility and disasters.

## ⚡ Performance & Scaling (Phase 20-21)

- **Parallel updates**: `Perception` and `Neural` systems use `Rayon` (`par_iter`).
- **Spatial Hashing**: Separate grids for `entities` and `food` for $O(1)$ sensing.
- **Buffer Pooling**: Reuse `Vec` and `HashSet` in `World` to minimize allocation jitter.
- **Snapshots**: Use `EntitySnapshot` for read-only parallel perception.

## 🔗 Hardware Resonance Logic

- **CPU Usage -> Climate**:
  - < 30%: Temperate (x1.0 metabolism)
  - 30-60%: Warm (x1.5)
  - 60-80%: Hot (x2.0)
  - > 80%: Scorching (x3.0)
- **RAM Usage -> Resource Scarcity**:
  - < 50%: Abundant
  - 50-70%: Strained
  - 70-90%: Scarce
  - > 90%: Famine (Death/Starvation risk high)

## 🧬 Biology & Ecology

- **Recurrent Brain**: 12 inputs (6 sensors + 6 memory inputs), 6 hidden, 5 outputs. Supports temporal coherence.
- **Life Cycles**: Entities are born as **Juveniles** (◦). Must survive for `maturity_age` (150 ticks) before reproducing.
- **Trophic Levels**: Herbivores (H-) vs Carnivores (C-).
- **Terrain Health**: Soil fertility regenerates at 0.001/tick. Barren state (░) below 0.15 fertility.
- **Disasters**: Dust Bowl triggered by Heat Wave + High Population. Wipes out fertility on Plains.
- **Pathogens**: Proximity-based transmission (radius 2.0). Immunity gained through survival.
- **Circadian Rhythms**: Day/Night cycle (2000 ticks). 40% metabolism reduction at night.

## 🧪 Testing Strategy

- **Unit Tests**: Located in `src/model/*.rs`.
- **Integration Tests**: Located in `tests/`.
  - `simulation_logic.rs`: Lifecycle and reproduction.
  - `genetic_flow.rs`: DNA protocols (HexDNA).
  - `ecology.rs`: Terrain fertility and trophic niche.
  - `pathogens.rs`: Contagion dynamics.
  - `disasters.rs`: Dust bowl trigger and collision physics.

## ⚓ Git Hooks (Husky)

- **pre-commit**: `cargo test` + `cargo fmt --all -- --check` + `cargo clippy --all-targets --all-features -- -D warnings`.
- **pre-push**: Full `cargo test` suite.

## 📝 Maintenance Protocol

- **Synchronous Updates**: Whenever adding new features or changing functionality, you **MUST**:
  1.  Update existing tests or add new tests to cover the changes.
  2.  Update all relevant documentation in both **English and Chinese** (README, MANUAL, ARCHITECTURE, etc.).
  3.  Reflect any core logic changes in this `AGENTS.md` file.

## 💡 Lessons Learned & Gotchas

1. **Clippy Sensitivity**: In tests, avoid `let mut x = X::default(); x.field = val;`. Prefer `let mut x = X { field: val, ..X::default() };` to avoid `field_reassign_with_default`.
2. **DNA Serialization**: `import_migrant` requires actual HexDNA string parsing via `Brain::from_hex`.
3. **Parallel Updates**: When using `Rayon` for entity updates, use the `EntitySnapshot` pattern and Buffer Pooling to avoid mutable borrow conflicts and allocation jitter.
4. **Disaster Sync**: Terrain disasters should be triggered by World and handled in TerrainGrid update.

## 🛠️ Tooling & Productivity

- **Search**: Prefer `rg` (ripgrep).
- **Find**: Prefer `fd` (or `fdfind`).
- **Consistency**: Avoid PowerShell-specific syntax in bash commands.
