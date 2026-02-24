# Agent Project Memory: primordium_core

> Core simulation engine crate containing the biological simulation logic.

## OVERVIEW

Core simulation engine implementing NEAT neural networks, spatial indexing, and multi-system entity lifecycle management.

## STRUCTURE

```
src/
├── brain/              # Neural network (topology, mutation, crossover, forward)
├── systems/            # Simulation systems (action, biological, civilization, ecological, environment, history, intel, interaction, social, stats, audio)
├── terrain/            # Terrain system (disasters, generation, succession)
├── spatial_hash.rs     # Spatial indexing for entity queries
├── lifecycle.rs        # Entity birth/death/reproduction
├── lineage_registry.rs # Global lineage tracking
├── lineage_tree.rs     # Ancestry tree visualization
├── snapshot.rs         # EntitySnapshot for parallel updates
├── pheromone.rs        # Chemical signaling grid
├── sound.rs            # Acoustic propagation grid
├── pathogen.rs         # Disease mechanics
├── influence.rs        # Social influence tracking
├── pressure.rs         # Build/dig activity tracking
├── metrics.rs          # Performance metrics
├── history.rs          # Event logging
├── blockchain.rs       # OpenTimestamps anchoring
├── environment.rs      # Hardware coupling (CPU/RAM)
├── food.rs             # Food spawning logic
├── interaction.rs      # Entity interaction rules
└── config.rs           # Simulation configuration
```

## WHERE TO LOOK

**Simulation Entry Point**: `World::update` (in parent crate) orchestrates all systems in fixed order.
**Neural Networks**: `brain/` module - NEAT-lite topology evolution with 29-6-12 architecture (47 nodes).
**Spatial Queries**: `spatial_hash.rs` - O(1) entity proximity lookups for perception and interaction.
**Parallel Execution**: `systems/` modules use Rayon with `EntitySnapshot` pattern for thread-safe updates.
**Entity Lifecycle**: `lifecycle.rs` - birth, death, reproduction, and HexDNA serialization.
**Lineage Tracking**: `lineage_registry.rs` (global) + `lineage_tree.rs` (visualization).
**Environmental Coupling**: `environment.rs` - CPU/RAM metrics drive climate and resource scarcity.
**Grid Systems**: `pheromone.rs`, `sound.rs`, `pressure.rs` - stigmergic communication layers.
**Terrain Dynamics**: `terrain/` module - succession, disasters, generation.

## CONVENTIONS

**System State**: Systems are stateless functions taking `&World` and `&mut State`. All mutable state lives in `State` struct.
**Parallel Phases**: Perception, Intel, and Biological systems run in parallel via Rayon. Use `EntitySnapshot` for read-only entity access.
**Proposal Pattern**: Action systems generate proposals first, then apply sequentially to avoid race conditions.
**Spatial Hash**: Always rebuild before parallel phases. Use `query_radius` for sensory lookups.
**Brain Updates**: Neural inference is pure function. Mutations happen during reproduction only.
**Grid Decay**: Pheromones and sounds decay every tick in `systems::environment`.

## ANTI-PATTERNS

**Direct Entity Mutation in Parallel**: Never mutate entities directly in parallel phases. Use proposals or buffer mutations.
**Cross-System State**: Don't share mutable state between systems. Use `State` struct as single source of truth.
**Skipping Snapshot**: Always use `EntitySnapshot` in parallel systems, never direct `&World.entities` access.
**Blocking in Parallel**: Avoid blocking operations (I/O, locks) in Rayon parallel closures.
**Grid Bounds**: Always check grid bounds before accessing `pheromone.grid[x][y]` or `sound.grid[x][y]`.
**Brain Mutation Timing**: Never mutate brain topology outside of reproduction. Only weights update via Hebbian learning.
