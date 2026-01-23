# Neural Network Architecture

Primordium entities behave using a dynamic, graph-based neural network inspired by **NEAT (NeuroEvolution of Augmenting Topologies)**.

## From Matrix to Graph (Phase 28)

In earlier phases, entities used a fixed MLP (Multilayer Perceptron) architecture. Phase 28 introduces **NEAT-lite**, allowing the brain's topology to evolve over time alongside its weights. This enables the emergence of complex, specialized cognitive structures that are not limited by a fixed layer count or neuron density.

## Topology

The brain starts with a standard minimal configuration but grows dynamically:

- **Initial Input Layer**: 18 Neurons (12 Environment, 6 Recurrent)
- **Initial Hidden Layer**: 6 Neurons
- **Initial Output Layer**: 8 Neurons
- **Dynamic Growth**: Through mutations, new hidden nodes and connections can be added indefinitely.

## Inputs (Sensors)

### Environmental Inputs (0-11)

| ID | Sensor | Description |
| ---- | --------- | -------------- |
| 0 | `FoodDX` | X-distance to nearest food |
| 1 | `FoodDY` | Y-distance to nearest food |
| 2 | `Energy` | Internal energy % |
| 3 | `Neighbors`| Count of nearby entities |
| 4 | `Pheromone`| Strength of food trail at location |
| 5 | `Tribe` | Tribe density nearby |
| 6 | `KX` | Kin Centroid X (relative) |
| 7 | `KY` | Kin Centroid Y (relative) |
| 8 | `SA` | Signal A (semantic pheromone) |
| 9 | `SB` | Signal B (semantic pheromone) |
| 10 | `WL` | Wall Proximity |
| 11 | `AG` | Age/Maturity |

### Recurrent Inputs (12-17)

| ID | Sensor | Description |
| ---- | --------- | -------------- |
| 12-17| `Memory` | Output values of the initial 6 hidden nodes from previous tick (T-1) |

## Outputs (Actions)

| ID | Action | Threshold |
| ---- | --------- | ------------ |
| 0 | `MoveX` | Continuous (-1.0 to 1.0) |
| 1 | `MoveY` | Continuous (-1.0 to 1.0) |
| 2 | `Speed` | Continuous (Max speed modulation) |
| 3 | `Aggro` | > 0.5 invokes attack state |
| 4 | `Share` | > 0.5 transfers energy to tribe |
| 5 | `Color` | Real-time color modulation (-1.0 to 1.0) |
| 6 | `EmitSA` | > 0.5 emits Signal A |
| 7 | `EmitSB` | > 0.5 emits Signal B |

## Topological Mutations

Evolution now acts on the structure of the brain through two primary mechanisms:

1.  **Add Connection**: A new connection is created between two previously unconnected nodes.
2.  **Add Node**: An existing connection is split. The old connection is disabled, and two new connections are created leading in and out of the new hidden node.

## Innovation Tracking

To allow for successful crossover (sexual reproduction) between different topologies, every new structural mutation is assigned a global **Innovation Number**. During crossover, genes with matching innovation numbers are aligned, while disjoint/excess genes are inherited from the fitter parent, preventing the "competing conventions" problem.

## Metabolic Cost of Complexity

Intelligence is not free. To prevent "bloat" (unnecessary complexity that doesn't provide a survival advantage), every structural element carries a metabolic maintenance cost added to the base idle metabolism:

- **Per Hidden Node**: 0.02 energy / tick
- **Per Enabled Connection**: 0.005 energy / tick

This creates a natural selection pressure for efficiency, where only complexity that significantly improves survival remains in the gene pool.

## Activation Function

We use `Tanh` (Hyperbolic Tangent) for all nodes to allow negative values, mapping signals to `[-1.0, 1.0]`.
