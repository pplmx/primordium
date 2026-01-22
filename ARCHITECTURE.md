# ARCHITECTURE

## 概述

Primordium 是一个受神经驱动的数字生命模拟框架。拥有神经网络大脑、遗传系统和生存本能。本文档总结了我们实现这个系统时的架构决策、设计模式和经验教训。

---

## 核心架构

### 1. 实体（Entity）组件化结构

从 Phase 20 开始，`Entity` 结构体已重构为基于组件的设计，以提高数据局部性和系统的解耦性。

```rust
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub physics: Physics,        // 空间属性（x, y, vx, vy, home）
    pub metabolism: Metabolism,  // 能量与生命周期
    pub health: Health,          // 免疫力与感染状态
    pub intel: Intel,            // 神经网络大脑与决策状态
}
```

**组件定义：**
- **Physics**: 处理位置、速度、颜色、符号以及领地锚点 (`home_x/y`)。
- **Metabolism**: 处理 `energy`、`max_energy`、营养级角色 (`Herbivore/Carnivore`)、出生 tick、代数和后代计数。
- **Health**: 处理病原体感染、感染计时器和免疫力（0.0 到 1.0）。
- **Intel**: 封装神经网络 `Brain` 以及上一次更新的决策状态（`last_aggression`, `last_share_intent`）。

**设计经验：**
- **唯一标识**：使用 `Uuid` 而非简单的自增 ID，避免重启后 ID 冲突，同时支持谱系追踪。
- **谱系追踪**：`parent_id` 允许构建完整的家族树，这对分析进化路径至关重要。
- **组件化**：将相关数据分组，使得系统（Systems）可以只操作它们需要的组件，提高了代码的可维护性。

---

### 2. 神经网络大脑（Brain）

每个代理拥有一个 **Recurrent (RNN-lite)** 结构的神经网络：

```rust
pub struct Brain {
    pub weights_ih: Vec<f32>,  // 12 inputs -> 6 hidden
    pub weights_ho: Vec<f32>,  // 6 hidden -> 5 outputs
    pub bias_h: Vec<f32>,
    pub bias_o: Vec<f32>,
}
```

**输入层（12 个神经元）：**
1. **环境输入 (6)**：食物方向 X/Y、能量水平、邻近密度、信息素强度、部落识别。
2. **记忆输入 (6)**：上一个 tick 的隐藏层状态，允许跨 tick 的行为连贯性。

**输出层（5 个神经元）：**
1. **移动方向 X**
2. **移动方向 Y**
3. **爆发加速**（Boost）
4. **攻击性**（Aggression）
5. **能量分享**（Share）

---

### 3. 状态机：EntityStatus

```rust
pub enum EntityStatus {
    Starving,   // 能量 < 20%
    Infected,   // 携带病原体
    Juvenile,   // 幼体阶段 (◦)
    Mating,     // 能量 > 繁殖阈值
    Hunting,    // 攻击性 > 0.5
    Sharing,    // 分享能量中 (♣)
    Foraging,   // 默认状态
}
```

---

## 核心系统：系统化解耦 (Phase 20-21)

`World::update` 已被分解为多个独立的逻辑系统，支持高效维护和未来扩展。

### 1. 感知系统 (Perception System)
- **职责**: 收集环境数据并转换为神经输入。
- **优化**: 使用 `rayon` 进行并行感知计算，并引入了 `food_hash` 空间哈希以实现 $O(1)$ 的局部食物感知（半径 20.0）。

### 2. 决策系统 (Neural System)
- **职责**: 运行神经网络前向传播，处理循环神经状态。
- **并行化**: 通过 `.par_iter()` 同时处理所有实体的神经网络推理。

### 3. 动作与代谢系统 (Action & Metabolism System)
- **职责**: 将神经输出转换为物理位移，计算并扣除代谢成本。

### 4. 生态与动态地形系统 (Dynamic Terrain)
- **职责**: 管理土地肥力恢复与灾难触发。
- **灾难**: "Dust Bowl" (沙尘暴) 会在高温且种群密集时触发，使平原变为荒芜之地并持续损耗肥力。
- **物理障碍**: 引入了 `Wall` (墙壁/岩石) 类型，实体无法穿过并会发生物理反弹。

### 5. 生物系统 (Biological System)
- **职责**: 处理感染进度、免疫力演化、老化和自然死亡。

### 6. 社交与捕食系统 (Social & Trophic System)
- **职责**: 处理复杂的实体间交互，如跨部落捕食、能量分享和信息素排放。

---

## 性能扩展 (Scaling)

- **Rayon 并行化**: 模拟的核心循环（感知与决策）现已实现多线程。
- **空间哈希**: 拥有独立的 `spatial_hash` (实体) 和 `food_hash` (食物)，优化查询复杂度。
- **Buffer Pooling**: 在 `World` 结构体中重用内部缓冲区（如 `perception_buffer`, `decision_buffer`），显著降低了并行执行时的内存分配抖动。

---

## 演进方向：深度智能与全球化

### 1. 认知升级
- **多层递归**: 增加隐藏层深度或引入 LSTM 单元以实现长期记忆。
- **情绪系统**: 内部荷尔蒙状态影响神经权重偏置。

### 2. 全球宇宙 (Global Hive)
- **分布式计算**: 跨机器的代理迁移与交互。
- **共识演化**: 建立全球范围内的传奇基因库。
