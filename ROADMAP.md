# Primordium (原初之境) - Development Roadmap

> *Evolution in silicon, powered by your machine*

A hardware-coupled artificial life simulation where digital organisms evolve neural networks in your terminal, with their world shaped by your computer's real-time performance.

---

## 🎯 Project Vision

Primordium is not just a screensaver—it's a **living laboratory** where:

- CPU temperature becomes environmental climate
- RAM pressure controls resource scarcity
- Neural networks emerge through natural selection
- Every legendary organism's DNA is preserved on blockchain
- Your machine becomes a god, and you become the observer

---

## 📦 Technology Stack

```toml
[dependencies]
# Core rendering
ratatui = "0.26"
crossterm = "0.27"

# System monitoring
sysinfo = "0.30"

# Simulation
rand = "0.8"

# Data persistence
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

# Blockchain (Phase 5.5+)
sha2 = "0.10"
hex = "0.4"
ethers = "2.0"  # Optional for NFT minting
```

---

## 🗺️ Development Phases

### Phase 1: Genesis - Physics Foundation

**Timeline:** 2-3 days
**Goal:** Build the terminal universe and basic physics

#### Core Features

- Initialize Ratatui TUI framework with crossterm backend
- Implement World grid system (100x50 default, configurable)
- Create Entity system with position and velocity vectors
- Basic physics: random walk with momentum
- Boundary collision detection (wrap-around or bounce)
- 60 FPS rendering loop with smooth updates

#### Data Structures

```rust
struct Entity {
    id: Uuid,
    x: f64,
    y: f64,
    vx: f64,  // velocity x
    vy: f64,  // velocity y
    color: Color,
    symbol: char,
}

struct World {
    width: u16,
    height: u16,
    entities: Vec<Entity>,
    tick: u64,
    config: WorldConfig,
}
```

#### Deliverables

- ✅ 50-100 colored entities moving randomly
- ✅ Stable 60 FPS performance
- ✅ Keyboard controls: Q to quit, Space to pause
- ✅ Basic debug overlay showing FPS and entity count

---

### Phase 2: The Breath of Life - Metabolism & Evolution

**Timeline:** 3-4 days
**Goal:** Introduce life, death, and heredity

#### 2.1 Energy System

- Add `energy` attribute to entities (initial: 100.0)
- Energy costs:
  - Moving: 1.0 per step
  - Idle: 0.5 per tick
  - Accelerating: 2.0 per step
- Death condition: energy ≤ 0

#### 2.2 Food Chain

- Spawn green food particles `*` randomly
- Maintain food population: 20-50 items dynamically
- Collision detection: entity overlaps food → +50 energy
- Food respawn rate: 0.5 items/second base rate

#### 2.3 Reproduction

- Trigger: energy > 150
- Mechanism:
  - Parent energy split 50/50 with offspring
  - Child spawns at adjacent cell
  - Inherit parent's velocity ± 10% mutation
  - Inherit color with ±15 RGB mutation
- Generation tracking: `generation` field increments

#### 2.4 Natural Selection Observables

- Population boom-bust cycles
- Color clustering (beneficial mutations spread)
- Speed optimization (too fast = starve, too slow = can't compete)

#### Data Extensions

```rust
struct Entity {
    // ... previous fields
    energy: f64,
    max_energy: f64,
    speed: f64,
    generation: u32,
    birth_tick: u64,
    parent_id: Option<Uuid>,
}

struct Food {
    x: u16,
    y: u16,
}
```

#### Deliverables

- ✅ Observable population oscillations
- ✅ Mass extinction events when food scarce
- ✅ Multi-generational lineages (Gen 10+)
- ✅ Statistics panel: population, births, deaths

---

### Phase 3: Hardware Resonance - Environmental Coupling

**Timeline:** 1-2 days
**Goal:** Bridge virtual and physical worlds

#### 3.1 System Monitoring

- Poll system stats every 1 second using `sysinfo`
- Metrics collected:
  - CPU usage percentage (global)
  - RAM usage percentage
  - System load average (1min)

#### 3.2 Climate Mapping

**Temperature System (CPU → Metabolism)**

```
CPU Usage     Climate State    Metabolism Multiplier
---------     -------------    ---------------------
0-30%         🌡️ Temperate     ×1.0 (baseline)
30-60%        🔥 Warm          ×1.5
60-80%        🌋 Hot           ×2.0
80-100%       ☀️ Scorching     ×3.0 (mass die-off)
```

**Resource Pressure (RAM → Food Scarcity)**

```
RAM Usage     Resource State   Food Spawn Rate
---------     --------------   ---------------
0-50%         🌾 Abundant      ×1.0
50-70%        ⚠️ Strained      ×0.7
70-85%        🚨 Scarce        ×0.4
85-100%       💀 Famine        ×0.1
```

#### 3.3 Visualization

- Top status bar (3 lines):
  - Line 1: `CPU: [▓▓▓▓▓░░░░░] 52% | Climate: 🔥 Warm`
  - Line 2: `RAM: [▓▓▓▓▓▓▓░░░] 73% | Resources: ⚠️ Strained`
  - Line 3: `Pop: 47 | Gen: 23 | Temp: ×1.5 | Food: ×0.7`
- Historical CPU graph (mini sparkline, last 60 seconds)

#### 3.4 Environmental Events

- **Heat Wave:** CPU spike > 80% for 10+ seconds → halve food spawn
- **Abundance:** RAM drops below 40% → 2× food for 30 seconds
- **Ice Age:** CPU < 10% for 60 seconds → all metabolism ×0.5

#### Deliverables

- ✅ Launch heavy program → observe population crash
- ✅ Close applications → population recovery
- ✅ Real-time correlation visible in stats
- ✅ Climate state changes reflected in entity behavior

---

### Phase 4: Neural Awakening - Intelligent Behavior

**Timeline:** 4-5 days
**Goal:** Replace random walk with learned behavior

#### 4.1 Sensory Inputs (4 dimensions)

```rust
struct Sensors {
    dx_to_food: f32,      // X distance to nearest food (-1.0 to 1.0)
    dy_to_food: f32,      // Y distance to nearest food (-1.0 to 1.0)
    energy_ratio: f32,    // current_energy / max_energy
    crowding: f32,        // neighbors within 5-cell radius / 10
}
```

#### 4.2 Neural Network Architecture

```
Input Layer [4] → Hidden Layer [6] → Output Layer [3]

Weights:
  - input_to_hidden: 4×6 = 24 floats
  - hidden_to_output: 6×3 = 18 floats
  Total DNA: 42 genes
```

**Activation:** Tanh for hidden, Tanh for output

**Output Interpretation:**

- `output[0]`: Move intention X (-1.0 = left, +1.0 = right)
- `output[1]`: Move intention Y (-1.0 = up, +1.0 = down)
- `output[2]`: Speed boost (0 = normal, 1 = 2× speed, 2× energy cost)

#### 4.3 Evolution Strategy

- **Initialization:** Random weights in [-1.0, 1.0]
- **Inheritance:** Child copies parent's brain exactly
- **Mutation:**
  - Each weight has 10% chance to mutate
  - Mutation: add random value from [-0.2, 0.2]
  - Rare large mutations: 1% chance of ±0.5 (genetic drift)

#### 4.4 Fitness Landscape

- Organisms with better brains eat more → more energy → more offspring
- Their genes dominate the population
- Inefficient brains (spinning, wall-hitting) die out

#### Implementation

```rust
struct Brain {
    weights_ih: [f32; 24],  // input to hidden
    weights_ho: [f32; 18],  // hidden to output
    bias_h: [f32; 6],
    bias_o: [f32; 3],
}

impl Brain {
    fn forward(&self, inputs: [f32; 4]) -> [f32; 3] {
        // Matrix multiply + tanh activation
        let mut hidden = [0.0; 6];
        for i in 0..6 {
            let mut sum = self.bias_h[i];
            for j in 0..4 {
                sum += inputs[j] * self.weights_ih[j * 6 + i];
            }
            hidden[i] = sum.tanh();
        }

        let mut output = [0.0; 3];
        for i in 0..3 {
            let mut sum = self.bias_o[i];
            for j in 0..6 {
                sum += hidden[j] * self.weights_ho[j * 3 + i];
            }
            output[i] = sum.tanh();
        }
        output
    }

    fn mutate(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for w in &mut self.weights_ih {
            if rng.gen::<f32>() < 0.1 {
                *w += rng.gen_range(-0.2..0.2);
                *w = w.clamp(-2.0, 2.0);
            }
        }
        // Same for weights_ho, biases...
    }
}
```

#### Deliverables

- ✅ By generation 100: directed movement toward food
- ✅ By generation 500: emergent behaviors (circling, ambushing)
- ✅ Average lifespan increases 3-5× vs Phase 2
- ✅ Brain visualization mode (show weight heatmap for selected entity)

---

### Phase 5: The Ledger - Historical Archives

**Timeline:** 2-3 days
**Goal:** Preserve evolutionary history for analysis

#### 5.1 Identity System

- Assign `Uuid` to every entity at birth
- Track lineage: `parent_id`, `generation`
- Record timestamp: `birth_tick`, `death_tick`

#### 5.2 Legend Criteria

An entity becomes **Legendary** if any of:

- Lifespan > 1000 ticks (~16 minutes @ 60fps)
- Offspring count > 10
- Peak energy > 300
- Survived a major environmental event

#### 5.3 Data Persistence

**Live Event Stream** (`logs/live.jsonl`)

```json
{"event":"birth","id":"a1b2c3d4","parent":"e5f6g7h8","gen":12,"tick":5432}
{"event":"death","id":"a1b2c3d4","age":1523,"offspring":8,"tick":6955}
{"event":"extinction","population":0,"tick":7890}
{"event":"climate_shift","from":"Temperate","to":"Hot","tick":8000}
```

**Legends Archive** (`logs/legends.json`)

```json
{
  "id": "a1b2c3d4-5678-90ef-ghij-klmnopqrstuv",
  "birth_tick": 5432,
  "death_tick": 6955,
  "lifespan": 1523,
  "generation": 12,
  "offspring_count": 8,
  "max_energy": 287.5,
  "birth_timestamp": "2026-01-20T14:32:11Z",
  "cause_of_death": "starvation",
  "brain_dna": {
    "weights_ih": [0.23, -0.45, 0.67, ...],
    "weights_ho": [0.12, 0.89, ...],
    "fitness_score": 0.87
  },
  "lineage": ["root_id", "parent_id", "grandparent_id"]
}
```

#### 5.4 Analysis Tools

Create separate binary `primordium-analyze`:

```bash
cargo run --bin analyze -- --input logs/legends.json --output report.md
```

**Features:**

- Parse `legends.json` and `live.jsonl`
- Build family tree using `petgraph` crate
- Detect "Mitochondrial Eve" (most recent common ancestor)
- Generate statistics:
  - Average lifespan by generation
  - Brain weight distribution heatmap
  - Extinction event timeline
- Export family tree as DOT graph for Graphviz

**Output Example:**

```markdown
# Primordium Evolution Report

## Summary
- Total simulation time: 47,392 ticks (13h 9m)
- Total births: 2,847
- Legendary organisms: 23
- Longest lineage: 89 generations

## Genetic Bottleneck
Generation 34: Population collapsed to 3 individuals.
All current organisms descend from Entity `f8a9b2c3`.

## Top Legends
1. "The Immortal" (a1b2c3d4) - 2,547 ticks, 18 offspring
2. "The Founder" (e5f6g7h8) - 892 ticks, 31 offspring
...
```

#### Deliverables

- ✅ 24-hour run generates complete historical record
- ✅ Analyzer outputs Markdown report + family tree PNG
- ✅ Can "resurrect" legendary brain by loading DNA
- ✅ Time-lapse replay mode: watch evolution in fast-forward

---

### Phase 5.5: Blockchain Anchoring - Immutable Proof

**Timeline:** 1-2 days
**Goal:** Cryptographically prove evolutionary history

#### 5.5.1 Hash Timestamping (Minimal Implementation)

**Process:**

1. Every hour, serialize all legendary organisms to JSON
2. Compute SHA-256 hash of the data
3. Submit hash to blockchain

**Options:**

- **Free:** OpenTimestamps (Bitcoin-anchored, no cost)
- **Cheap:** Polygon PoS (~$0.01 per tx)
- **Fast:** Base L2 (~$0.05 per tx)

```rust
use sha2::{Sha256, Digest};

async fn anchor_legends(legends: &[Legend]) -> Result<String> {
    let json = serde_json::to_string(legends)?;
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    // Option 1: OpenTimestamps
    opentimestamps::stamp(&hash_hex).await?;

    // Option 2: Polygon transaction
    let tx = ethereum_client
        .send_transaction(hash_hex)
        .await?;

    Ok(tx.hash)
}
```

**Verification:**

- User downloads `legends.json`
- Computes local hash
- Queries blockchain for matching hash
- If match: proves data hasn't been tampered with

#### 5.5.2 NFT Minting (Optional Extension)

**Trigger:** Entity becomes legendary

**Metadata Generation:**

```json
{
  "name": "Primordium Legend #47",
  "description": "Generation 47 organism, survived 2547 ticks across 3 climate shifts. Offspring: 18",
  "image": "data:image/svg+xml;base64,...",
  "attributes": [
    {"trait_type": "Generation", "value": 47},
    {"trait_type": "Lifespan", "value": 2547},
    {"trait_type": "Climate Resistance", "value": "High"},
    {"trait_type": "Brain Efficiency", "value": 0.87}
  ],
  "dna": "0x23a4b6c8..." // Hex-encoded brain weights
}
```

**Smart Contract (ERC-721):**

```solidity
contract PrimordiumLegends {
    struct Legend {
        uint256 generation;
        uint256 lifespan;
        bytes dna; // Neural network weights
    }

    mapping(uint256 => Legend) public legends;

    function mint(address to, Legend memory legend) public {
        // Mint NFT with embedded DNA
    }
}
```

**Use Cases:**

- Trade legendary genes on OpenSea
- Cross-pollinate: import NFT DNA into your simulation
- Competitive leaderboards: whose simulation breeds the best?

#### Deliverables

- ✅ Hourly blockchain anchoring (zero user intervention)
- ✅ Web UI to verify any legend's authenticity
- ✅ (Optional) NFT gallery showing all legendary organisms
- ✅ Cost under $5/month for continuous anchoring

---

### Phase 6: Immersion - Polish & Deployment

**Timeline:** 2-3 days
**Goal:** Production-ready experience

#### 6.1 Operating Modes

**Standard Mode** (`primordium`)

- Full UI: statistics, graphs, controls
- Interactive controls:
  - `Space`: Pause/Resume
  - `+/-`: Adjust time speed (0.5× to 4×)
  - `R`: Reset world
  - `S`: Save snapshot
  - `L`: Load snapshot
  - `B`: Toggle brain visualization
  - `H`: Show help overlay

**Screensaver Mode** (`primordium --screensaver`)

- Minimal UI: only organisms and food
- No borders, no text
- Lower FPS (30fps) for energy efficiency
- Auto-pause after 1 hour of inactivity

**Headless Mode** (`primordium --headless`)

- No rendering at all
- Pure simulation + logging
- For server-side experiments
- Output: JSON stats every 60 seconds

**Analysis Mode** (`primordium --replay logs/history.jsonl`)

- Load historical data
- Time-travel through evolution
- Scrub timeline, filter events

#### 6.2 Performance Optimizations

**Spatial Partitioning:**

- Implement QuadTree for entity collision detection
- O(n²) → O(n log n) for food detection
- Critical when population > 200

**Rendering Optimization:**

- Frustum culling: only render visible entities
- Batch rendering: group entities by color
- Dirty rectangle tracking: only redraw changed regions

**Memory Management:**

- Object pooling for entities (avoid alloc/dealloc churn)
- Circular buffer for event logs (limit to last 10,000 events)

#### 6.3 Configuration System

**`config.toml`:**

```toml
[world]
width = 200
height = 100
initial_population = 100
initial_food = 30

[simulation]
target_fps = 60
time_scale = 1.0

[metabolism]
base_move_cost = 1.0
base_idle_cost = 0.5
reproduction_threshold = 150
energy_split_ratio = 0.5

[evolution]
mutation_rate = 0.1
mutation_magnitude = 0.2
large_mutation_chance = 0.01

[environment]
cpu_temp_multipliers = [1.0, 1.5, 2.0, 3.0]
ram_food_multipliers = [1.0, 0.7, 0.4, 0.1]

[logging]
enable_live_log = true
enable_legends = true
legend_lifespan_threshold = 1000
legend_offspring_threshold = 10

[blockchain]
enable_anchoring = false
anchor_interval_seconds = 3600
network = "polygon"  # or "opentimestamps"
```

#### 6.4 Distribution

**Build Release:**

```bash
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-msvc
```

**Installation Methods:**

1. **Cargo:** `cargo install primordium`
2. **Homebrew (macOS):** `brew install primordium`
3. **Binaries:** GitHub Releases with precompiled executables
4. **Docker:** `docker run -it primordium/primordium`

**Shell Integration:**

```bash
# .zshrc / .bashrc
alias life='primordium --screensaver'

# Launch on terminal idle (macOS)
echo 'primordium --screensaver' > ~/.config/iterm2/screensaver.sh
```

#### Deliverables

- ✅ Release build uses < 5% CPU (screensaver mode)
- ✅ Handles terminal resize gracefully
- ✅ No memory leaks after 7-day run
- ✅ Cross-platform builds for Linux/macOS/Windows
- ✅ Published to crates.io

---

## 🚀 Extended Roadmap (Phase 7+)

### Phase 7.1: Predator-Prey Dynamics

**New Entity Type:** Red predators that hunt blue herbivores

- Predators gain energy by "eating" herbivores
- Herbivores eat green food
- Three-tier food chain creates stable oscillations
- Co-evolution: prey evolve evasion, predators evolve pursuit

### Phase 7.2: Social Behavior

**Emergent Cooperation:**

- Pheromone system: entities leave chemical trails
- Food sharing: high-energy entities can donate to neighbors
- Territorial behavior: aggressive entities drive others away
- Tribe formation: color-based group identity

### Phase 7.3: Terrain & Geography

**Environmental Heterogeneity:**

- Mountains: slow movement, high food
- Rivers: fast movement, no food
- Oases: food concentration points
- Migration patterns emerge naturally

### Phase 7.4: Advanced Neural Networks

**Upgrade to Deep Learning:**

- Replace hand-coded NN with `tch-rs` (PyTorch bindings)
- Recurrent layers for memory (LSTM cells)
- Vision system: 5×5 pixel "retina"
- Export trained brains to ONNX for visualization

### Phase 7.5: Decentralized Simulation

**Multi-Node Network:**

- Run multiple Primordium instances
- Entities can migrate between nodes (via HTTP API)
- Genetic exchange: trade DNA between simulations
- Global leaderboard (blockchain-verified)

### Phase 7.6: WebAssembly Port

**Browser Version:**

- Compile to WASM with `wasm-pack`
- Canvas-based rendering (no terminal)
- Share simulations via URL
- Embedded in blog posts / documentation

### Phase 7.7: Creative Extensions

**Art & Music:**

- Export time-lapse videos with `ffmpeg`
- Generate ambient music: population → melody, energy → rhythm
- VR visualization with `bevy` game engine
- NFT entire simulation states (not just organisms)

---

## 📊 Development Timeline

| Phase | Duration | Cumulative | Key Milestone |
|-------|----------|------------|---------------|
| Phase 1 | 2-3 days | 3 days | First moving pixels |
| Phase 2 | 3-4 days | 7 days | Natural selection visible |
| Phase 3 | 1-2 days | 9 days | Hardware coupling working |
| Phase 4 | 4-5 days | 14 days | Intelligent behavior emerges |
| Phase 5 | 2-3 days | 17 days | Historical records complete |
| Phase 5.5 | 1-2 days | 19 days | Blockchain integration |
| Phase 6 | 2-3 days | 21 days | Production release |

**Total Core Development:** 3 weeks (4-6 hours/day)

---

## 🎯 Success Metrics

### Technical

- ✅ Stable 60 FPS with 500+ entities
- ✅ Memory usage < 50MB
- ✅ Zero crashes in 72-hour stress test
- ✅ Cross-platform builds pass CI

### Scientific

- ✅ Observable evolution: Gen 1 vs Gen 100 show clear fitness improvement
- ✅ Hardware correlation: R² > 0.7 between CPU and metabolism
- ✅ Reproducible results: same config → similar outcomes
- ✅ Lineage tracking: can trace any organism to common ancestor

### Community

- ✅ 100+ GitHub stars in first month
- ✅ 10+ community-submitted brain DNAs
- ✅ Featured on Hacker News / Reddit r/rust
- ✅ Academic paper citation (artificial life conference)

---

## 🛠️ Quick Start Commands

```bash
# Initialize project
cargo new primordium
cd primordium

# Add dependencies (copy from tech stack section)
# ... edit Cargo.toml

# Phase 1 development
cargo run

# Phase 6 release
cargo build --release
./target/release/primordium --screensaver

# Analysis
cargo run --bin analyze -- --input logs/legends.json

# Deploy
cargo publish
```

---

## 📚 Resources

**Core Concepts:**

- [Ratatui Book](https://ratatui.rs)
- [Artificial Life Primer](https://mitpress.mit.edu/books/introduction-artificial-life)
- [Evolutionary Neural Networks](https://www.sciencedirect.com/topics/computer-science/neuroevolution)

**Inspiration:**

- Conway's Game of Life
- The Bibites (YouTube simulation)
- Avida (digital evolution platform)
- Tierra (self-replicating programs)

**Community:**

- Discord: (to be created)
- GitHub Discussions
- r/rust, r/ArtificialLife

---

## 🌱 Philosophy

Primordium is an experiment in **emergent complexity**. You provide the rules, the hardware provides the pressure, and evolution writes the story.

Every run is unique. Every lineage is precious. Every extinction teaches us something.

Welcome to the primordial soup. Let there be life.

---

*Last updated: 2026-01-20*
*Version: 1.0.0-alpha*
