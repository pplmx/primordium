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
4. **Speciation**: A **2% chance** for a massive shift in `trophic_potential` (±0.4), potentially flipping the organism's ecological role.

## Phenotypic Genes

Phase 23 introduces a set of specialized phenotypic genes that define the physical capabilities of an entity. These traits are encoded in the Genotype and are subject to the same evolutionary pressures as neural weights.

| Gene | Range | Impact |
| ---- | ----- | ------ |
| **Sensing Range** | 3.0 - 15.0 | Radius of environmental perception. |
| **Max Speed** | 0.5 - 3.0 | Maximum velocity achievable. |
| **Max Energy** | 100 - 500 | Total energy storage capacity. |
| **Metabolic Niche** | 0.0 - 1.0 | Specialization for nutrient types (Green vs Blue). |
| **Trophic Potential**| 0.0 - 1.0 | Dietary strategy (Herbivore vs Carnivore). |

## Life History Genes (Phase 32)

Phase 32 introduces **Life History Strategies**, allowing lineages to evolve different reproductive and developmental patterns.

| Gene | Range | Impact |
| ---- | ----- | ------ |
| **Reproductive Investment** | 0.1 - 0.9 | The ratio of parental energy transferred to offspring. |
| **Maturity Gene** | 0.5 - 2.0 | Multiplier for standard maturity age (150 ticks). |

### Growth vs Size Trade-off

An entity's **Max Energy** (stomach size) is now coupled with its **Maturity Gene**. Larger maturity multipliers allow for significantly larger energy capacities, enabling a "Slow and Steady" survival strategy.

- **Fast Maturity** (< 1.0): Smaller energy capacity, faster generation turnover.
- **Slow Maturity** (> 1.0): Massive energy capacity, longer developmental period.

## Genotype Structure (Unified)

The genome is now unified into a single `Genotype` structure that encapsulates both physical traits and neural parameters.

1. **Phenotypic Block**: Encodes physical attributes (Sensing, Speed, Energy, etc.)
2. **Neural Block**: Encodes the weights and biases of the RNN-lite brain.

Attributes are no longer static "labels" but are fully mutable and inheritable traits. A mutation in the Phenotypic Block directly alters the physical manifestation of the entity in the next generation.

## Macroevolution & Ancestry (Phase 34)

Phase 34 shifts the analytical focus from individual survival to the structural evolution of the entire ecosystem. Every lineage is now part of a branching tree built using parent/child relationships.

### Lineage Registry

The simulation maintains a persistent registry of all lineages in `logs/lineages.json`. This file tracks:
- **Ancestry**: The parent lineage ID for every branch.
- **Founding DNA**: The HexDNA of the original ancestor that triggered the speciation event.
- **Success Metrics**: Peak population, total lifespan, and trophic dominance.

### The Tree of Life

By utilizing `petgraph`, the engine constructs a real-time Directed Acyclic Graph (DAG) representing the branching history of all active and historical lineages.
- **Branching**: When a mutation leads to a significant genetic distance from the current lineage norm, a new "child" lineage is registered.
- **Persistence**: Lineage data is preserved across sessions, allowing for long-term evolutionary studies.
- **Visualization**: The TUI (via the `A` key) provides a live view of this tree, highlighting the top 5 dominant dynasties and their trophic strategies.
