# Genetics & HexDNA Protocol

Primordium uses a hexadecimal-based string format ("HexDNA") to export and import entity genomes. This allows users to share creatures or analyze their genetic makeup.

## DNA Structure

The genome consists of two parts:

1. **Attributes**: Fixed genetic traits (Speed, Color, etc.)
2. **Brain**: Neural network weights and biases.

### Format

`[GenotypeHex]`

The entire `Genotype` struct is serialized via `serde_json` and converted to a single hexadecimal string. This ensures 100% fidelity for both physical traits and neural weights.

## Mutation Logic

Each time an entity reproduces:

1. **Crossover**: Offspring takes attributes from both parents (50/50 chance per gene).
2. **Standard Mutation**: A probability (**0.1**) to alter a gene/weight value by ±0.2.
3. **Genetic Drift**: A small probability (**0.01**) for a "macro-mutation" (±0.5 change).
4. **Speciation**: A **2% chance** to flip the entity's trophic role (**Herbivore ↔ Carnivore**).

## Phenotypic Genes

Phase 23 introduces a set of specialized phenotypic genes that define the physical capabilities of an entity. These traits are encoded in the Genotype and are subject to the same evolutionary pressures as neural weights.

| Gene | Range | Impact |
| ---- | ----- | ------ |
| **Sensing Range** | 3.0 - 15.0 | Radius of environmental perception. |
| **Max Speed** | 0.5 - 3.0 | Maximum velocity achievable. |
| **Max Energy** | 100 - 500 | Total energy storage capacity. |

## Genotype Structure (Unified)

The genome is now unified into a single `Genotype` structure that encapsulates both physical traits and neural parameters.

1. **Phenotypic Block**: Encodes physical attributes (Sensing, Speed, Energy, etc.)
2. **Neural Block**: Encodes the weights and biases of the RNN-lite brain.

Attributes are no longer static "labels" but are fully mutable and inheritable traits. A mutation in the Phenotypic Block directly alters the physical manifestation of the entity in the next generation.

## Lineage Tracking (Phase 24)

Phase 24 introduces **Lineage Tracking**, allowing the simulation to monitor the long-term success and diversification of ancestral lines. Every entity is now part of a specific lineage that can be traced back to the original population.

### Ancestral Persistence

- **Dynastic ID**: Each entity carries a `lineage_id` within its `Genotype`.
- **Crossover Inheritance**: During sexual reproduction, the offspring inherits the `lineage_id` from the **first parent** (the one that initiated the mating or is considered the "primary" ancestor in the pairing).
- **Mutation Stability**: Mutation and macro-mutation do not change the `lineage_id`. A lineage remains consistent even as its physical and neural traits diverge significantly over time.
- **Cross-Universe Preservation**: When an entity migrates between simulation instances (Global Hive), its `lineage_id` is preserved, allowing for the tracking of "interstellar" dynasties across the multiverse.

This system enables the visualization of macroevolutionary trends, such as which original ancestors' descendants currently dominate the ecosystem.
