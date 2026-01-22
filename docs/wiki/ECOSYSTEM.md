# Ecosystem Dynamics

The world of Primordium is governed by strict thermodynamic rules.

## Energy Formulas

### Metabolism (Cost of Living)

Every tick, an entity loses energy:

$$ E_{loss} = E_{base} + (Speed \times C_{move}) + (BrainComplexity \times C_{think}) $$

Where:

- $E_{base} = 0.5$
- $C_{move} = 1.2$ (Terrain modifiers apply)
- $C_{think} = 0.1$

### Photosynthesis (Food Growth)

Food spawns based on `SpatialHash` density checks.

- **Spring**: Growth Rate $\times 1.5$
- **Winter**: Growth Rate $\times 0.5$

### Pheromone Decay

Chemical trails dissipate exponentially:

$$ P_{new} = P_{old} \times (1.0 - DecayRate) $$

Default decay rate is 0.5% per tick.
