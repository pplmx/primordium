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

## 💡 Lessons Learned & Gotchas

1. **Clippy Sensitivity**: In tests, avoid `let mut x = X::default(); x.field = val;`. Prefer `let mut x = X { field: val, ..X::default() };` to avoid `field_reassign_with_default`.
2. **DNA Serialization**: `import_migrant` requires actual HexDNA string parsing via `Brain::from_hex`.
3. **TUI Layout**: Always check `f.size()` and provide minimum dimensions for modals to avoid panic on small terminals.

## 🛠️ Tooling & Productivity

- **Search**: Prefer `rg` (ripgrep) over `grep` for faster, recursive code searching.
- **Find**: Prefer `fd` (or `fdfind`) over `find` for cleaner syntax and speed.
- **Consistency**: Avoid PowerShell-specific syntax in bash commands to ensure scripts and hooks remain portable and consistent across environments.
