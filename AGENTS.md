# Agent Project Memory: Primordium

> AI Agent 专用快速参考。详细信息请查阅对应文档。

## 📚 Documentation Index

| 需求 | 参考文档 |
|------|----------|
| 项目架构、目录结构、设计哲学 | [`ARCHITECTURE.md`](./ARCHITECTURE.md) |
| 神经网络拓扑、输入输出 | [`docs/wiki/BRAIN.md`](./docs/wiki/BRAIN.md) |
| 生态系统、能量公式、代谢 | [`docs/wiki/ECOSYSTEM.md`](./docs/wiki/ECOSYSTEM.md) |
| 历史、考古学、化石记录 | [`docs/wiki/HISTORY.md`](./docs/wiki/HISTORY.md) |
| HexDNA、遗传、变异逻辑 | [`docs/wiki/GENETICS.md`](./docs/wiki/GENETICS.md) |
| 用户手册、控制键位 | [`docs/MANUAL.md`](./docs/MANUAL.md) / [`docs/MANUAL_zh.md`](./docs/MANUAL_zh.md) |
| 项目概述、快速开始 | [`README.md`](./README.md) / [`docs/README_zh.md`](./docs/README_zh.md) |
| 版本变更记录 | [`CHANGELOG.md`](./CHANGELOG.md) |

---

## 🏗️ Quick Architecture Reference

> 详见 [`ARCHITECTURE.md`](./ARCHITECTURE.md)

```
src/
├── main.rs              # TUI 入口
├── lib.rs               # 库入口 (WASM 导出)
├── app/                 # TUI 应用层 (state, render, input, help, onboarding)
├── model/               # 模拟引擎核心
│   ├── state/           # 数据层 (entity, terrain, environment, food, pheromone, pathogen, lineage_registry)
│   ├── systems/         # 系统层 (intel, action, biological, social, ecological, environment, stats)
│   ├── infra/           # 基础设施 (blockchain, network, lineage_tree)
│   ├── brain.rs         # 神经网络 (20-6-8 RNN-lite / NEAT-lite)
│   ├── quadtree.rs      # 空间索引 (实为 SpatialHash)
│   ├── world.rs         # 协调器
│   ├── config.rs        # 配置
│   ├── history.rs       # 事件日志
│   └── migration.rs     # 实体迁移
├── ui/                  # 渲染抽象 (tui, web_renderer)
├── client/              # WASM 客户端 (wasm32 only)
├── server/              # P2P 中继服务器
└── bin/                 # 工具 (verify, analyze)
```

### Systems Execution Order

`World::update` 每 tick 执行顺序:

1. **Perception** (Rayon 并行) — 感知计算
2. **Intel** (Rayon 并行) — 神经网络推理
3. **Action** — 移动、边界
4. **Biological** — 代谢、死亡
5. **Social** — 捕食、繁殖、群体防御 (Group Defense)
6. **Ecological** — 食物生成
7. **Environment** — 时代、季节
8. **Stats** — 统计更新 (含 Lineage Stats)

---

## 🧬 Entity Architecture (Phase 38)

Entities follow a Component-Based (CBE) model with a unified **Genotype**.

### Structural Hierarchy

- `Entity`
    - `Physics`: Phenotype expression (sensing, speed).
    - `Metabolism`: Phenotype expression (energy capacity, carbon emission).
    - `Intel`: Decision center.
        - `Genotype`: The inheritable payload (encodes the DNA).
            - **Phenotypic Genes**: `sensing_range`, `max_speed`, `max_energy`, `metabolic_niche`.
            - **Life History Genes**: `reproductive_investment`, `maturity_gene`.
            - **Trophic Genes**: `trophic_potential` (0.0=Herbivore, 1.0=Carnivore).
            - **Neural Genes**: `Brain` (Dynamic Graph-based NEAT-lite).

### Environmental Succession (Phase 38)

- **Dynamic Biomes**: Terrain cells transition between Plains, Forest, and Desert based on `fertility` and `plant_biomass`.
- **Carbon Cycle**: Animals emit carbon; Forests sequestrate it. High `carbon_level` triggers global warming (shifting climate states).
- **Biodiversity Hotspots**: Automatic detection of grid regions with high lineage density.
- **Soil Feedback**: Overgrazing reduces fertility; biomass presence aids recovery (Succession).

### Resilience & Stasis (Phase 39)

- **Population-Aware Mutation**: Mutation scaling (0.5x to 3.0x) based on population density to balance exploration and exploitation.
- **Genetic Drift**: Stochastic trait randomization in bottlenecked populations (<10 entities).

### Archeology & History (Phase 40)

- **Fossil Record**: Persistent archival of extinct legendary genotypes in `logs/fossils.json`.
- **Snapshots**: Periodic macro-state capture (every 1,000 ticks) for history browsing.
- **TUI Archeology**: Navigate world history with `[` / `]` keys in the `Y` view.

### Massive Parallelism (Phase 41)

- **Rayon Integration**: Parallelized Perception and Intel/Action systems.
- **Command Proposals**: 3-pass update loop (Snapshot -> Parallel Proposals -> Sequential Apply).
- **Spatial Scaling**: Optimized for 10,000+ entities via row-partitioned Spatial Hash.

### Life History Strategies (Phase 32)

- **Investment**: `reproductive_investment` (0.1 to 0.9) defines the % of parent energy given to offspring.
- **Maturation**: `maturity_gene` (0.5 to 2.0) scales the time needed to reach adulthood and the `max_energy` ceiling.

### Brain Details

- **Architecture**: Dynamic graph-based topology. Initialized as 20 inputs (14 sensors + 6 memory) → 6 hidden → 8 outputs.
- **Topological Evolution**: Supports "Add Node" and "Add Connection" mutations with Innovation Tracking for crossover.
- **Memory**: The 6 initial hidden layer values from $T-1$ are fed back as inputs for $T$.
- **Metabolic Cost**: 0.02 per hidden node + 0.005 per enabled connection.

### Metabolic Niches (Phase 31)

- **Resource Specialization**: Entities evolve a `metabolic_niche` (0.0 to 1.0).
- **Nutrient Coupling**: Digestive efficiency is $1.0 - |niche - food\_type|$.
- **Terrain Strategy**: Blue food (1.0) in Mountains/Rivers; Green food (0.0) in Plains.

### Social Coordination (Phase 30)

- **Kin Recognition**: Sensing kin centroid (KX, KY).
- **Herding Bonus**: +0.05 energy/tick for alignment with kin centroid.
- **Semantic Signals**: SA/SB emission and sensing for evolved communication.

### Trophic Continuum (Phase 33)

- **Trophic Potential**: Sliding scale from 0.0 (Herbivore) to 1.0 (Carnivore).
- **Efficiency**: Plant gain ∝ $(1.0 - trophic\_potential)$; Meat gain ∝ $trophic\_potential$.
- **Trophic Cascade (Phase 35)**: Over-grazing and predator competition create self-regulating population cycles. Stability alerts (EcoAlert) notify of collapse.

### Action System Trade-offs

- **Sensing Radius**: +0.1 → +2% base idle cost.
- **Max Speed**: +0.1 → +5% movement cost.
- **Inertia**: $Acceleration \propto \frac{1}{MaxEnergy}$. High energy capacity reduces steering responsiveness.

### Social Hierarchy (Phase 49)

- **Rank Calculation**: Score = Energy(30%) + Age(30%) + Offspring(10%) + Reputation(30%).
- **Soldier Caste**: Requires Rank > 0.8 AND Aggression > 0.5. Bonus damage: 1.5x (flat), 2.0x (War Zone).
- **Tribal Splitting**: Triggered by high density (>0.8) and low rank (<0.2). Result: New color mutation.
- **Leadership Vector**: Calculated in `World::update` Pass 1. Influences movement in `Action` system.

---

## 🧪 Testing Strategy

- **Unit Tests**: `src/model/**/*.rs`
- **Integration Tests**: `tests/`

| 文件 | 覆盖范围 |
|------|----------|
| `lifecycle.rs` | 生命周期、繁殖 |
| `genetic_flow.rs` | HexDNA、Genetic Surge |
| `ecology.rs` | 土壤肥力、营养级 |
| `pathogens.rs` | 传染、免疫 |
| `disasters.rs` | Dust Bowl、碰撞 |
| `environment_coupling.rs` | 硬件耦合 (CPU→气候, RAM→资源) |
| `migration_network.rs` | 实体迁移、P2P |
| `persistence.rs` | 状态序列化 |
| `social_v2.rs` | 社会行为、防御、信号 |
| `lineage_persistence.rs` | 谱系注册、持久化、宏观指标 |
| `environmental_succession.rs` | 环境演替、碳循环、多样性热点 |
| `genetic_bottlenecks.rs` | 遗传瓶颈、动态突变、遗传漂变 |
| `archeology.rs` | 考古学工具、化石记录、快照 |
| `stress_test.rs` | 高负载基准 (1500+ 实体) |
| `world_evolution.rs` | 时代演进、昼夜节律 |
| `social_hierarchy.rs` | 社会等级、士兵阶层、部落分裂 |

---

## ⚓ Git Hooks

- **pre-commit**: `cargo test` + `cargo fmt --check` + `cargo clippy -D warnings`
- **pre-push**: Full test suite

---

## 📝 Maintenance Protocol

功能变更时 **必须同步更新**:

1. ✅ 测试用例
2. ✅ 中英文文档 (README, MANUAL, ARCHITECTURE 等)
3. ✅ 本文件 (如涉及 agent 关键信息)

---

## 💡 Gotchas & Lessons Learned

### Clippy 陷阱

```rust
// ❌ BAD - field_reassign_with_default
let mut x = X::default();
x.field = val;

// ✅ GOOD
let x = X { field: val, ..X::default() };
```

### 文件命名注意

- `quadtree.rs` 实际实现的是 **SpatialHash**,不是四叉树

### WASM 条件编译

- 多数模块受 `#[cfg(target_arch = "wasm32")]` 门控
- 调试时注意编译目标

### DNA 序列化

- `import_migrant` 需要通过 `Genotype::from_hex` 解析包含物理基因与神经网络的完整 HexDNA 字符串。

### 并行更新

- 使用 `EntitySnapshot` 模式避免可变借用冲突
- Buffer Pooling 减少分配抖动

### 灾害同步

- 地形灾害由 `World` 触发,在 `TerrainGrid` 更新中处理

### 神经网络 Fix
- Output Nodes 应为 20..28 (共8个)。之前版本曾有 Off-by-one 错误 (21..30)，已在 Phase 49 修复。

---

## 📦 Binary Targets

| Binary | Command | Purpose |
|--------|---------|---------|
| `primordium` | `cargo run --release` | TUI 模拟 (A: 谱系树, Shift+A: 导出 DOT) |
| `server` | `cargo run --bin server` | P2P 中继 (port 3000) |
| `verify` | `cargo run --bin verify` | 区块链验证 |
| `analyze` | `cargo run --bin analyze` | 历史分析 |

---

## 🛠️ Tooling

- **Search**: `rg` (ripgrep)
- **Find**: `fd` / `fdfind`
- **Avoid**: PowerShell 特定语法
