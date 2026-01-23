# Genetics & HexDNA Protocol

Primordium uses a hexadecimal-based string format ("HexDNA") to export and import entity genomes. This allows users to share creatures or analyze their genetic makeup.

## DNA Structure

The genome consists of two parts:

1. **Attributes**: Fixed genetic traits (Speed, Color, etc.)
2. **Brain**: Neural network weights and biases.

### Format

`[Generation]-[AttributesHex]-[BrainHex]`

## Mutation Logic

Each time an entity reproduces:

1. **Crossover**: Offspring takes attributes from both parents (50/50 chance per gene).
2. **Standard Mutation**: A probability (**0.1**) to alter a gene/weight value by ±0.2.
3. **Genetic Drift**: A small probability (**0.01**) for a "macro-mutation" (±0.5 change).
4. **Speciation**: A **2% chance** to flip the entity's trophic role (**Herbivore ↔ Carnivore**).

## Attributes Table

| Attribute | Impact |
| ----------- | --------- |
| **Speed** | Max movement range per tick. Higher cost. |
| **Strength**| Damage dealt in combat. Higher cost. |
| **Color** | Defines tribe membership. |
| **Digestion**| Efficiency of converting food to energy. |
