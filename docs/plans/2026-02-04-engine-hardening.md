# Design Plan: Engine Hardening & Optimization (Feb 2026)

## 1. Overview
This plan outlines critical updates to the Primordium core simulation engine to improve stability (panic prevention), performance (complexity reduction), and architectural elegance.

## 2. Stability: The Zero-Panic Policy
### Problem
The hot loop in `src/model/world/systems.rs:219` uses `.unwrap()` on a partner's genotype. In a high-concurrency simulation, any inconsistency in entity state could trigger a full process crash.

### Solution
- Replace `.unwrap()` with `if let Some(partner_genotype) = partner_snap.genotype.as_ref()`.
- Log a debug warning if the genotype is missing instead of panicking.
- Audit `infra/network.rs` for `expect()` calls and replace them with `anyhow::Result` propagation or safe defaults.

## 3. Performance: Pre-calculated Social Context
### Problem
Predation defense logic currently performs nested spatial queries ($O(N \cdot M^2)$), which spikes CPU usage in high-density regions.

### Solution
- **LineagePowerGrid**: Introduce a new grid system in `primordium_core::spatial_hash`.
- **Pass 1 Calculation**: During the spatial indexing pass, accumulate "Lineage Power" (based on Social Rank and proximity) into a coarse grid.
- **Pass 2 Lookup**: Predation logic will use an $O(1)$ lookup in this grid to determine defense multipliers instead of searching for neighbors again.

## 4. Memory: Allocation-Free Interaction Pipeline
### Problem
Collection of `Vec<InteractionCommand>` per entity creates 10,000+ tiny heap allocations every tick.

### Solution
- Use `SmallVec<[InteractionCommand; 4]>` in `perceive_and_decide_internal`.
- Use `interaction_commands.reserve(pop_len)` and `extend` to reuse the main command buffer capacity.

## 5. Architecture: System Decomposition
### Problem
`perceive_and_decide_internal` is a monolithic "God Function" exceeding 150 lines of complex logic.

### Solution
- Split into `PerceptionSystem`, `InferenceSystem`, and `InteractionFactory`.
- Standardize RNG handling using a unified `DeterministicRng` trait for the simulation world.

## 6. Verification Plan
- **Stability**: Run `stress_test` for 10,000 ticks with high mutation rates.
- **Performance**: Benchmark UPS (Updates Per Second) before and after changes.
- **Accuracy**: Ensure the new `LineagePowerGrid` behavior matches the original neighbor-counting logic (or improves upon it).
