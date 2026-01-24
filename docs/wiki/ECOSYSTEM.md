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
- $M_{env}$: Environmental multiplier. 
    - **Circadian**: Day=1.0, Night=0.6.
    - **Climate**: Temperate=1.0, Warm=1.5, Hot=2.0, Scorching=3.0.
    - **Era Pressure**: Primordial=1.0, DawnOfLife=0.9, Flourishing=1.1, DominanceWar=1.5, ApexEra=1.2.
    - **Hardware Coupling**: Linked to CPU load (1.0-3.0).

### Nutrient Cycling & Niche Construction (Phase 44)

The ecosystem is now a closed loop where life actively constructs its own niche.

#### Corpse Fertilization
Death is not an exit, but a return. When an entity dies (hunger, predation, or old age), it returns a percentage of its stored biomass to the soil:
- **Formula**: Soil Fertility $+= (\frac{MaxEnergy}{100} \times 0.02)$.
- **Impact**: High-death zones (battlefields or famine sites) become fertile hotspots, accelerating plant recovery.

#### Metabolic Feedback (Excretion)
High-energy entities (Energy > 70%) moving across the world have a **10% chance** per tick to "excrete" nutrients.
- **Effect**: Increases local Soil Fertility by **0.01**.
- **Impact**: Population centers naturally "farm" their surroundings, encouraging localized food blooms.

### Trophic Continuum (Phase 33)

Phase 33 replaces binary "Herbivore vs. Carnivore" roles with a continuous genetic spectrum defined by the `trophic_potential` (TP) gene (0.0 to 1.0).

#### Efficiency Scaling

An entity's ability to extract energy from plants (environment) vs. meat (other entities) scales linearly with its `trophic_potential`:

- **Plant Gain**: $E_{gain} = FoodValue \times (1.0 - TP)$
- **Meat Gain**: $E_{gain} = VictimEnergy \times TP$

*Note: Pure herbivores (TP=0.0) gain maximum energy from food clusters but zero from predation. Pure carnivores (TP=1.0) must hunt to survive, as environmental food provides no energy.*

### Ecosystem Stability (Phase 35)

Phase 35 introduces feedback loops to prevent runaway population growth and ensure long-term stability.

#### Over-grazing
High herbivore biomass puts pressure on the terrain. If the total energy consumed by herbivores in a region exceeds the soil's regeneration capacity, the **Soil Fertility** drops, reducing future food growth rates until the population thins.

#### Hunter Competition
Predatory efficiency is no longer static. As the density of predators increases, the energy gain per kill scales inversely with predator biomass. This simulates competition for the best cuts and the energy spent defending kills from rivals.

#### EcoAlerts
The system monitors for "Trophic Collapse" scenarios:
- **Trophic Collapse**: Sudden extinction of all predators leading to herbivore overpopulation and subsequent famine.
- **Overgrazing Alert**: Sustained soil depletion that threatens the entire local food chain.

### Environmental Succession & Carbon Cycle (Phase 38)

Phase 38 introduces a dynamic environment that responds to biological activity through soil feedback and atmospheric changes.

#### Dynamic Biomes
Terrain cells are no longer static. They transition between different states based on local ecological conditions:
- **Plains**: The baseline state.
- **Forest**: Emerges in areas with high **soil fertility** and sustained **plant biomass**. Forests provide higher food yields and act as carbon sinks.
- **Desert**: Result of extreme **overgrazing** and fertility depletion. Deserts have minimal food growth and high metabolic stress.

#### Carbon Cycle
The ecosystem now tracks a global `carbon_level`, creating a feedback loop between organisms and the atmosphere:
- **Emissions**: Animals emit carbon as a byproduct of metabolism.
- **Sequestration**: Forest biomes actively sequestrate (absorb) carbon from the atmosphere.
- **Global Warming**: If `carbon_level` exceeds a critical threshold, it triggers global warming, shifting the world into hotter climate states (Warm -> Hot -> Scorching), increasing metabolic costs for all entities.

#### Biodiversity Hotspots
The system automatically detects and monitors "Hotspots"—grid regions with exceptionally high **lineage density**. These areas are critical for evolutionary innovation but are also more susceptible to rapid disease spread or resource exhaustion.

#### Soil Feedback & Succession
The relationship between life and land is bidirectional:
- **Depletion**: Overgrazing reduces soil fertility.
- **Recovery**: The presence of biomass and the absence of grazing pressure allow fertility to recover over time, enabling the land to transition back from Desert to Plains, and eventually to Forest.

### Social Interaction & Game Theory (Phase 46)

#### Inclusive Fitness (Hamilton's Rule)

Social behaviors are now governed by the **Coefficient of Relatedness ($r$)**, where $r = 2^{-dist \times 0.5}$.

- **Energy Sharing**: Entities share energy only if $r > 0.25$.
- **Formula**: Shared Amount $= Energy \times 0.05 \times r$.
- **Requirement**: Giver must be > 70% full.

#### Territoriality

Entities are more defensive near their birthplace:

- **Bonus**: **1.5x Aggression bonus** if within **8.0 units** of birth coordinates.

#### Social Defense (Phase 46)

Lineage protection is no longer binary. It scales with the sum of relatedness in the vicinity.

- **Group Defense**: $M_{defense} = \max(0.4, 1.0 - (\sum r \times 0.15))$
- **Impact**: Closely related clusters are almost immune to external predation, while "outsiders" with no kin are vulnerable.

#### Reputation & Punishment (Phase 46)

The ecosystem enforces cooperation through a **Reputation System** (0.0 to 1.0).

- **Betrayal**: Attacking an entity with $r > 0.5$ (kin) reduces reputation by **0.3**.
- **Exploitation**: Low reputation (< 0.5) entities lose tribe protection and can be hunted by their own kin.
- **Altruism**: Sharing energy increases reputation proportionally to $r$.
- **Recovery**: Reputation slowly recovers towards 1.0 over time (0.001 per tick).

#### Social Grid & Zones (Phase 46)

The simulation space can be partitioned into **Social Zones**:

- **Peace Zone (Blue)**: Predation is strictly forbidden. Favors specialization and metabolic efficiency.
- **War Zone (Red)**: Attack power is doubled (2x). Allows intra-lineage predation regardless of reputation.

#### Social Coordination (Phase 30)

Phase 30 introduces advanced coordination mechanisms based on kin recognition and semantic signaling.

- **Kin Recognition**: Entities sense the relative center of mass of their lineage members (**KX**, **KY**). This allows for emergent swarming and collective migration behaviors.
- **Herding Bonus**: To encourage group cohesion, entities receive a **0.05 energy bonus** per tick when moving in the same direction as their kin centroid (calculated via dot product > 0.5 between movement vector and centroid vector).
- **Semantic Pheromones (Signal A/B)**: Beyond simple food trails, entities can now emit and sense abstract signals (**SA**, **SB**). These serve as a neural substrate for evolved coordination, allowing lineages to develop complex "languages" for danger, recruitment, or territory marking.

#### Dynamic Signaling

Entities can modulate their appearance for communication or camouflage:

- **Signal**: Real-time color modulation (Brighten/Warning vs Darken/Stealth).
- **Cost**: Actively signaling costs **0.1 energy per unit** of modulation intensity.

### World Eras (Phase 42)

The simulation progresses through narrative eras triggered by macro-ecological metrics rather than simple time:

| Era | Primary Trigger | Effect |
| --- | ------- | ------ |
| **Primordial** | Initial state (Tick 0) | Chaos adaptation, high mutation. |
| **Dawn of Life**| `AvgLifespan > 200` OR `HerbivoreBiomass > 2000` | Stable population emerging. |
| **Flourishing** | `Hotspots >= 2` AND `Population > 150` | High diversity, adaptive radiation. |
| **Dominance War** | `CO2 > 800` OR `PredatorBiomass % > 30%` | Resource scarcity, metabolic stress (1.5x). |
| **Apex Era** | `TopFitness > 8000` | Peak evolution reached, stability focus. |

### Photosynthesis (Food Growth)

Food spawns based on `SpatialHash` density checks.

- **Spring**: Growth Rate $\times 1.5$
- **Winter**: Growth Rate $\times 0.5$

### Pheromone Decay

Chemical trails dissipate exponentially:

$$ P_{new} = P_{old} \times (1.0 - DecayRate) $$

- **Default decay rate**: 0.5% per tick.
- **Cleanup threshold**: Strengths below 0.01 are reset to 0.0.

## Metabolic Niches (Phase 31)

Phase 31 introduces **Resource Diversity**, forcing lineages to specialize in specific nutrient types to maximize digestive efficiency.

### Resource Diversity
Food items are no longer generic. Each food source has a `nutrient_type` ranging from **0.0 (Green)** to **1.0 (Blue)**.

### Terrain-Nutrient Coupling
The environment influences the type of food that spawns:
- **Mountains & Rivers**: High mineral/hydration areas favor **Blue** nutrient types (1.0).
- **Plains & Oases**: Organic-rich lowlands favor **Green** nutrient types (0.0).

### Digestive Efficiency
An entity's ability to extract energy from food depends on the alignment between its genetic `metabolic_niche` and the food's `nutrient_type`.

$$ Efficiency = 1.0 - |genotype.metabolic\_niche - food.nutrient\_type| $$

### Energy Gain Scaling
- **Specialist Match**: A perfect match (e.g., niche 0.0 eating type 0.0) yields a **1.2x** energy bonus.
- **Mismatch Penalty**: A total mismatch (e.g., niche 1.0 eating type 0.0) yields only **0.2x** energy, leading to starvation despite eating.

This selection pressure drives lineages to migrate toward terrain that matches their metabolic specialization.

## Phenotypic Trade-offs

The evolutionary advantage of superior physical traits is balanced by increased metabolic and physical costs.

### Sensing Cost

Extended perception requires more neural processing and sensory maintenance.

- **Cost**: Every **+0.1** increase in Sensing Range adds **+2%** to the base Idle Cost.

### Movement Cost

High-speed locomotion is energy-intensive.

- **Cost**: Every **+0.1** increase in Max Speed adds **+5%** to the movement cost multiplier.

## Reproductive Strategies (Phase 32)

Phase 32 introduces **R/K Selection Theory** into the evolutionary engine, allowing lineages to adapt their life history to environmental stability.

### Strategy R (Opportunists)
Evolved for unstable or unpredictable environments where rapid population growth is key.
- **Traits**: Low maturity age, low parental investment (`reproductive_investment`).
- **Advantage**: High turnover rate; can quickly colonize empty habitats after disasters (e.g., Dust Bowl).

### Strategy K (Specialists)
Evolved for stable, competitive environments where individual quality outweighs quantity.
- **Traits**: High maturity age, high parental investment (`reproductive_investment`).
- **Advantage**: Offspring start with massive energy reserves, making them highly resilient to initial competition or famine.

### Developmental Momentum
Large specialists take significantly longer to reach adulthood due to the `maturity_gene` multiplier. However, their coupled **Max Energy** capacity allows them to survive long periods of scarcity that would wipe out Strategy R populations.

### Inertia & Responsiveness

Larger energy reserves increase the physical "mass" of the entity.

- **Effect**: Entities with higher **Max Energy** have higher inertia, reducing their acceleration and overall responsiveness to neural steering commands.

## Macroevolution & Ancestry (Phase 34)

Phase 34 shifts the analytical focus from individual survival to the structural evolution of the entire ecosystem. By utilizing `petgraph`, the engine now constructs a real-time directed acyclic graph (DAG) representing the branching history of all lineages.

### Lineage Branching
As entities mutate and form distinct clusters, the system identifies significant genetic divergence and creates new nodes in the "Tree of Life." This allows users to trace any dominant organism back to its primordial ancestor.

### Ancestry Visualization (The 'A' View)
The TUI now includes a dedicated **Ancestry** tab (toggled via the `A` key). 
- **Top 5 Dynasties**: Visualizes the most successful evolutionary branches currently active in the simulation.
- **Trophic Overlay**: Colors nodes based on their dominant metabolic strategy (Herbivore vs. Carnivore).
- **DOT Export**: Pressing `Shift+A` exports the current evolutionary tree in Graphviz/DOT format for external high-resolution analysis.
