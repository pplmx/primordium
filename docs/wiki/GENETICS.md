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

Mutation is the primary driver of diversity. The engine uses a population-aware scaling system and environmental forcing to balance exploration and stability.

### Environmental & Network Forcing (Phase 61)

The mutation rate is no longer purely internal. Environmental events, potentially synchronized across the Hive network, can force rapid adaptation.

- **Radiation Storms**: During "Solar Flare" events, mutation rates increase by **5.0x** and mutation amounts by **2.0x**. This triggers a period of "Adaptive Radiation" where lineages must rapidly explore new genetic configurations to survive increased metabolic stress.

### Ancestral Traits & Epigenetics (Phase 61)

Success leaves a mark beyond the immediate genotype. High-fitness lineages accumulate **Ancestral Traits** that provide persistent physical buffs.

| Trait | Requirement | Impact |
| :--- | :--- | :--- |
| **Hardened Metabolism** | 2000+ ticks survival | -20% Base Idle Cost |
| **Acute Senses** | Dominant rank | +20% Perception Range |
| **Swift Movement** | 50+ population | +10% Max Speed |

These traits are inherited trans-generationally and are applied directly to the phenotype, representing the "Epigenetic Memory" of the ancestral line.

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

---

## HexDNA 2.0 Protocol

HexDNA 2.0 is the canonical genetic serialization format for Primordium, enabling 100% fidelity genome transfer between simulation instances, version migrations, and cross-instance entity migration via the P2P network.

### Format Specification

HexDNA 2.0 encodes the complete `Genotype` structure as a hexadecimal string representation of the JSON-serialized genome.

```
[HexDNA_Version][Block_Delimiter][Genotype_JSON_Hex]
```

#### Version Header

| Byte Position | Length | Description |
| :--- | :--- | :--- |
| 0-1 | 2 | Protocol version identifier (`"20"` for HexDNA 2.0) |
| 2 | 1 | Block delimiter (`\|`) |
| 3+ | N | Hex-encoded JSON payload |

#### Genotype JSON Structure

```json
{
  "version": "2.0",
  "brain": {
    "topology": {
      "nodes": [
        {"id": 0, "layer": "input", "activation": "tanh"},
        {"id": 1, "layer": "input", "activation": "tanh"},
        ...
        {"id": 21, "layer": "hidden", "activation": "relu"},
        {"id": 22, "layer": "output", "activation": "sigmoid"}
      ],
      "connections": [
        {"src": 0, "dst": 21, "weight": 0.5, "enabled": true},
        {"src": 1, "dst": 21, "weight": -0.3, "enabled": true},
        ...
      ],
      "innovation_history": [
        {"innovation_id": 0, "parent_ids": [0, 1], "node_id": 21}
      ]
    },
    "weights": [0.5, -0.3, 0.1, ...],
    "biases": [0.0, 0.0, 0.1, ...]
  },
  "phenotype": {
    "sensing_range": 7.5,
    "max_speed": 1.8,
    "max_energy": 350.0,
    "metabolic_niche": 0.65,
    "trophic_potential": 0.3,
    "aggression": 0.4,
    "color": [128, 200, 80]
  },
  "life_history": {
    "reproductive_investment": 0.6,
    "maturity_gene": 1.2,
    "mate_preference": 0.45,
    "pairing_bias": 0.7
  },
  "metadata": {
    "lineage_id": "7f3a2c1e",
    "generation": 42,
    "created_at": 1699999999,
    "source_version": "2.0.0"
  }
}
```

### Serialization Process

The conversion from `Genotype` to HexDNA follows this pipeline:

```
Genotype Struct
    ↓ (serde_json::to_string)
JSON String
    ↓ (hex::encode)
Hexadecimal String
    ↓ (format!("20|{}"))
HexDNA 2.0 String
```

### Deserialization Process

Importing HexDNA reverses the pipeline:

```
HexDNA 2.0 String
    ↓ (parse header, extract payload)
Hexadecimal Payload
    ↓ (hex::decode)
JSON String
    ↓ (serde_json::from_str)
Genotype Struct
    ↓ (validation)
Ready for Spawn
```

### Validation Checksum

HexDNA 2.0 includes a CRC32 checksum for integrity verification:

```
[Version][Delimiter][Payload_Hex][Checksum]
```

The checksum is computed over the JSON payload before hex encoding.

---

## Genotype Serialization & Deserialization

### Export Operations

#### Console Export (`C` key)

When a user exports an entity's genome:

1. **Snapshot**: The entity's current `Genotype` is captured
2. **Serialize**: Convert to JSON via `serde_json`
3. **Encode**: Transform to hexadecimal representation
4. **Display**: Present in a modal overlay for copying
5. **File Option**: User may save to `exports/` directory

#### Batch Export

Multiple genotypes can be exported as a collection:

```json
{
  "collection_version": "1.0",
  "exported_at": 1699999999,
  "genomes": [
    {"hexdna": "20|7a4f...", "entity_id": 1234},
    {"hexdna": "20|b82c...", "entity_id": 1235}
  ]
}
```

### Import Operations

#### Console Import (`V` key)

Importing a HexDNA string:

1. **Parse**: Extract version header and payload
2. **Validate**: Verify protocol version compatibility
3. **Decode**: Convert hex to JSON string
4. **Deserialize**: Parse into `Genotype` struct
5. **Validate Genome**: Check all values within valid ranges
6. **Spawn**: Create new entity with imported genome

#### Error Handling

| Error Code | Condition | Recovery |
| :--- | :--- | :--- |
| `INVALID_VERSION` | Header version != "20" | Reject import |
| `CHECKSUM_MISMATCH` | CRC32 verification failed | Reject import |
| `MALFORMED_JSON` | JSON parse error | Reject import |
| `OUT_OF_RANGE` | Gene value outside valid bounds | Clamp or reject |
| `MISSING_FIELDS` | Required genome fields absent | Reject import |

---

## Genetic Migration Protocol

The migration protocol enables entities to move between simulation instances via the P2P network, preserving their complete genetic identity.

### Migration Flow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Source World   │     │   Relay Server  │     │  Target World   │
│                 │     │                 │     │                 │
│ 1. Snapshot     │────▶│ 2. Validate     │────▶│ 4. Deserialize  │
│    Genotype     │     │    HexDNA       │     │    Genotype     │
│                 │     │                 │     │                 │
│ 3. Transmit     │◀────│                 │◀────│ 5. Spawn        │
│    via WebSocket│     │                 │     │    Entity       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Step-by-Step Protocol

#### Phase 1: Source Snapshot

```rust
fn prepare_migration(entity: &Entity, world: &World) -> Result<HexDNA, MigrationError> {
    // Capture complete genotype
    let genotype = entity.genotype.clone();

    // Serialize to HexDNA 2.0
    let hexdna = Genotype::to_hexdna(&genotype)?;

    // Attach migration metadata
    let metadata = MigrationMetadata {
        source_id: world.instance_id,
        timestamp: world.current_tick(),
        lineage_id: genotype.lineage_id,
        fitness: entity.fitness_score(),
    };

    Ok(HexDNA::with_metadata(hexdna, metadata))
}
```

#### Phase 2: Network Transmission

The relay server handles migration requests:

```rust
async fn handle_migration_request(
    payload: MigrationPayload,
    server: &RelayServer,
) -> Result<(), MigrationError> {
    // Validate source instance is active
    server.validate_source(&payload.source_id)?;

    // Verify HexDNA checksum
    payload.hexdna.validate_checksum()?;

    // Broadcast to target instances
    server.broadcast_migration(payload).await?;

    Ok(())
}
```

#### Phase 3: Target Deserialization

```rust
fn import_migrant(hexdna: &str, world: &mut World) -> Result<Entity, MigrationError> {
    // Parse HexDNA 2.0 format
    let genotype = Genotype::from_hex(hexdna)?;

    // Validate compatibility with target world
    world.validate_genotype_compatibility(&genotype)?;

    // Create entity at migration point
    let entity = Entity::from_genotype(genotype, world.migration_entry_point);

    // Register in lineage system
    world.lineage_registry.register_migrant(&entity);

    Ok(entity)
}
```

### Compatibility Guarantees

HexDNA 2.0 ensures cross-version compatibility through:

1. **Forward Compatibility**: Unknown JSON fields are ignored during deserialization
2. **Backward Compatibility**: Default values used for missing fields
3. **Version Negotiation**: Source and target versions compared before migration
4. **Schema Evolution**: Optional fields marked in genotype structure

### Migration Constraints

| Constraint | Description |
| :--- | :--- |
| **Max Genotype Size** | 4KB per genome (enforced at export) |
| **Rate Limit** | 10 migrations per tick per instance |
| **Validation Timeout** | 500ms per migration request |
| **Lineage Preservation** | Original lineage_id maintained across migration |

---

## Genotype Evolution Mechanisms

### Mutation Operators

#### Point Mutation

Individual gene values shift by a small amount:

```rust
fn point_mutation(gene: f32, rate: f32, amount: f32) -> f32 {
    if random_f32() < rate {
        let direction = if random_bool() { 1.0 } else { -1.0 };
        (gene + direction * amount).clamp(GENE_MIN, GENE_MAX)
    } else {
        gene
    }
}
```

#### Topology Mutation (NEAT-lite)

The brain's neural network can evolve new structure:

| Mutation Type | Description | Probability |
| :--- | :--- | :--- |
| **Add Node** | Split existing connection, insert new hidden node | 0.02 |
| **Add Connection** | Connect two previously unconnected nodes | 0.05 |
| **Disable Connection** | Disable existing connection (weight = 0) | 0.01 |
| **Weight Perturbation** | Small random adjustment to existing weights | 0.1 |

#### Color Mutation

Phenotypic color evolves through RGB channel perturbation:

```rust
fn mutate_color(color: [u8; 3], amount: f32) -> [u8; 3] {
    color.map(|ch| {
        let delta = (random_f32() - 0.5) * 2.0 * amount * 255.0;
        (ch as f32 + delta).clamp(0.0, 255.0) as u8
    })
}
```

### Crossover (Sexual Reproduction)

When two entities mate, their genomes combine:

```rust
fn crossover(parent_a: &Genotype, parent_b: &Genotype) -> Genotype {
    // Uniform crossover for continuous genes
    let phenotype = if random_bool() {
        parent_a.phenotype.clone()
    } else {
        parent_b.phenotype.clone()
    };

    // Neural topology: average of innovation histories
    let brain = NEATCrossover::crossover(&parent_a.brain, &parent_b.brain);

    Genotype {
        brain,
        phenotype,
        life_history: parent_a.life_history.clone(), // Inherit from mother
        ..Default::default()
    }
}
```

### Epigenetic Inheritance

Environmental adaptations can influence offspring:

- **Stress Response**: Parents experiencing high stress may produce offspring with enhanced metabolic efficiency
- **Nutritional Memory**: Well-fed parents may produce offspring with higher initial energy reserves
- **Social Learning**: Offspring inherit social bond preferences from parents

---

## Version Migration & Backward Compatibility

### Schema Versioning

Each genotype carries a schema version for migration:

```rust
struct Genotype {
    schema_version: u32,
    // ... other fields
}
```

### Migration Strategies

| Source Version | Target Version | Strategy |
| :--- | :--- | :--- |
| 1.x | 2.0 | Full schema transformation |
| 2.0 | 2.0 | Direct pass-through |
| 2.0 | Future | Forward compatibility mode |

### Legacy Format Support

HexDNA 1.x formats are automatically upgraded:

```
1x|HexEncodedLegacyPayload
    ↓ (LegacyDeserializer)
Genotype (v2.0 format)
    ↓ (validate)
Ready for simulation
```

---

## Performance Considerations

### Serialization Benchmarks

| Operation | Time Complexity | Typical Duration |
| :--- | :--- | :--- |
| Serialize Genotype | O(N) where N = brain size | ~50μs |
| Hex Encode | O(2N) | ~30μs |
| Deserialize Genotype | O(N) | ~60μs |
| Hex Decode | O(2N) | ~40μs |

### Memory Optimization

- **Lazy Deserialization**: Brain topology loaded on-demand during migration
- **String Interning**: Repeated lineage_id values share memory
- **Pool Allocation**: Genotype objects allocated from object pool during batch imports
