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

Phase 38 introduces a dynamic environment that responds to biological activity through soil feedback and atmospheric changes. The world is no longer a static backdrop but an active participant in the evolutionary drama, where life reshapes the land and the land shapes the course of evolution.

#### Dynamic Biomes

Terrain cells are no longer static entities but living systems that evolve based on local ecological conditions. Each cell maintains a biome state that influences food production, entity metabolism, and carbon dynamics. The biome system creates emergent landscape patterns that reflect the history of life in each region.

##### Biome States

| Biome | Characteristics | Food Yield | Carbon Effect |
|-------|-----------------|------------|---------------|
| **Plains** | Baseline state, moderate fertility | 1.0x (baseline) | Neutral |
| **Forest** | High fertility, sustained biomass | 1.5x (enhanced) | Sequestration (-0.5/tick) |
| **Desert** | Depleted fertility, overgrazed | 0.3x (minimal) | Neutral (barren) |

##### Biome Transition Rules

Biome transitions occur when specific ecological thresholds are met over sustained periods. The system uses a hysteresis approach to prevent rapid oscillation between states.

**Plains → Forest Transition:**
- Requires **Soil Fertility > 0.7** sustained for 500+ consecutive ticks
- Requires **Average Plant Biomass > 50** in the cell region
- Transition probability: 5% per tick when conditions met

**Forest → Plains Regression:**
- Triggers when **Soil Fertility < 0.4** due to sustained grazing
- Triggers when **Plant Biomass < 20** for 200+ ticks
- Transition probability: 10% per tick when conditions met

**Plains → Desert Transition:**
- Triggers when **Soil Fertility < 0.15** from extreme overgrazing
- Requires **Cumulative Grazing Pressure > 200** (integrated over time)
- Transition probability: 3% per tick when conditions met

**Desert → Plains Recovery:**
- Requires **Soil Fertility > 0.25** (natural regeneration)
- Requires **Absence of Grazing Pressure** for 1000+ ticks
- Transition probability: 2% per tick when conditions met

##### Biome Influence on Entity Behavior

Entities perceive biome boundaries through their sensory systems and adjust their behavior accordingly:
- **Forest Attraction**: Herbivores gravitate toward forest edges for enhanced food yields
- **Desert Avoidance**: High metabolic stress in deserts discourages prolonged occupancy
- **Corridor Formation**: Migration paths naturally form along biome boundaries

#### Carbon Cycle

The ecosystem now tracks a global `carbon_level` variable that creates a feedback loop between organisms and the atmosphere. This atmospheric system connects all life through a shared resource that transcends individual lifespans and lineages.

##### Carbon Sources (Emissions)

Animals emit carbon as a metabolic byproduct, with emission rates scaling to their biological activity:

| Activity Type | Carbon Emission Rate | Trigger Condition |
|---------------|---------------------|-------------------|
| **Basal Metabolism** | 0.001 per tick | Every living entity |
| **Movement** | 0.005 per unit distance | When velocity > 0 |
| **Brain Activity** | 0.002 per hidden node | During neural inference |
| **Reproduction** | 0.05 per offspring | At birth event |
| **Aggression** | 0.01 per attack | During combat actions |

**Global Emission Formula:**
$$ Carbon_{emission} = \sum_{entities} (E_{base} \times M_{activity} \times 0.001) $$

##### Carbon Sinks (Sequestration)

Forest biomes act as carbon sinks, actively removing carbon from the atmosphere through simulated photosynthesis:

| Biome Type | Sequestration Rate | Mechanism |
|------------|-------------------|-----------|
| **Forest** | -0.5 per tick per cell | Active photosynthesis |
| **Plains** | -0.05 per tick per cell | Grassland absorption |
| **Desert** | 0.0 | No biomass for absorption |

**Sequestration Formula:**
$$ Carbon_{sequestration} = \sum_{cells} (Biomass_{density} \times BiomeFactor \times 0.01) $$

##### Global Carbon Balance

The net carbon change per tick is calculated as:
$$ \Delta Carbon = Carbon_{emission} - Carbon_{sequestration} $$

The system maintains `carbon_level` within bounds (0 to 2000), where:
- **Low Carbon (0-400)**: Ice Age conditions, reduced metabolic activity
- **Optimal Carbon (400-800)**: Temperate baseline conditions
- **Elevated Carbon (800-1200)**: Warming phase begins
- **Critical Carbon (>1200)**: Global warming cascade triggered

##### Global Warming Cascade

When `carbon_level` exceeds the critical threshold of 1200, the system initiates a warming cascade that affects the entire world:

**Phase 1: Warning (1200-1400)**
- Climate shifts from Temperate to Warm globally
- Metabolic multiplier increases from 1.0 to 1.5
- Food growth rates decrease by 20%

**Phase 2: Crisis (1400-1600)**
- Climate shifts from Warm to Hot globally
- Metabolic multiplier increases to 2.0
- Water sources begin to dry up (River effectiveness reduced)

**Phase 3: Catastrophe (>1600)**
- Climate shifts to Scorching globally
- Metabolic multiplier increases to 3.0
- Desertification accelerates globally
- Mass extinction threshold approached

**Recovery Mechanism:**
Carbon naturally decays at 0.1% per tick when emissions drop below sequestration. Additionally, catastrophic events (Mass Extinction, Dust Bowl) can artificially reduce carbon levels by 30-50%.

##### Carbon and Evolution

The carbon cycle creates evolutionary pressures that shape lineage trajectories:
- **High Carbon Eras**: Favor heat-tolerant phenotypes, reduced body size, efficient cooling
- **Low Carbon Eras**: Favor insulation, metabolic efficiency, cold-adapted traits
- **Carbon Fluctuation**: Creates boom-bust cycles that test evolutionary resilience

#### Biodiversity Hotspots

The system automatically detects and monitors "Hotspots"—grid regions with exceptionally high lineage density. These areas are critical for evolutionary innovation but are also more susceptible to rapid disease spread or resource exhaustion.

##### Hotspot Detection Algorithm

Hotspots are identified through a multi-scale density analysis:

**Step 1: Grid Partitioning**
The world is divided into 10x10 grid regions for analysis.

**Step 2: Lineage Density Calculation**
For each grid cell, calculate:
$$ Density_{lineage} = \frac{\sum_{entities} (1.0 - Distance_{normalized})}{CellArea} $$

**Step 3: Threshold Application**
A region qualifies as a hotspot when:
- **Lineage Count > 15** distinct lineages present
- **Lineage Diversity Index > 0.7** (Shannon entropy normalized)
- **Population Density > 0.5** (entities per unit area)

**Step 4: Hotspot Classification**

| Hotspot Type | Characteristics | Evolutionary Impact |
|--------------|-----------------|---------------------|
| **Radiation Zone** | High diversity, rapid adaptation | Accelerated trait evolution |
| **Refugium** | Stable population, low pressure | Conservation of ancestral traits |
| **Competitive Hub** | High density, intense selection | Arms race dynamics |

##### Hotspot Dynamics

**Emergence:**
Hotspots emerge naturally when favorable conditions converge:
- Resource abundance attracts diverse lineages
- Geographic features create natural boundaries
- Historical contingency shapes initial colonists

**Dissolution:**
Hotspots collapse when pressure exceeds carrying capacity:
- Resource depletion triggers mass emigration
- Disease outbreaks decimate populations
- Environmental changes alter habitat suitability

**Feedback Effects:**
Hotspots influence their surrounding environment:
- **Enhanced Mutation**: Hotspot proximity increases mutation rates by 1.5x
- **Disease Transmission**: Pathogens spread 3x faster within hotspots
- **Cultural Evolution**: Social behaviors emerge and spread rapidly

##### Monitoring and Visualization

The simulation tracks hotspot metrics in real-time:
- **Hotspot Count**: Number of active hotspots globally
- **Hotspot Stability**: Rate of hotspot turnover (emergence/dissolution)
- **Hotspot Diversity**: Average lineage diversity within hotspots

Users can observe hotspots through the TUI overlay, which highlights high-density regions with distinctive coloring.

#### Soil Feedback & Succession

The relationship between life and land is bidirectional, creating a continuous dialogue between organisms and their environment. Soil is not merely a substrate but a living system that records the history of biological activity.

##### Fertility Dynamics

**Depletion Mechanisms:**
- **Grazing Pressure**: Each food consumption reduces local fertility by 0.001
- **Erosion**: High-velocity movement across terrain increases erosion by 0.0005
- **Nutrient Export**: Entity migration carries nutrients away from birth regions

**Recovery Mechanisms:**
- **Corpse Fertilization**: Dead entities return 2% of max energy as fertility
- **Metabolic Excretion**: High-energy entities excrete nutrients (10% chance per tick)
- **Natural Regeneration**: Base fertility recovery of 0.0001 per tick
- **Biomass Presence**: Living plants contribute 0.0002 per tick

**Fertility Formula:**
$$ Fertility_{new} = Fertility_{old} + Recovery_{rate} - Depletion_{pressure} + Biomass_{contribution} $$

##### Succession Stages

The ecological succession follows a predictable trajectory:

**Stage 1: Pioneer (Desert/Barrens)**
- Fertility: 0.0-0.2
- Dominant life: Opportunistic R-strategists
- Food yield: Minimal
- Succession time: Variable

**Stage 2: Establishment (Plains)**
- Fertility: 0.2-0.5
- Dominant life: Mixed strategies
- Food yield: Baseline
- Succession time: 500-1000 ticks

**Stage 3: Development (Forest Transition)**
- Fertility: 0.5-0.7
- Dominant life: K-strategists emerging
- Food yield: 1.2x baseline
- Succession time: 1000-2000 ticks

**Stage 4: Climax (Mature Forest)**
- Fertility: 0.7-1.0
- Dominant life: Stable equilibrium
- Food yield: 1.5x baseline
- Succession time: Stable state

##### Human-Readable Succession Summary

| Stage | Fertility | Biomass | Food Multiplier | Typical Duration |
|-------|-----------|---------|-----------------|------------------|
| Pioneer | 0.0-0.2 | Sparse | 0.3x | Variable |
| Establishment | 0.2-0.5 | Moderate | 1.0x | 500-1000 ticks |
| Development | 0.5-0.7 | High | 1.2x | 1000-2000 ticks |
| Climax | 0.7-1.0 | Dense | 1.5x | Indefinite |

##### Anthropogenic Feedback

Entities actively shape their environment through:
- **Territorial Marking**: High-density areas develop "home field advantage"
- **Resource Concentration**: Successful lineages create positive feedback loops
- **Legacy Effects**: Ancestral success leaves marks in soil fertility patterns

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

### Advanced Social Hierarchy (Phase 49)

Tribes have evolved from egalitarian groups to stratified societies with leaders and specialized castes.

#### Social Rank
Every entity calculates a `rank` (0.0 to 1.0) based on four pillars of fitness:
- **Energy Reserves** (30%): Immediate survival capability.
- **Age Experience** (30%): Proof of survival skills (`age / 2000`).
- **Offspring Count** (10%): Evolutionary success.
- **Reputation** (30%): Social trust and altruism history.

#### Leadership Vectors
Rank dictates influence. Entities perceive the movement vector of the highest-ranking local tribe member (the "Alpha").
- **Alpha Influence**: Lower-ranking entities are drawn to follow the Alpha's path, creating organized movement without hard-coded flocking.

#### The Soldier Caste
A specialized phenotype emerges at the intersection of high rank and aggression.
- **Requirements**: `Rank > 0.8` AND `Aggression_Output > 0.5`.
- **Role**: Soldiers are the dedicated defenders/invaders of the tribe.
- **Combat Bonus**: Soldiers deal **1.5x damage**. In designated **War Zones**, this bonus increases to **2.0x**.

#### Tribal Splitting (The Fracture)
When varying success levels create tension in high-density areas, tribes can fracture.
- **Trigger**: An entity with **Low Rank (<0.2)** in a **High Density (>0.8)** environment.
- **Mechanism**: The entity initiates a "Schism", mutating its color significantly to founding a new, distinct tribe.
- **Effect**: Reduces local competition by breaking the "Same Tribe" protection pact, allowing the new tribe to fight for resources or migrate away.

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
