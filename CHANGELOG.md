# Changelog - Primordium

[简体中文](./docs/CHANGELOG_zh.md)

All notable changes to the **Primordium** project will be documented in this file. This project adheres to a phase-based evolutionary development cycle.

---

## [Phase 21: Environmental Fluidity & Disasters] - 2026-01-21

### Evolutionary Leap: Temporal Coherence & Dynamic Crises

The digital ecosystem has gained memory and faces its first environmental catastrophes, requiring more sophisticated survival strategies.

#### ✨ Features

- **Recurrent Neural Networks (RNN-lite)**: Upgraded Brain architecture to include feedback loops from the previous tick's hidden state, enabling time-coherent behavior and internal memory.
- **Dust Bowl Disaster**: Introduced dynamic terrain events where high population stress and heat waves trigger widespread soil depletion and barren transitions.
- **Physical Barriers**: Added impassable `Wall` terrain types that force organisms to evolve steering and obstacle avoidance.
- **Proximity-Based Sensing**: Migrated from global food knowledge to a realistic "Sensing Radius" (20 units) powered by a dedicated `food_hash`.

#### 🛠️ Technical Achievements

- **O(1) Sensing**: Integrated a second Spatial Hash specifically for resources, reducing sensory query complexity.
- **Buffer Pooling**: Implemented reusable heap allocations within the `World` struct, significantly reducing allocation overhead during parallel execution.
- **Stateful Intelligence**: Added persistent `last_hidden` states to the `Intel` component, supporting the new recurrent brain architecture.
- **Physics Engine Update**: Enhanced `handle_movement` to support collision detection and reflection against impassable terrain.

---

## [Phase 20: Cognitive Synthesis & Systemic Refactor] - 2026-01-21

### Evolutionary Leap: Component-Based Life & Parallel Intelligence

The simulation has undergone its most significant architectural evolution yet, transitioning to a modular, component-based system designed for peak performance and extreme scalability.

#### ✨ Features

- **Component-Based Entity (CBE)**: Organism attributes are now logically grouped into Physics, Metabolism, Health, and Intel components, improving system isolation and data locality.
- **Systemic Decomposition**: The world update loop is now a pipeline of specialized "Systems" (Perception, Action, Biological, Social), making the logic easier to extend and maintain.
- **Rayon-Powered Parallelism**: Integrated the Rayon data-parallelism library to handle heavy computational loads across all CPU cores.
- **High-Density Scaling**: Optimized for 5000+ simultaneous entities with multi-threaded neural processing.

#### 🛠️ Technical Achievements

- **Parallel Brain Inference**: Neural network forward passes are now processed in parallel using `.par_iter()`.
- **Systemic Pipeline**: Refactored `World::update` into discrete stages, resolving long-standing "monolith" technical debt.
- **Data Locality**: Component grouping allows the simulation to process only relevant data subsets per system, reducing cache misses.
- **Logic Parity**: Achieved 100% functional parity with previous versions while dramatically increasing throughput.

---

... (rest of file)
