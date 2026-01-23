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

#### Social Coordination (Phase 30)

Phase 30 introduces advanced coordination mechanisms based on kin recognition and semantic signaling.

- **Kin Recognition**: Entities sense the relative center of mass of their lineage members (**KX**, **KY**). This allows for emergent swarming and collective migration behaviors.
- **Herding Bonus**: To encourage group cohesion, entities receive a **0.05 energy bonus** per tick when moving in the same direction as their kin centroid (calculated via dot product > 0.5 between movement vector and centroid vector).
- **Semantic Pheromones (Signal A/B)**: Beyond simple food trails, entities can now emit and sense abstract signals (**SA**, **SB**). These serve as a neural substrate for evolved coordination, allowing lineages to develop complex "languages" for danger, recruitment, or territory marking.

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
