# Agent Project Memory: Primordium

This file contains critical project-specific knowledge for AI agents working on Primordium.

## 🏗️ Architecture & Layout

### TUI Structure (ratatui)
- **Status Bar (Top)**: 4 lines.
  1. CPU Gauge + Era Icon/Name.
  2. RAM Gauge + Resource State Icon.
  3. World Stats (Pop, Species, Gen, AvgLife, Entropy).
  4. **Legend Bar**: Displays symbols for Entity Status (●♦♥†♣) and Terrain (▲≈◊*).
- **Sparklines (Middle-Top)**: 2 panes for CPU Load and Population Health.
- **World (Center)**: The main simulation grid.
- **Live Chronicle (Bottom)**: Scrolling log of events (Birth, Death, Climate Shift, etc.).
- **Brain Panel (Right)**: Neural network visualization (toggled with 'B').

### Project Modularization
- `src/app/`: Split from the original monolithic `app.rs`.
  - `mod.rs`: Run loop and high-level update logic.
  - `state.rs`: `App` struct fields and initialization.
  - `render.rs`: TUI rendering logic.
  - `input.rs`: Event handling (Key/Mouse).
  - `onboarding.rs`: First-run tutorial screens.
  - `help.rs`: Tabbed help system.

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

- **Life Cycles**: Entities are born as **Juveniles** (◦). They must survive for `maturity_age` (default 150 ticks) before reaching adulthood. Only adults can reproduce.
- **Trophic Levels**:
  - **Herbivores (H-)**: Primary consumers. Eat plant food (`*`). 0.2x predation gain.
  - **Carnivores (C-)**: Predators. Cannot eat plants. 1.2x predation gain.
- **Terrain Health**: Each cell has a fertility level. Eating food depletes fertility. If fertility < 0.2, the cell becomes **Barren** (░) and stops spawning food until it recovers.
- **Pathogens**: Microscopic threats that spread between nearby entities.
  - **Transmission**: Depends on virulence vs host immunity.
  - **Effects**: Constant energy drain.
  - **Immunity**: Gained through survival and inherited with minor mutation.
- **Circadian Rhythms**: A Day/Night cycle (default 2000 ticks) that governs the world.
  - **Light Levels**: Higher food growth during the day; minimal growth at night.
  - **Metabolism**: Energy cost is reduced by 40% at night (rest period).

## 🧪 Testing Strategy

- **Unit Tests**: Located in `src/model/*.rs`. Fast, isolated.
- **Integration Tests**: Located in `tests/`.
  - `simulation_logic.rs`: Lifecycle and reproduction.
  - `genetic_flow.rs`: DNA protocols (HexDNA).
  - `environment_coupling.rs`: Hardware-to-World mapping.
- **Critical Requirement**: `HistoryLogger` requires `logs/` directory. Implementation in `src/model/history.rs` now handles auto-creation.

## ⚓ Git Hooks (Husky)

- **pre-commit**: `cargo test` + `cargo fmt --all -- --check` + `cargo clippy --all-targets --all-features -- -D warnings`.
- **pre-push**: Full `cargo test` suite.
- **commit-msg**: Conventional Commits enforcement (`feat`, `fix`, `docs`, `refactor`, etc.).

## 📝 Maintenance Protocol

- **Synchronous Updates**: Whenever adding new features or changing functionality, you **MUST**:
  1.  Update existing tests or add new tests to cover the changes.
  2.  Update all relevant documentation in both **English and Chinese** (README, MANUAL, ARCHITECTURE, etc.).
  3.  Reflect any core logic changes in this `AGENTS.md` file.

## 💡 Lessons Learned & Gotchas

1. **Clippy Sensitivity**: In tests, avoid `let mut x = X::default(); x.field = val;`. Prefer `let mut x = X { field: val, ..X::default() };` to avoid `field_reassign_with_default`.
2. **DNA Serialization**: `import_migrant` requires actual HexDNA string parsing via `Brain::from_hex`.
3. **TUI Layout**: Always check `f.size()` and provide minimum dimensions for modals to avoid panic on small terminals.

## 🛠️ Tooling & Productivity

- **Search**: Prefer `rg` (ripgrep) over `grep` for faster, recursive code searching.
- **Find**: Prefer `fd` (or `fdfind`) over `find` for cleaner syntax and speed.
- **Consistency**: Avoid PowerShell-specific syntax in bash commands to ensure scripts and hooks remain portable and consistent across environments.
