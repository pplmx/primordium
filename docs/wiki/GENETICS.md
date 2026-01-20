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
2. **Mutation**: A small probability (0.01) to alter a gene value by ±5%.

## Attributes Table
| Attribute | Impact |
|-----------|--------|
| **Speed** | Max movement range per tick. Higher cost. |
| **Strength**| Damage dealt in combat. Higher cost. |
| **Color** | Defines tribe membership. |
| **Digestion**| Efficiency of converting food to energy. |
