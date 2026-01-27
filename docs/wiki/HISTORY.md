# History & Archeology (Phase 40)

Phase 40 introduces the **Archeology Tool**, **Fossil Record**, and **History Snapshots**, enabling users to preserve and analyze the long-term evolutionary trajectory of their world.

## Archeology Tool

The Archeology Tool allows users to travel back in time and observe the macro-evolutionary state of the world.

- **Toggle View**: Press the `y` key to open the Archeology & Fossil Record panel.
- **Time Travel**: Use the `[` (back) and `]` (forward) keys to navigate through recorded history snapshots.
- **Data Display**: Each snapshot shows the population count, species count, atmospheric carbon levels, and identified biodiversity hotspots at that specific point in time.

## Fossil Record

The **Fossil Record** is a persistent registry of the most successful lineages that have gone extinct. It ensures that evolutionary innovations are not lost even after a species vanishes from the living world.

### Fossilization Process
When the last member of a lineage dies, the system triggers the **Fossilization** process:
1.  **Legendary Archiving**: Throughout its life, each lineage tracks its "Best Legendary" representative—the individual with the highest fitness score (calculated based on age, offspring count, and peak energy).
2.  **Extraction**: Upon extinction, this legendary representative's genotype, including its complete neural brain architecture, is extracted.
3.  **Fossilization**: A `Fossil` record is created, capturing the lineage's peak stats (Max Generation, Total Offspring produced, Peak Population) and its brain DNA.
4.  **Persistence**: Fossils are stored in `logs/fossils.json` and remain accessible across simulation runs.

### Viewing Fossils
The Archeology panel (`y`) displays the top 10 most "interesting" fossils, ranked by the total number of offspring the lineage produced. This allows users to pay homage to the great dynasties of the past.

## History Snapshots

To enable deep history browsing, the engine periodically captures the entire macro-state of the world.

### Snapshot Mechanics
- **Frequency**: A world snapshot is taken every **1,000 ticks**.
- **Contents**: Each snapshot captures `PopulationStats`, which includes:
    - Global Population and Species Count.
    - Average Lifespan and Brain Entropy.
    - Biomass distribution (Herbivore vs. Carnivore).
    - Atmospheric Carbon levels and Mutation Scaling factors.
    - Count of entities per active lineage.
    - Location of biodiversity hotspots.
- **Logging**: Snapshots are streamed to `logs/live.jsonl` as `Snapshot` events, providing an immutable record of the world's progress.

## Civilizational History (Phase 63)

Simulation history now records the rise of civilizational structures.

- **Outpost Timeline**: Snapshots track the density and ownership of Outposts (`Ψ`), mapping the expansion of digital territories.
- **Power Grid Formation**: History captures when lineages successfully link remote outposts via canal networks, marking the transition from decentralized tribes to integrated civilizations.
- **Climate Legacy**: Long-term atmospheric graphs show how dominant lineages used forest management near outposts to reverse global warming, leaving a "Planetary Fingerprint" of their reign.

The history system is built around several key structures:
- `Fossil`: The data structure representing an extinct lineage's legacy.
- `FossilRegistry`: Manages the collection of fossils and handles I/O.
- `LiveEvent::Snapshot`: The event type used for periodic state capture.
- `PopulationStats`: The comprehensive metric set captured in each snapshot.

By combining real-time event logging with periodic snapshots and persistent fossilization, Primordium creates a rich, navigable history that turns every simulation run into a unique saga of life, death, and digital evolution.
