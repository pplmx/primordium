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

每个代理拥有一个 **6-6-5** 结构的神经网络：

```rust
pub struct Brain {
    pub weights_ih: Vec<f32>,  // 6 inputs -> 6 hidden
    pub weights_ho: Vec<f32>,  // 6 hidden -> 5 outputs
    pub bias_h: Vec<f32>,
    pub bias_o: Vec<f32>,
}
```

**输入层（6 个神经元）：**
1. **食物方向 X**：最近食物的归一化 X 向量（-1 到 1）
2. **食物方向 Y**：最近食物的归一化 Y 向量（-1 到 1）
3. **能量水平**：当前能量 / 最大能量（0 到 1）
4. **邻近密度**：附近代理数量 / 10（0 到 1）
5. **信息素强度**：当前位置的化学信号浓度
6. **部落识别**：周围是否有同部落成员（-1 到 1）

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

## 核心系统：系统化解耦 (Phase 20)

`World::update` 已被分解为多个独立的逻辑系统，支持高效维护和未来扩展。

### 1. 感知系统 (Perception System)
- **职责**: 收集环境数据并转换为神经输入。
- **优化**: 使用 `rayon` 进行并行感知计算。

### 2. 决策系统 (Neural System)
- **职责**: 运行神经网络前向传播。
- **并行化**: 通过 `.par_iter()` 同时处理所有实体的神经网络推理。

### 3. 动作与代谢系统 (Action & Metabolism System)
- **职责**: 将神经输出转换为物理位移，计算并扣除代谢成本。

### 4. 生物系统 (Biological System)
- **职责**: 处理感染进度、免疫力演化、老化和自然死亡。

### 5. 社交与捕食系统 (Social & Trophic System)
- **职责**: 处理复杂的实体间交互，如跨部落捕食、能量分享和信息素排放。

---

## 环境系统

### 1. 病原体与免疫系统 (Pathogen System)
- **传播机制**: 感染者通过空间哈希查询邻近实体，根据概率和免疫力进行传染。
- **免疫演化**: 实体在感染痊愈后会获得免疫力提升，且会遗传给后代。

### 2. 昼夜循环 (Circadian Rhythms)
- **光照影响**: 植物生长概率与当前光照强度 (`light_level`) 成正比。
- **代谢波动**: 实体在夜间进入低代谢模式，静息能量消耗降低 40%。

### 3. 地形与季节
- **地形类型**: 平原、山脉（阻碍）、河流（加速）、绿洲（丰富食物）。
- **季节循环**: 春、夏、秋、冬影响食物增长率和代谢速率。

---

## 性能扩展 (Scaling)

- **Rayon 并行化**: 模拟的核心循环（感知与决策）现已实现多线程。
- **空间哈希**: 优化实体查询复杂度从 $O(N^2)$ 到 $O(N \log N)$。
- **Snapshots 机制**: 使用实体快照解决并行更新时的借用冲突。

---

## 演进方向：动态与灾难

### 1. 环境流动性 (Environmental Fluidity)
- **动态地形**: 河流干涸、 barren land 上的火灾蔓延。
- **自然灾害**: 陨石撞击、洪水、磁暴。

### 2. 认知升级
- **记忆神经元**: 支持跨 tick 的行为连贯性。
- **递归思维**: 循环神经网络 (RNN) 结构的引入。
