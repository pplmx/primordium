# Neural Network Architecture

Primordium entities behave using a feed-forward neural network.

## Topology

- **Input Layer**: 12 Neurons (6 Environment, 6 Recurrent)
- **Hidden Layer**: 6 Neurons
- **Output Layer**: 5 Neurons

## Inputs (Sensors)

### Environmental Inputs (0-5)

| ID | Sensor | Description |
| ---- | --------- | -------------- |
| 0 | `FoodDX` | X-distance to nearest food |
| 1 | `FoodDY` | Y-distance to nearest food |
| 2 | `Energy` | Internal energy % |
| 3 | `Neighbors` | Count of nearby entities |
| 4 | `Pheromone` | Strength of food trail at location |
| 5 | `Tribe` | Tribe density nearby |

### Recurrent Inputs (6-11)

| ID | Sensor | Description |
| ---- | --------- | -------------- |
| 6-11| `Memory` | Hidden layer state from previous tick (T-1) |

## Outputs (Actions)

| ID | Action | Threshold |
| ---- | --------- | ------------ |
| 0 | `MoveX` | Continuous (-1.0 to 1.0) |
| 1 | `MoveY` | Continuous (-1.0 to 1.0) |
| 2 | `Boost` | > 0.5 triggers speed boost |
| 3 | `Aggro` | > 0.0 invokes attack state |
| 4 | `Share` | > 0.5 transfers energy to tribe |

## Activation Function

We use `Tanh` (Hyperbolic Tangent) for hidden layers to allow negative values, mapping inputs to `[-1.0, 1.0]`.
