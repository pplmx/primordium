# Genetics & HexDNA Protocol (Phase 50)

Primordium uses a hexadecimal-based string format ("HexDNA") to export and import entity genomes. This allows users to share creatures or analyze their genetic makeup with 100% fidelity.

## Unified Genotype Structure

In Phase 50, the genome is unified into a single `Genotype` structure. Brain and Body are no longer separate entities but part of a single inheritable payload.

1. **Neural Block**: Encodes the topology, weights, and biases of the dynamic graph-based brain.
2. **Phenotypic Block**: Encodes physical attributes (Sensing, Speed, Energy, etc.).
3. **Life History Block**: Encodes reproductive strategies and developmental timing.

### HexDNA Format

The entire `Genotype` struct is serialized via `serde_json` and converted to a hexadecimal string.
`[GenotypeHex]`

---

## Gene Catalog

| Gene | Range | Impact |
| :--- | :--- | :--- |
| **Sensing Range** | 3.0 - 15.0 | Radius of environmental perception. |
| **Max Speed** | 0.5 - 3.0 | Maximum velocity achievable. |
| **Max Energy** | 100 - 500 | Total energy storage capacity (coupled to Maturity). |
| **Metabolic Niche** | 0.0 - 1.0 | Specialization for nutrient types (0.0=Green/Plains, 1.0=Blue/Mountains). |
| **Trophic Potential**| 0.0 - 1.0 | Dietary strategy (0.0=Herbivore, 1.0=Carnivore). |
| **Reproductive Investment** | 0.1 - 0.9 | % of parental energy transferred to offspring. |
| **Maturity Gene** | 0.5 - 2.0 | Multiplier for maturation time and energy ceiling. |
| **Mate Preference** | 0.0 - 1.0 | Sexual selection bias (preferred Trophic Potential in mates). |
| **Pairing Bias** | 0.0 - 1.0 | Tendency to form long-term bonds vs. opportunistic mating. |

---

## Mutation Logic

Mutation is the primary driver of diversity. The engine uses a population-aware scaling system to balance exploration and stability.

### Mutation Rates & Amounts

- **Base Mutation Rate**: Defined in `EvolutionConfig` (default ~0.1).
- **Base Mutation Amount**: Defined in `EvolutionConfig` (default ~0.1).

### Population-Aware Scaling

The mutation intensity scales dynamically based on the current population:

- **Bottleneck (< Threshold)**: Scaling up to **3.0x** to encourage rapid adaptation.
- **Stasis (> Threshold)**: Scaling down to **0.5x** to preserve successful traits.
- **Genetic Drift**: If population < 10, random traits may be completely randomized (5% chance) to simulate stochastic drift.

### Mutation Ranges (± Amount)

| Gene | Mutation Type | Range / Constraint |
| :--- | :--- | :--- |
| **Brain** | Topology/Weights | NEAT-lite mutation rules. |
| **Sensing Range** | Multiplicative | ± Amount * Current Value (Clamp 3-15). |
| **Max Speed** | Multiplicative | ± Amount * Current Value (Clamp 0.5-3). |
| **Maturity Gene** | Additive | ± Amount (Clamp 0.5-2.0). |
| **Max Energy** | Derived | $200.0 \times MaturityGene$ (Clamp 100-500). |
| **Metabolic Niche** | Additive | ± Amount (Clamp 0.0-1.0). |
| **Trophic Potential**| Additive | ± Amount (Clamp 0.0-1.0). |
| **Reproductive Investment**| Additive | ± Amount (Clamp 0.1-0.9). |
| **Mate Preference** | Additive | ± Amount (Clamp 0.0-1.0). |
| **Pairing Bias** | Additive | ± Amount (Clamp 0.0-1.0). |

---

## Life History Strategies: r vs K Selection

Phase 50 formalizes the trade-off between reproductive speed and individual robustness through the coupling of `Maturity Gene` and `Reproductive Investment`.

### r-Selection (Opportunistic)

- **Genes**: Low Maturity (< 1.0), Low Investment (< 0.4).
- **Strategy**: Produce many offspring quickly. Each offspring starts with low energy but reaches adulthood fast.
- **Environment**: Favored in unstable, high-disaster environments where individual survival is low.

### K-Selection (Robust)

- **Genes**: High Maturity (> 1.5), High Investment (> 0.7).
- **Strategy**: Produce few, high-quality offspring. Offspring start with massive energy reserves and have a high `Max Energy` ceiling but take much longer to mature.
- **Environment**: Favored in stable, high-competition "Biodiversity Hotspots" where individual efficiency is paramount.

---

## Genetic Distance & Speciation

The distance $D$ between two genomes determines if they belong to the same lineage.

$$ D = D_{brain} + \sum \frac{|Trait_{A} - Trait_{B}|}{Scale} $$

### Scaling Factors

- **Sensing Range**: 5.0
- **Max Speed**: 1.0
- **Max Energy**: 100.0
- **Metabolic Niche**: 1.0
- **Trophic Potential**: 1.0
- **Pairing Bias**: 1.0

If $D > SpeciationThreshold$ (default 5.0), a new `lineage_id` is generated, marking a macroevolutionary split.

---

## Registry & Archeology

- **Lineage Registry**: Tracks ancestry, founding DNA, and success metrics in `logs/lineages.json`.
- **Fossil Record**: Extinct legendary genotypes (high rank/age) are preserved in `logs/fossils.json`.
- **Pruning**: Extinct lineages with low impact (< 3 total entities) are periodically removed to maintain performance.
