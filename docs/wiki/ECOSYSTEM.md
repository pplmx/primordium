# Ecosystem Dynamics

The world of Primordium is governed by strict thermodynamic rules.

## Energy Formulas

### Metabolism (Cost of Living)

Every tick, an entity loses energy:

$$ E_{loss} = (E_{base} + (Speed \times C_{move}) + (BrainComplexity \times C_{think})) \times M_{env} $$

Where:

- $E_{base} = 0.5$ (Idle cost)
- $C_{move} = 1.0$ (Base movement cost; Terrain/Predation modifiers apply)
- $C_{think} = 0.1$
- $M_{env}$: Environmental multiplier (Night: 0.6, CPU-load: 1.0-3.0)

### Predation & Trophic Levels

When an entity kills another, it gains a portion of the target's energy:

- **Carnivore**: Gains **1.2x** of target energy (Predation specialist).
- **Herbivore**: Gains **0.2x** of target energy (Inefficient predator/cannibal).

### Social Interaction

#### Energy Sharing

Entities can altruistically share energy with tribe members:

- **Transfer**: 5% of giver's energy per tick.
- **Threshold**: Giver must have **> 70%** energy.
- **Requirements**: Distance < 2.0 units, same tribe (Manhattan RGB distance < 60).

#### Territoriality

Entities are more defensive near their birthplace:

- **Bonus**: **1.5x Aggression bonus** if within **8.0 units** of birth coordinates.

#### Social Defense

Lineage members provide mutual protection against predators:

- **Group Defense**: Damage received is reduced based on nearby lineage allies.
- **Multiplier**: Ranges from **1.0** (isolated) down to **0.4** (dense group).
- **Formula**: $M_{defense} = \max(0.4, 1.0 - (Allies \times 0.1))$

#### Dynamic Signaling

Entities can modulate their appearance for communication or camouflage:

- **Signal**: Real-time color modulation (Brighten/Warning vs Darken/Stealth).
- **Cost**: Actively signaling costs **0.1 energy per unit** of modulation intensity.

### World Eras

The simulation progresses through narrative eras based on global metrics:

| Era | Trigger |
| --- | ------- |
| **Genesis** | Initial state (Tick 0) |
| **Dawn of Life**| `tick > 5000` AND `avg_lifespan > 200` |
| **Flourishing** | `population > 200` AND `species_count > 3` |
| **Apex Era** | `top_fitness > 5000` |

### Photosynthesis (Food Growth)

Food spawns based on `SpatialHash` density checks.

- **Spring**: Growth Rate $\times 1.5$
- **Winter**: Growth Rate $\times 0.5$

### Pheromone Decay

Chemical trails dissipate exponentially:

$$ P_{new} = P_{old} \times (1.0 - DecayRate) $$

- **Default decay rate**: 0.5% per tick.
- **Cleanup threshold**: Strengths below 0.01 are reset to 0.0.

## Phenotypic Trade-offs

The evolutionary advantage of superior physical traits is balanced by increased metabolic and physical costs.

### Sensing Cost

Extended perception requires more neural processing and sensory maintenance.

- **Cost**: Every **+0.1** increase in Sensing Range adds **+2%** to the base Idle Cost.

### Movement Cost

High-speed locomotion is energy-intensive.

- **Cost**: Every **+0.1** increase in Max Speed adds **+5%** to the movement cost multiplier.

### Inertia & Responsiveness

Larger energy reserves increase the physical "mass" of the entity.

- **Effect**: Entities with higher **Max Energy** have higher inertia, reducing their acceleration and overall responsiveness to neural steering commands.
