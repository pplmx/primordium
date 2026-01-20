# ARCHITECTURE

## 概述

Primordium 是一个受神经驱动的数字生命模拟框架。拥有神经网络大脑、遗传系统和生存本能。本文档总结了我们实现这个系统时的架构决策、设计模式和经验教训。

---

## 核心架构

### 1. 实体（Entity）结构

```rust
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
    pub energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub generation: u32,
    pub birth_tick: u64,
    pub offspring_count: u32,
    pub brain: Brain,
    pub last_aggression: f32,
}
```

**设计经验：**
- **唯一标识**：使用 `Uuid` 而非简单的自增 ID，避免重启后 ID 冲突，同时支持谱系追踪
- **谱系追踪**：`parent_id` 允许构建完整的家族树，这对分析进化路径至关重要
- **视觉多样性**：RGB 颜色 + 符号编码，每个代理都有独特的视觉特征
- **能量管理**：使用 `energy`、`max_energy`、`peak_energy` 三层能量模型：
  - `energy`：当前能量，用于生存决策
  - `max_energy`：最大能量，定义能量上限
  - `peak_energy`：历史峰值，用于传奇判定（hall of fame）

---

### 2. 神经网络大脑（Brain）

每个代理拥有一个 4-6-4 结构的神经网络：

```rust
pub struct Brain {
    pub weights_ih: [f32; 24],  // 4 inputs -> 6 hidden
    pub weights_ho: [f32; 24],  // 6 hidden -> 4 outputs
    pub bias_h: [f32; 6],
    pub bias_o: [f32; 4],
}
```

**输入层（4 个神经元）：**
1. **食物方向 X**：最近食物的归一化 X 向量（-1 到 1）
2. **食物方向 Y**：最近食物的归一化 Y 向量（-1 到 1）
3. **能量水平**：当前能量 / 最大能量（0 到 1）
4. **邻近密度**：附近代理数量 / 10（0 到 1）

**输出层（4 个神经元）：**
1. **移动方向 X**：-1（左）到 1（右）
2. **移动方向 Y**：-1（上）到 1（下）
3. **速度**：-1（慢）到 1（快）
4. **攻击性**：-1（被动）到 1（攻击）

**激活函数：**
- 使用 `tanh` 而非 `sigmoid`，原因：
  - 输出范围 [-1, 1]，更适合方向控制
  - 中心在 0，权重初始化更稳定
  - 计算简单，性能优秀

**设计经验：**
- **轻量级网络**：58 个权重 + 10 个偏置 = 68 个参数，足以产生复杂行为，同时保持高性能
- **输入归一化**：所有输入都归一化到 [-1, 1]，避免训练不稳定
- **固定架构**：不使用动态层数或神经元数，保持所有代理的计算一致性

---

### 3. 状态机：EntityStatus

```rust
pub enum EntityStatus {
    Starving,   // 能量 < 20%
    Mating,     // 能量 > 繁殖阈值
    Hunting,    // 攻击性 > 0.5
    Foraging,   // 默认状态
}
```

**视觉编码：**
- `†`（Starving）：深红色 (150, 50, 50) - 危险状态
- `♥`（Mating）：粉色 (255, 105, 180) - 繁殖信号
- `♦`（Hunting）：红橙色 (255, 69, 0) - 攻击状态
- `●`（Foraging）：自身颜色 - 正常行为

**状态判定优先级：**
```rust
pub fn status(&self, reproduction_threshold: f64) -> EntityStatus {
    if self.energy / self.max_energy < 0.2 {
        EntityStatus::Starving        // 最高优先级：生存
    } else if self.last_aggression > 0.5 {
        EntityStatus::Hunting         // 第二优先级：狩猎
    } else if self.energy > reproduction_threshold {
        EntityStatus::Mating          // 第三优先级：繁殖
    } else {
        EntityStatus::Foraging        // 默认：觅食
    }
}
```

**设计经验：**
- **状态层级**：优先级明确，避免状态冲突（如同时想觅食和攻击）
- **视觉反馈**：用户可以一眼识别每个代理的当前目标
- **动态切换**：状态每 tick 重新计算，反应环境变化

---

## 进化系统

### 1. 突变（Mutation）

**双重突变机制：**
```rust
pub fn mutate_with_config(&mut self, config: &EvolutionConfig) {
    let mut mutate_val = |v: &mut f32| {
        let r = rng.gen::<f32>();
        if r < config.drift_rate {
            *v += rng.gen_range(-config.drift_amount..config.drift_amount);  // 小幅度漂移
        } else if r < config.mutation_rate {
            *v += rng.gen_range(-config.mutation_amount..config.mutation_amount);  // 大幅度突变
        }
        *v = v.clamp(-2.0, 2.0);  // 限制权重范围
    };
}
```

**参数：**
- `drift_rate`：基因漂移概率（如 0.3）- 小幅度的随机变化
- `drift_amount`：漂移幅度（如 0.01）- 微调现有行为
- `mutation_rate`：突变概率（如 0.1）- 大幅度的行为改变
- `mutation_amount`：突变幅度（如 0.5）- 探索新的行为模式
- 权重范围：`[-2.0, 2.0]` - 防止权重爆炸

**设计经验：**
- **探索与利用的平衡**：
  - `drift`：利用 - 微调现有优良基因
  - `mutation`：探索 - 跳出局部最优
- **权重裁剪**：`clamp(-2.0, 2.0)` 防止权重无限增长
- **随机性**：每个权重独立判断突变，保持基因多样性

### 2. 交叉（Crossover）

```rust
pub fn crossover(parent1: &Brain, parent2: &Brain) -> Self {
    let mut child = parent1.clone();
    for i in 0..child.weights_ih.len() {
        if rng.gen_bool(0.5) {
            child.weights_ih[i] = parent2.weights_ih[i];
        }
    }
    // ... 对所有权重和偏置执行相同操作
    child
}
```

**设计经验：**
- **均匀交叉（Uniform Crossover）**：每个基因位有 50% 概率来自任一父本
- **无算术交叉**：不使用 `(p1 + p2) / 2`，保持离散基因多样性
- **独立选择**：权重和偏置独立选择，允许特征组合

### 3. 物种识别（Genotype Clustering）

```rust
pub fn genotype_distance(&self, other: &Brain) -> f32 {
    let mut sum_sq = 0.0;
    for (w1, w2) in self.weights_ih.iter().zip(other.weights_ih.iter()) {
        sum_sq += (w1 - w2).powi(2);
    }
    // ... 对所有权重和偏置计算
    sum_sq.sqrt()
}

// 物种聚类
let mut representatives: Vec<&Brain> = Vec::new();
let threshold = 2.0;
for e in entities {
    let mut found = false;
    for rep in &representatives {
        if e.brain.genotype_distance(rep) < threshold {
            found = true;
            break;
        }
    }
    if !found {
        representatives.push(&e.brain);
    }
}
self.species_count = representatives.len();
```

**设计经验：**
- **欧氏距离**：使用 L2 范数衡量基因型差异
- **动态阈值**：`threshold = 2.0` - 需要根据进化阶段调整
- **贪心聚类**：简化版 K-means，性能足够且代码简单
- **物种计数**：用于 Era System 的状态判定（如物种数 > 3 进入 Flourishing Era）

---

## 空间索引：SpatialHash

### 问题
O(N²) 的感知查询（每个代理感知所有其他代理）在大量代理时性能灾难。

### 解决方案：空间哈希网格

```rust
pub struct SpatialHash {
    grid: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f64,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    pub fn insert(&mut self, x: f64, y: f64, entity_index: usize) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        self.grid.entry((cell_x, cell_y)).or_insert_with(Vec::new).push(entity_index);
    }

    pub fn query(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut result = Vec::new();
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        let range = (radius / self.cell_size).ceil() as i32;

        for dx in -range..=range {
            for dy in -range..=range {
                if let Some(cell) = self.grid.get(&(cell_x + dx, cell_y + dy)) {
                    result.extend(cell.iter());
                }
            }
        }
        result
    }
}
```

**使用方式：**
```rust
// 每帧重建
self.spatial_hash.clear();
for (i, e) in self.entities.iter().enumerate() {
    self.spatial_hash.insert(e.x, e.y, i);
}

// 查询附近代理（用于猎物寻找和交配）
let nearby = self.spatial_hash.query(entity.x, entity.y, 5.0);
```

**性能对比：**
| 代理数量 | O(N²) 查询 | 空间哈希 |
|---------|-----------|---------|
| 100     | 10,000    | ~20     |
| 1000    | 1,000,000 | ~200    |
| 10000   | 100,000,000 | ~2000 |

**设计经验：**
- **cell_size = 5.0**：平衡粒度和查询范围
- **每帧重建**：简单直接，避免复杂的一致性维护
- **半径查询**：支持任意感知半径
- **返回引用索引**：避免克隆实体，保持高性能

---

## 环境耦合系统

### 1. 硬件感知环境（Environment）

```rust
pub struct Environment {
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
    pub load_avg: f64,
    pub current_era: Era,
    pub current_season: Season,
    // ... 事件计时器
}
```

**CPU → 新陈代谢：**
```rust
pub fn metabolism_multiplier(&self) -> f64 {
    let base = match self.climate() {
        ClimateState::Temperate => 1.0,
        ClimateState::Warm => 1.5,
        ClimateState::Hot => 2.0,
        ClimateState::Scorching => 3.0,
    };
    base * self.current_season.metabolism_multiplier()
}
```

**RAM → 食物稀缺：**
```rust
pub fn food_spawn_multiplier(&self) -> f64 {
    let base = match self.resource_state() {
        ResourceState::Abundant => 1.0,
        ResourceState::Strained => 0.7,
        ResourceState::Scarce => 0.4,
        ResourceState::Famine => 0.1,
    };
    base * self.current_season.food_multiplier()
}
```

**设计经验：**
- **实时耦合**：每秒读取一次系统指标（通过 `sysinfo` crate）
- **事件计时器**：`heat_wave_timer`、`ice_age_timer` 实现阈值触发机制
- **季节循环**：`season_duration = 10000` ticks 平衡节奏和可变性

### 2. Era System：叙事引擎

```rust
pub enum Era {
    Primordial,    // 混沌适应期
    DawnOfLife,    // 稳定种群期
    Flourishing,   // 高多样性期
    DominanceWar,  // 高捕食期
    ApexEra,       // 巅峰适应期
}

pub fn update_era(&mut self, tick: u64, pop_stats: &PopulationStats) {
    if self.current_era == Era::Primordial {
        if tick > 5000 && pop_stats.avg_lifespan > 200.0 {
            self.current_era = Era::DawnOfLife;
        }
    } else if self.current_era == Era::DawnOfLife {
        if pop_stats.population > 200 && pop_stats.species_count > 3 {
            self.current_era = Era::Flourishing;
        }
    } else if self.current_era == Era::Flourishing {
        if self.cpu_usage > 70.0 {
            self.current_era = Era::DominanceWar;
        }
    }

    if pop_stats.top_fitness > 5000.0 {
        self.current_era = Era::ApexEra;
    }
}
```

**设计经验：**
- **多条件触发**：结合时间、种群统计、硬件指标
- **单一方向**：Era 只前进不倒退，叙事清晰
- **视觉反馈**：每个 Era 有独特的图标（🌀🌱🌸⚔️👑）

---

## 统计与分析

### 1. PopulationStats：种群统计

```rust
pub struct PopulationStats {
    pub population: usize,
    pub avg_lifespan: f64,
    pub avg_brain_entropy: f64,
    pub species_count: usize,
    pub top_fitness: f64,
    recent_deaths: VecDeque<f64>,  // 滚动窗口
}

impl PopulationStats {
    pub fn record_death(&mut self, lifespan: u64) {
        self.recent_deaths.push_back(lifespan as f64);
        if self.recent_deaths.len() > 100 {
            self.recent_deaths.pop_front();
        }
        self.avg_lifespan = self.recent_deaths.iter().sum::<f64>() / self.recent_deaths.len() as f64;
    }
}
```

**设计经验：**
- **滚动平均**：`VecDeque` 容量 100，避免历史数据爆炸
- **实时更新**：每次死亡立即记录，反映即时环境压力
- **物种计数**：基于基因型距离的动态聚类

### 2. Brain Entropy：大脑熵

```rust
pub fn update_snapshot(&mut self, entities: &[Entity]) {
    // Shannon Entropy: H = -Σ p(x) * log₂(p(x))
    let mut weight_freq = HashMap::new();
    for e in entities {
        for &w in &e.brain.weights_ih[0..8] {  // 采样前 8 个权重
            let bin = (w * 5.0).round() as i32;  // 分箱到 0.2 增量
            *weight_freq.entry(bin).or_insert(0.0) += 1.0;
        }
    }
    let total_samples = weight_freq.values().sum::<f64>();
    let mut entropy = 0.0;
    for &count in weight_freq.values() {
        let p = count / total_samples;
        if p > 0.0 {
            entropy -= p * p.log2();  // Shannon Entropy
        }
    }
    self.avg_brain_entropy = entropy;
}
```

**设计经验：**
- **采样优化**：只计算前 8 个权重，避免性能瓶颈
- **分箱策略**：0.2 增量平衡精度和稳定性
- **信息论度量**：熵值反映大脑多样性和探索能力

---

## 行为模式

### 1. 猎食（Predation）

```rust
if predation_mode {
    for t_idx in self.spatial_hash.query(entity.x, entity.y, 1.5) {
        let (v_id, _, _, v_e, _, _) = entity_snapshots[t_idx];
        if v_id != entity.id && !killed_ids.contains(&v_id) && v_e < entity.energy {
            entity.energy += v_e * 0.8;  // 获得猎物 80% 能量
            killed_ids.insert(v_id);
            // ... 记录死亡事件
        }
    }
}
```

**设计经验：**
- **能量成本**：捕食模式移动成本 ×2.0
- **弱肉强食**：只能吃比自己能量低的代理
- **能量传递效率**：80% - 符合生态系统能量金字塔
- **瞬间死亡**：捕食没有战斗动画，保持代码简单

### 2. 繁殖（Reproduction）

```rust
if entity.energy > reproduction_threshold {
    let mate_indices = self.spatial_hash.query(entity.x, entity.y, 2.0);
    let mut mate_idx = None;
    for m_idx in mate_indices {
        if m_idx != i && !killed_ids.contains(&entities[m_idx].id) && entities[m_idx].energy > 100.0 {
            mate_idx = Some(m_idx);
            break;
        }
    }
    let baby = if let Some(m_idx) = mate_idx {
        // 性繁殖：基因交叉
        let child_brain = Brain::crossover(&entities[i].brain, &entities[m_idx].brain);
        child_brain.mutate_with_config(&config.evolution);
        entities[i].reproduce_with_mate(self.tick, child_brain)
    } else {
        // 无性繁殖：自我克隆 + 突变
        entities[i].reproduce(self.tick, &config.evolution)
    };
}
```

**设计经验：**
- **混合繁殖策略**：有伴侣时性繁殖，无伴侣时无性繁殖
- **能量门槛**：繁殖需要高能量（> 阈值 + 额外 100）
- **基因交换**：性繁殖引入新的基因组合
- **能量分割**：亲代和子代各获得一半能量

### 3. 觅食（Foraging）

```rust
fn sense_nearest_food(&self, entity: &Entity) -> (f64, f64) {
    let mut dx_food = 0.0;
    let mut dy_food = 0.0;
    let mut min_dist_sq = f64::MAX;
    for f in &self.food {
        let dx = f.x - entity.x;
        let dy = f.y - entity.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            dx_food = dx;
            dy_food = dy;
        }
    }
    (dx_food, dy_food)
}
```

**设计经验：**
- **全知感知**：代理知道所有食物位置，无需搜索
- **最近优先**：神经网络控制移动，感知提供方向
- **距离平方**：避免 `sqrt` 计算，提升性能

---

## 地形系统

### TerrainGrid：地形影响

```rust
pub struct TerrainGrid {
    cells: Vec<TerrainType>,
    width: usize,
    height: usize,
}

pub enum TerrainType {
    Plains,     // 平原：移动 ×1.0，食物 ×1.0
    Mountains,  // 山地：移动 ×0.3，食物 ×0.2
    Rivers,     // 河流：移动 ×1.5，食物 ×0.8
    Oasis,      // 绿洲：移动 ×1.0，食物 ×2.0
}

impl TerrainGrid {
    pub fn movement_modifier(&self, x: f64, y: f64) -> f64 {
        let cell_x = x.floor() as usize;
        let cell_y = y.floor() as usize;
        match self.cells[cell_y * self.width + cell_x] {
            TerrainType::Mountains => 0.3,
            TerrainType::Rivers => 1.5,
            _ => 1.0,
        }
    }

    pub fn food_spawn_modifier(&self, x: f64, y: f64) -> f64 {
        // ... 类似逻辑
    }
}
```

**设计经验：**
- **地形生成**：使用 Perlin Noise 或 Simplex Noise 生成自然地形
- **可视化**：不同地形用不同字符表示（▲山地，≈河流，◊绿洲）
- **生态压力**：地形创造资源不均匀分布，驱动迁移行为

---

## 数据持久化

### HistoryLogger：事件日志

```rust
pub struct HistoryLogger {
    live_file: BufWriter<File>,
}

pub fn log_event(&mut self, event: LiveEvent) -> anyhow::Result<()> {
    let json = serde_json::to_string(&event)?;
    writeln!(self.live_file, "{}", json)?;
    self.live_file.flush()?;
    Ok(())
}
```

**事件类型：**
```rust
pub enum LiveEvent {
    Birth { id, parent_id, gen, tick, timestamp },
    Death { id, age, offspring, tick, timestamp, cause },
    ClimateShift { from, to, tick, timestamp },
    Extinction { population, tick, timestamp },
}
```

**设计经验：**
- **JSONL 格式**：每行一个 JSON 对象，易于流式处理
- **实时刷新**：每次写入后 `flush()`，防止数据丢失
- **时戳记录**：每个事件包含 ISO 8601 时间戳

### Legend：传奇实体

```rust
pub struct Legend {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub birth_tick: u64,
    pub death_tick: u64,
    pub lifespan: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub peak_energy: f64,
    pub brain_dna: Brain,
    pub color_rgb: (u8, u8, u8),
}

fn archive_if_legend(&self, entity: &Entity) {
    let lifespan = self.tick - entity.birth_tick;
    if lifespan > 1000 || entity.offspring_count > 10 || entity.peak_energy > 300.0 {
        self.logger.archive_legend(Legend { /* ... */ });
    }
}
```

**设计经验：**
- **多条件判定**：寿命 > 1000 或后代 > 10 或能量峰值 > 300
- **完整基因组**：保存完整的 Brain 结构，可重放实验
- **颜色基因**：保存 RGB，用于可视化谱系

### HexDNA：基因组导出/导入

```rust
impl Brain {
    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        serde_json::from_slice(&bytes)
    }
}
```

**使用方式：**
- 按 `C` 导出选中代理的 HexDNA 到文件
- 按 `V` 从文件导入 HexDNA 到世界

**设计经验：**
- **JSON + Hex**：先序列化为 JSON，再编码为 Hex，保持可读性
- **便携格式**：文本文件易于分享和版本控制
- **实验重放**：可以导入成功基因，测试在不同环境下的表现

---

## 性能优化经验

### 1. 空间哈希
- **O(N²) → O(N)**：查询从二次复杂度降为线性
- **每帧重建**：简单直接，避免复杂的一致性维护
- **cell_size = 5.0**：平衡粒度和查询范围

### 2. 批量更新
```rust
let mut alive_entities = Vec::new();
let mut new_babies = Vec::new();
let mut killed_ids = HashSet::new>();

// ... 处理所有实体

self.entities = alive_entities;
self.entities.append(&mut new_babies);
```

**设计经验：**
- **延迟写入**：先收集变化，再一次性应用到主集合
- **避免内存重分配**：使用 `Vec::with_capacity` 预分配
- **死亡标记**：`killed_ids` 避免重复处理

### 3. 采样优化
```rust
for &w in &e.brain.weights_ih[0..8] {  // 只采样前 8 个权重
    let bin = (w * 5.0).round() as i32;
    *weight_freq.entry(bin).or_insert(0.0) += 1.0;
}
```

**设计经验：**
- **部分采样**：不计算所有 24+24+6+4=58 个权重
- **代表性足够**：前 8 个权重足够反映基因多样性
- **性能提升**：在大种群（>1000）时显著

### 4. 事件去重
```rust
pub fn sense_nearest_food(&self, entity: &Entity) -> (f64, f64) {
    let mut dx_food = 0.0;
    let mut dy_food = 0.0;
    let mut min_dist_sq = f64::MAX;
    for f in &self.food {
        let dist_sq = (f.x - entity.x).powi(2) + (f.y - entity.y).powi(2);
        if dist_sq < min_dist_sq {
            min_dist_sq = dist_sq;
            dx_food = f.x - entity.x;
            dy_food = f.y - entity.y;
        }
    }
    (dx_food, dy_food)
}
```

**设计经验：**
- **距离平方比较**：避免 `sqrt` 计算
- **单次查询**：每个代理每帧只查询一次最近食物
- **早期退出**：找到最近食物即可，无需继续遍历

---

## 调试与可视化

### 1. Hall of Fame：实时排行榜

```rust
pub struct HallOfFame {
    pub top_living: Vec<(f64, Entity)>,
}

pub fn update(&mut self, entities: &[Entity], tick: u64) {
    let mut scores: Vec<(f64, Entity)> = entities.iter().map(|e| {
        let age = tick - e.birth_tick;
        let score = (age as f64 * 0.5) + (e.offspring_count as f64 * 10.0) + (e.peak_energy * 0.2);
        (score, e.clone())
    }).collect();
    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    self.top_living = scores.into_iter().take(3).collect();
}
```

**评分公式：**
```
Score = Age × 0.5 + Offspring × 10.0 + PeakEnergy × 0.2
```

**设计经验：**
- **综合指标**：平衡寿命、繁殖和能量
- **后代权重高**：鼓励繁殖策略
- **实时更新**：每 60 ticks 更新一次（~1 秒）

### 2. 名字生成

```rust
pub fn name(&self) -> String {
    let id_str = self.id.to_string();
    let bytes = id_str.as_bytes();

    let syllables = ["ae", "ba", "co", ...];  // 24 个音节
    let prefix = ["Aethel", "Bel", "Cor", ...];  // 25 个前缀

    let p_idx = bytes[0] as usize % prefix.len();
    let s1_idx = bytes[1] as usize % syllables.len();
    let s2_idx = bytes[2] as usize % syllables.len();

    format!("{}{}{}-Gen{}", prefix[p_idx], syllables[s1_idx], syllables[s2_idx], self.generation)
}
```

**示例：**
- `Aethelbaelo-Gen1`
- `Cordaeru-Gen42`
- `Belquco-Gen7`

**设计经验：**
- **确定性生成**：相同 UUID 产生相同名字
- **可读性好**：类似真实语言的名字
- **包含代数**：直观显示进化深度

---

## 常见陷阱与解决方案

### 1. 种群爆炸
**问题：** 快速繁殖导致实体数超过性能上限

**解决方案：**
```rust
if self.entities.len() > 500 {
    // 随机移除 10% 的实体
    let remove_count = self.entities.len() / 10;
    self.entities.drain(0..remove_count);
}
```

### 2. 基因漂移
**问题：** 所有个体收敛到相同基因型，失去多样性

**解决方案：**
- 提高 `mutation_amount`
- 降低繁殖门槛，增加基因交流
- 引入环境波动（季节、灾害）

### 3. 无限能量
**问题：** 捕食导致能量无限增长

**解决方案：**
```rust
if entity.energy > entity.max_energy {
    entity.energy = entity.max_energy;
}
```

### 4. 边界卡死
**问题：** 实体卡在地图边缘

**解决方案：**
```rust
if entity.x <= 0.0 {
    entity.x = 0.0;
    entity.vx = -entity.vx;  // 反弹
} else if entity.x >= width_f {
    entity.x = width_f - 0.1;
    entity.vx = -entity.vx;
}
```

---

## 扩展方向

### 1. 复杂行为
- **合作**：群体觅食、防御联盟
- **交流**：信息素、信号传递
- **学习**：生命周期内强化学习

### 2. 生态网络
- **多层次食物链**：植物 → 食草 → 食肉 → 顶级掠食者
- **共生关系**：互利共生、寄生、片利共生
- **疾病传播**：病毒、细菌感染

### 3. 环境动力学
- **气候变化**：长期趋势 + 随机事件
- **地质事件**：火山爆发、地震、陨石撞击
- **人类干预**：资源投放、种群控制

### 4. 认知升级
- **记忆系统**：记住食物位置、危险区域
- **计划能力**：多步决策，而非即时反应
- **自我意识**：内部状态监控、情绪系统

---

## 总结

Primordium 的代理系统是一个平衡**简单性**和**复杂性**的框架：

**简单性：**
- 固定 4-6-4 神经网络架构
- 简单的突变和交叉机制
- 直接的状态机行为

**复杂性：**
- 从 68 个参数涌现出复杂行为
- 硬件耦合创造独特的进化压力
- 多层系统（地形、季节、Era）交互

**关键经验：**
1. **性能优先**：空间哈希、批量更新、采样优化
2. **可视化反馈**：符号编码、颜色映射、实时统计
3. **可扩展性**：模块化设计，易于添加新特性
4. **叙事引擎**：Era System 赋予模拟故事性

这个系统展示了**如何从简单的规则涌现出复杂的行为**——这是人工生命的核心魅力。
