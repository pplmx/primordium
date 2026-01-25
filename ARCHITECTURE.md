# Primordium 架构设计文档

本文档定义了 Primordium 项目的核心架构规范，作为重构和功能扩展的长期指导准则。

## 1. 设计哲学：Simulation as a Universe (引擎化)

我们将 `src/model` 视为一个高度内聚、自包含的**模拟宇宙引擎**。它包含了运行模拟所需的所有数据结构、业务逻辑以及模拟专用的外部设施。

* **对内内聚**：模拟的核心逻辑（如变异、捕食、区块链存证）全部收纳于 `model`。
* **对外解耦**：宿主环境（UI 渲染、App 生命周期、硬件监控）仅作为观察者，通过 API 与 `model` 交互。

## 2. 核心架构分层

整个模拟宇宙被划分为物理对等的三层：

### A. 数据层 (Model::State)

* **路径**: `src/model/state/`

* **规范**: 仅包含纯粹的名词（数据结构）。
* **原则**: **无谓词化**。State 内的方法仅限于 `new()`, `get_*()`, `is_*()`。任何会改变、演进数据的逻辑严禁进入此层。

### B. 系统层 (Model::Systems)

* **路径**: `src/model/systems/`

* **规范**: 包含所有演进逻辑（动词）。
* **原则**: 逻辑以纯函数或 Stateless 的过程存在。通过 `Context` 结构体访问 State。
    * **`ecological.rs`**: 处理食性级联（Trophic Cascades）与资源自平衡。
    * **`environment.rs`**: 环境演替、生物群落（Biomes）转换、碳循环及时代（Era）更替逻辑。
    * **`social.rs`**: 社交行为、捕食者-猎物动态、群体防御（Group Defense）、共生系统（Symbiosis）及跨物种杂交。
    * **`stats.rs`**: 宏观指标计算与多样性热点（Biodiversity Hotspots）探测。
    * **`intel.rs` / `biological.rs`**: 神经决策推理、代谢生命史策略及职业特化（Caste Specialization）逻辑。

### C. 基础设施层 (Model::Infra)

* **路径**: `src/model/infra/`

* **规范**: 模拟专用的外部 IO 或复杂协议。
* **原则**: 隔离模拟宇宙与外部系统的交互逻辑。
    * **`blockchain.rs`**: 基于 OpenTimestamps 的区块链锚定，为演化史提供不可篡改的证据。
    * **`network.rs`**: 定义跨宇宙迁移的 JSON 协议 (`NetMessage`) 及对等体元数据。
    * **`lineage_tree.rs`**: 基于 `petgraph` 的宏观演化分析引擎，记录谱系的分支与演化树。

### D. 核心引擎组件 (Core Engine Components)

这些模块位于 `src/model/` 根目录，作为支撑整个模拟运行的“物理常数”和“核心基座”：

* **`brain.rs` (Intel/Will)**: 模拟实体的神经网络模型及推理引擎。采用基于图结构的 NEAT-lite 架构，支持拓扑演化与时间连贯性。
* **`quadtree.rs` (Spatial Index)**: 高性能空间索引（Spatial Hash）。采用行分区（Row-partitioned）并行优化，为碰撞检测和感知提供物理加速。
* **`world.rs` (Coordinator)**: 整个模拟宇宙的“总线”。采用三阶段并行更新环（Snapshot -> Parallel Proposals -> Sequential Apply），通过 Rayon 实现大规模并行化。
* **`config.rs` (Constants)**: 模拟宇宙的物理规则参数。
* **`history.rs` (Archive)**: 模拟时空的观测记录，支持周期性状态快照与考古学回溯（Fossil Record）。
* **`migration.rs` (Engine Bus)**: 处理跨引擎（跨 Universe）的实体迁移协议，实现实体的序列化与反序列化。

## 4. 其它顶级组件

除了核心的 `model` 引擎外，项目还包含以下支撑层：

### A. 库与入口 (Library & Entry Points)

* **`src/lib.rs`**: **项目统一导出库**。它不仅是 Rust 库的入口，还包含了 WASM (WebAssembly) 的导出接口，定义了模拟引擎与 Web 环境的交互边界。

* **`src/main.rs`**: **原生应用入口**。负责 CLI 参数解析，并根据指令启动 TUI 环境、模拟屏保或 Headless 高速测试模式。

### B. 客户端层 (src/client/)

* **路径**: `src/client/`

* **职责**: **Web 特定逻辑**。收纳了仅在 WASM/浏览器环境下运行的逻辑，如 `NetworkManager`，通过 WebSocket 实现非阻塞的实体迁移通信。

### C. 模拟中继服务器 (src/server/)

* **路径**: `src/server/`

* **职责**: **多宇宙通信中心**。
    * **Relay Server**: 基于 **Axum** 和 **Tokio** 构建，维持多端的 WebSocket 长连接。
    * **Broadcasting**: 负责在不同模拟实例（Peer）之间广播 `MigrateEntity` 消息。
    * **Stats API**: 提供 REST 接口用于监控全球模拟状态。

### D. 实用工具集 (src/bin/)

* **路径**: `src/bin/`

* **职责**: **二进制工具链**。包含独立运行的辅助程序：
    * `analyze.rs`: 用于对模拟后的快照进行大规模数据分析。
    * `verify.rs`: 用于验证区块链上的锚定存证。

## 5. 目录与命名空间

```text
src/
├── main.rs          # 【应用入口】CLI/Native
├── lib.rs           # 【库入口】WASM/Library
├── client/          # 【Web 端】浏览器专用适配逻辑
├── bin/             # 【工具链】分析与验证辅助程序
├── model/           # 【核心】统一 Model 空间 (引擎实体)
│   ├── state/       # 【底层数据】entity, env, terrain, food...
│   ├── systems/     # 【动力系统】action, social, climate, stats...
│   ├── infra/       # 【通信/存证】blockchain, network...
│   ├── brain.rs     # 【核心：智能】
│   ├── quadtree.rs  # 【核心：空间】
│   ├── migration.rs # 【核心：入口】
│   ├── world.rs     # 【核心：协调】
│   ├── config.rs    # 【核心：规则】
│   └── history.rs   # 【核心：记录】
├── app/             # 【宿主】生命周期与事件循环
└── ui/              # 【表现】多端绘制逻辑
```

## 6. 命名与调用准则

1. **数据引用**: 使用 `crate::model::state::*`。
2. **系统调用**: 推荐在调用方使用别名以增强语义：

    ```rust
    use crate::model::systems::environment as environment_system;
    environment_system::update_era(&mut self.env, ...);
    ```

3. **循环依赖**: 通过将逻辑提升至顶层协调器（World）或抽离独立的 System 来消除 State 模块间的循环依赖。

## 7. 重构演进记录 (2026-01)

* **Phase 1-10**: 完成了从单体 `World` 向 `Systems` 的初步拆分。
* **Phase 11**: 目录结构引擎化重组，建立 `state`, `systems`, `infra` 三级架构。
* **Phase 12**: 模型去行为化完成。`Entity` 和 `Brain` 现为纯数据结构，所有行为逻辑已迁移至 `systems/`。
* **Phase 13-16**: 集成测试覆盖增强，添加 `social_dynamics`, `world_evolution`, `persistence`, `stress_test` 等验证模块。
* **Phase 17**: Cargo Workspace 剥离方案已设计，暂缓执行（当前单 crate 结构已满足需求）。
* **Phase 18**: 代码质量优化，移除 `unwrap()` 并添加核心 API 文档注释。
* **Phase 22**: **Parallel Evolution & Global Hive**。引入基于 Axum 的中继服务器与分布式迁移协议，实现跨主机的实体共享与演化协同。
* **Phase 23**: **Phenotypic Specialization**。将物理属性（速度、感知、能量容量）整合入 `Genotype`。实现了物理特征的可遗传、可变异，并配套建立了基于物理极限的代谢权衡（Trade-offs）算法。
* **Phase 24**: **Lineage & Macroevolution**。引入了谱系追踪（Lineage Tracking）机制。通过在 `Genotype` 中整合 `lineage_id`，实现了对远古祖先后代的长期追踪。该系统支持跨宇宙谱系保存，并为 TUI 提供了宏观演化（Macroevolution）的可视化数据支持，标志着从个体演化向族群演进分析的跨越。
* **Phase 28**: **Complex Brain Evolution (NEAT-lite)**。将神经网络从固定矩阵的 MLP 升级为动态图结构的 NEAT-lite 架构。支持节点与连接的拓扑变异，并通过创新追踪 (Innovation Tracking) 确保复杂的交叉遗传。引入了基于大脑复杂度的代谢惩罚（Metabolic Penalty）以防止无效冗余。
* **Phase 31**: **Metabolic Niches & Resource Diversity**。引入了多色食物系统（Green/Blue），并配套演化出 `metabolic_niche` 基因。生物现在必须权衡其消化效率（0.2x 到 1.2x），导致种群在不同地形（平原 vs 山脉）产生明显的生态位分化。
* **Phase 32**: **Life History Strategies (R/K Selection)**。实现了生活史策略的遗传。通过 `reproductive_investment` 和 `maturity_gene` 基因，生物可以演化出“多而弱”的 R 策略或“少而精”的 K 策略。建立了发育缩放逻辑（Body size ∝ Maturity），模拟了生长周期与体型的生物学平衡。
* **Phase 32.5**: **Hardening & Quality Lockdown**。完成了全量集成测试修复与引擎硬化。引入了 `ActionContext` 设计模式，确保了 19x8 复杂神经架构下的逻辑稳定性，并验证了在极端代谢压力与网络损坏 DNA 情况下的生存边界。
* **Phase 34**: **The Tree of Life (Ancestry Visualization)**。引入了基于 `petgraph` 的谱系树 analysis 系统 (`lineage_tree.rs`)。实现了对种群宏观演化路径的实时追踪，支持在 TUI 中可视化优势王朝的演化分支，并提供 Graphviz/DOT 格式导出功能。
* **Phase 35**: **Trophic Cascades**。引入了自调节的食性级联（Trophic Cascade）机制。实现了捕食者-猎物种群数量的自动调节循环，并配套了生态稳定性告警系统 (EcoAlert)，用于监测生态系统的崩溃风险。
* **Phase 38**: **Environmental Succession**。引入了环境演替机制。支持动态生物群落（平原、森林、沙漠）的自动转换，建立了全球碳循环模型（碳排放 vs 碳汇），并实现了多样性热点（Biodiversity Hotspots）的自动探测。
* **Phase 39**: **Resilience & Stasis**。强化了种群韧性逻辑。在极小种群（<10 个体）中引入遗传漂变（Genetic Drift），并实现了基于种群密度的动态突变率缩放（Population-aware Mutation Scaling），平衡演化的探索与利用。
* **Phase 40**: **Archeology & Deep History**。建立了考古学与深层历史记录系统。支持持久化的化石档案 (`logs/fossils.json`) 以记录绝灭的传奇谱系，并引入了周期性的宏观状态快照，允许用户在 TUI 中回溯世界演化历史。
* **Phase 41**: **Massive Parallelism**。实现了大规模并行化仿真。基于 Rayon 构建了三阶段更新环（快照 -> 并行提案 -> 顺序应用），并优化了行分区（Row-partitioned）的空间哈希算法，支持万级实体的高性能模拟。
* **Phase 42**: **Macro-Environmental Pressures**。引入了宏观环境压力系统。根据不同时代（Era）自动调整选择压力（如代谢率、突变率缩放），由生物量、碳水平和多样性等宏观指标驱动时代的更替。
* **Phase 49**: **Advanced Social Hierarchies**。实现了高级社会等级系统。引入了 **Social Rank**（社会等级）计算逻辑以及 **Soldier**（士兵）阶层。实现了基于密度的 **Tribal Splitting**（部落分裂）机制，当低等级个体处于拥挤环境时会自发分裂出新的敌对部落。修复了神经网络输出层的寻址偏移问题。
