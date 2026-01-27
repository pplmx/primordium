use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LineageGoal {
    Expansion,
    Dominance,
    Resilience,
}

/// High-level metrics for an ancestral line.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineageRecord {
    pub id: Uuid,
    pub name: String,
    pub total_entities_produced: usize,
    pub current_population: usize,
    pub peak_population: usize,
    pub max_generation: u32,
    pub total_energy_consumed: f64,
    pub first_appearance_tick: u64,
    pub is_extinct: bool,
    pub best_legend_id: Option<Uuid>,
    pub completed_goals: std::collections::HashSet<LineageGoal>,
    #[serde(
        skip,
        default = "crate::model::state::lineage_registry::create_shared_memory"
    )]
    pub collective_memory: std::sync::Arc<std::sync::RwLock<HashMap<String, f32>>>,
}

pub fn create_shared_memory() -> std::sync::Arc<std::sync::RwLock<HashMap<String, f32>>> {
    std::sync::Arc::new(std::sync::RwLock::new(HashMap::new()))
}

impl Default for LineageRecord {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: "Unknown".to_string(),
            total_entities_produced: 0,
            current_population: 0,
            peak_population: 0,
            max_generation: 0,
            total_energy_consumed: 0.0,
            first_appearance_tick: 0,
            is_extinct: false,
            best_legend_id: None,
            completed_goals: std::collections::HashSet::new(),
            collective_memory: create_shared_memory(),
        }
    }
}

/// Persistent registry of all lineages that have ever existed in the world.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct LineageRegistry {
    pub lineages: HashMap<Uuid, LineageRecord>,
}

impl LineageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_birth(&mut self, id: Uuid, gen: u32, tick: u64) {
        let entry = self.lineages.entry(id).or_insert_with(|| LineageRecord {
            id,
            name: format!("Lineage-{}", &id.to_string()[..4]),
            first_appearance_tick: tick,
            ..Default::default()
        });
        entry.total_entities_produced += 1;
        entry.current_population += 1;
        if entry.current_population > entry.peak_population {
            entry.peak_population = entry.current_population;
        }
        if gen > entry.max_generation {
            entry.max_generation = gen;
        }
        entry.is_extinct = false;
    }

    pub fn record_migration_in(&mut self, id: Uuid, gen: u32, tick: u64) {
        let entry = self.lineages.entry(id).or_insert_with(|| LineageRecord {
            id,
            name: format!("Migrant-{}", &id.to_string()[..4]),
            first_appearance_tick: tick,
            ..Default::default()
        });
        entry.current_population += 1;
        if entry.current_population > entry.peak_population {
            entry.peak_population = entry.current_population;
        }
        if gen > entry.max_generation {
            entry.max_generation = gen;
        }
        entry.is_extinct = false;
    }

    pub fn record_death(&mut self, id: Uuid) {
        if let Some(record) = self.lineages.get_mut(&id) {
            record.current_population = record.current_population.saturating_sub(1);
            if record.current_population == 0 {
                record.is_extinct = true;
            }
        }
    }

    pub fn record_consumption(&mut self, id: Uuid, amount: f64) {
        if let Some(record) = self.lineages.get_mut(&id) {
            record.total_energy_consumed += amount;
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let registry = serde_json::from_reader(reader)?;
        Ok(registry)
    }

    pub fn get_top_lineages(&self, count: usize) -> Vec<(&Uuid, &LineageRecord)> {
        let mut list: Vec<_> = self.lineages.iter().collect();
        list.sort_by(|a, b| {
            b.1.total_entities_produced
                .cmp(&a.1.total_entities_produced)
        });
        list.into_iter().take(count).collect()
    }

    pub fn update_best_legend(&mut self, lineage_id: Uuid, legend_id: Uuid) {
        if let Some(record) = self.lineages.get_mut(&lineage_id) {
            record.best_legend_id = Some(legend_id);
        }
    }

    pub fn get_memory_value(&self, id: &Uuid, key: &str) -> f32 {
        if let Some(record) = self.lineages.get(id) {
            if let Ok(mem) = record.collective_memory.read() {
                return *mem.get(key).unwrap_or(&0.0);
            }
        }
        0.0
    }

    pub fn set_memory_value(&self, id: &Uuid, key: &str, value: f32) {
        if let Some(record) = self.lineages.get(id) {
            if let Ok(mut mem) = record.collective_memory.write() {
                mem.insert(key.to_string(), value);
            }
        }
    }

    pub fn get_extinct_lineages(&self) -> Vec<Uuid> {
        self.lineages
            .iter()
            .filter(|(_, r)| r.is_extinct)
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn check_goals(&mut self, tick: u64, _social_grid: &[u8], _width: u16, _height: u16) {
        for record in self.lineages.values_mut() {
            if !record.is_extinct {
                if record.current_population >= 50
                    && !record.completed_goals.contains(&LineageGoal::Expansion)
                {
                    record.completed_goals.insert(LineageGoal::Expansion);
                }

                if tick.saturating_sub(record.first_appearance_tick) >= 2000
                    && !record.completed_goals.contains(&LineageGoal::Resilience)
                {
                    record.completed_goals.insert(LineageGoal::Resilience);
                }
            }
        }
    }

    pub fn prune(&mut self) {
        // Prune lineages that are extinct and have produced less than 3 entities
        // and have no legendary representative (best_legend_id).
        self.lineages.retain(|_, record| {
            !record.is_extinct
                || record.total_entities_produced >= 3
                || record.best_legend_id.is_some()
        });
    }
}
