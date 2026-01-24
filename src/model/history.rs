use crate::model::infra::lineage_tree::AncestryTree;
use crate::model::state::entity::Entity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fossil {
    pub lineage_id: Uuid,
    pub name: String,
    pub color_rgb: (u8, u8, u8),
    pub avg_lifespan: f64,
    pub max_generation: u32,
    pub total_offspring: u32,
    pub extinct_tick: u64,
    pub peak_population: usize,
    pub genotype: crate::model::state::entity::Genotype,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FossilRegistry {
    pub fossils: Vec<Fossil>,
}

impl FossilRegistry {
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &str) -> anyhow::Result<Self> {
        if !std::path::Path::new(path).exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        let registry = serde_json::from_str(&data)?;
        Ok(registry)
    }

    pub fn add_fossil(&mut self, fossil: Fossil) {
        self.fossils.push(fossil);
        if self.fossils.len() > 100 {
            self.fossils
                .sort_by(|a, b| b.total_offspring.cmp(&a.total_offspring));
            self.fossils.truncate(100);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event")]
pub enum LiveEvent {
    Birth {
        id: Uuid,
        parent_id: Option<Uuid>,
        gen: u32,
        tick: u64,
        timestamp: String,
    },
    Death {
        id: Uuid,
        age: u64,
        offspring: u32,
        tick: u64,
        timestamp: String,
        #[serde(default)]
        cause: String,
    },
    ClimateShift {
        from: String,
        to: String,
        tick: u64,
        timestamp: String,
    },
    Extinction {
        population: usize,
        tick: u64,
        timestamp: String,
    },
    EcoAlert {
        message: String,
        tick: u64,
        timestamp: String,
    },
    Snapshot {
        tick: u64,
        stats: PopulationStats,
        timestamp: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Legend {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub lineage_id: Uuid,
    pub birth_tick: u64,
    pub death_tick: u64,
    pub lifespan: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub peak_energy: f64,
    pub birth_timestamp: String,
    pub death_timestamp: String,
    pub genotype: crate::model::state::entity::Genotype,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PopulationStats {
    pub population: usize,
    pub avg_lifespan: f64,
    pub avg_brain_entropy: f64,
    pub species_count: usize,
    pub top_fitness: f64,
    pub biomass_h: f64,
    pub biomass_c: f64,
    pub food_count: usize,
    pub lineage_counts: HashMap<Uuid, usize>,
    pub carbon_level: f64,
    pub biodiversity_hotspots: usize,
    pub mutation_scale: f32,
    pub evolutionary_velocity: f32, // NEW: Moving average of genetic distances
    recent_deaths: VecDeque<f64>,
    recent_distances: VecDeque<f32>, // NEW
}

impl Default for PopulationStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PopulationStats {
    pub fn new() -> Self {
        Self {
            population: 0,
            avg_lifespan: 0.0,
            avg_brain_entropy: 0.0,
            species_count: 0,
            top_fitness: 0.0,
            biomass_h: 0.0,
            biomass_c: 0.0,
            food_count: 0,
            lineage_counts: HashMap::new(),
            carbon_level: 0.0,
            biodiversity_hotspots: 0,
            mutation_scale: 1.0,
            evolutionary_velocity: 0.0,
            recent_deaths: VecDeque::with_capacity(100),
            recent_distances: VecDeque::with_capacity(100),
        }
    }

    pub fn record_birth_distance(&mut self, distance: f32) {
        self.recent_distances.push_back(distance);
        if self.recent_distances.len() > 100 {
            self.recent_distances.pop_front();
        }
        if !self.recent_distances.is_empty() {
            self.evolutionary_velocity =
                self.recent_distances.iter().sum::<f32>() / self.recent_distances.len() as f32;
        }
    }

    pub fn record_death(&mut self, lifespan: u64) {
        self.recent_deaths.push_back(lifespan as f64);
        if self.recent_deaths.len() > 100 {
            self.recent_deaths.pop_front();
        }
        self.avg_lifespan =
            self.recent_deaths.iter().sum::<f64>() / self.recent_deaths.len() as f64;
    }

    pub fn update_snapshot(
        &mut self,
        entities: &[crate::model::state::entity::Entity],
        food_count: usize,
        top_fitness: f64,
        carbon_level: f64,
        mutation_scale: f32,
    ) {
        self.population = entities.len();
        self.food_count = food_count;
        self.top_fitness = top_fitness;
        self.carbon_level = carbon_level;
        self.mutation_scale = mutation_scale;
        self.lineage_counts.clear();
        self.biomass_h = 0.0;
        self.biomass_c = 0.0;
        self.biodiversity_hotspots = 0;

        if entities.is_empty() {
            self.avg_brain_entropy = 0.0;
            self.species_count = 0;
            return;
        }

        let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
        for e in entities {
            *self
                .lineage_counts
                .entry(e.metabolism.lineage_id)
                .or_insert(0) += 1;
            let tp = e.metabolism.trophic_potential;
            if tp < 0.4 {
                self.biomass_h += e.metabolism.energy;
            } else if tp > 0.6 {
                self.biomass_c += e.metabolism.energy;
            }
            let sx = (e.physics.x / 10.0) as i32;
            let sy = (e.physics.y / 10.0) as i32;
            sectors
                .entry((sx, sy))
                .or_default()
                .insert(e.metabolism.lineage_id);
        }
        self.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();

        let mut complexity_freq = HashMap::new();
        for e in entities {
            let conn_count = e
                .intel
                .genotype
                .brain
                .connections
                .iter()
                .filter(|c| c.enabled)
                .count();
            let bucket = (conn_count / 10) * 10;
            *complexity_freq.entry(bucket).or_insert(0.0) += 1.0;
        }
        let total_samples = complexity_freq.values().sum::<f64>();
        let mut entropy = 0.0;
        for &count in complexity_freq.values() {
            let p = count / total_samples;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        self.avg_brain_entropy = entropy;

        let mut representatives: Vec<&crate::model::brain::Brain> = Vec::new();
        let threshold = 2.0;
        for e in entities {
            let mut found = false;
            for rep in &representatives {
                if e.intel.genotype.brain.genotype_distance(rep) < threshold {
                    found = true;
                    break;
                }
            }
            if !found {
                representatives.push(&e.intel.genotype.brain);
            }
        }
        self.species_count = representatives.len();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HallOfFame {
    pub top_living: Vec<(f64, crate::model::state::entity::Entity)>,
}

impl Default for HallOfFame {
    fn default() -> Self {
        Self::new()
    }
}

impl HallOfFame {
    pub fn new() -> Self {
        Self {
            top_living: Vec::with_capacity(3),
        }
    }
    pub fn update(&mut self, entities: &[crate::model::state::entity::Entity], tick: u64) {
        let mut scores: Vec<(f64, crate::model::state::entity::Entity)> = entities
            .iter()
            .map(|e| {
                let age = tick - e.metabolism.birth_tick;
                let score = (age as f64 * 0.5)
                    + (e.metabolism.offspring_count as f64 * 10.0)
                    + (e.metabolism.peak_energy * 0.2);
                (score, e.clone())
            })
            .collect();
        scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        self.top_living = scores.into_iter().take(3).collect();
    }
}

pub struct HistoryLogger {
    live_file: Option<BufWriter<File>>,
    log_dir: String,
}

impl HistoryLogger {
    pub fn new() -> anyhow::Result<Self> {
        Self::new_at("logs")
    }

    pub fn new_at(dir: &str) -> anyhow::Result<Self> {
        if !std::path::Path::new(dir).exists() {
            std::fs::create_dir_all(dir)?;
        }
        let file_path = format!("{}/live.jsonl", dir);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        Ok(Self {
            live_file: Some(BufWriter::new(file)),
            log_dir: dir.to_string(),
        })
    }

    pub fn new_dummy() -> Self {
        Self {
            live_file: None,
            log_dir: "".to_string(),
        }
    }

    pub fn log_event(&mut self, event: LiveEvent) -> anyhow::Result<()> {
        if let Some(ref mut file) = self.live_file {
            let json = serde_json::to_string(&event)?;
            writeln!(file, "{}", json)?;
            file.flush()?;
        }
        Ok(())
    }

    pub fn archive_legend(&self, legend: Legend) -> anyhow::Result<()> {
        if self.live_file.is_some() {
            let file_path = format!("{}/legends.json", self.log_dir);
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;
            let json = serde_json::to_string(&legend)?;
            writeln!(file, "{}", json)?;
        }
        Ok(())
    }

    pub fn get_all_legends(&self) -> anyhow::Result<Vec<Legend>> {
        let file_path = format!("{}/legends.json", self.log_dir);
        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => return Ok(vec![]),
        };
        let reader = BufReader::new(file);
        let mut legends = Vec::new();
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(legend) = serde_json::from_str::<Legend>(&l) {
                legends.push(legend);
            }
        }
        Ok(legends)
    }

    pub fn get_ancestry_tree(&self, living: &[Entity]) -> anyhow::Result<AncestryTree> {
        let legends = self.get_all_legends()?;
        Ok(AncestryTree::build(&legends, living))
    }

    pub fn get_snapshots(&self) -> anyhow::Result<Vec<(u64, PopulationStats)>> {
        let file_path = format!("{}/live.jsonl", self.log_dir);
        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => return Ok(vec![]),
        };
        let reader = BufReader::new(file);
        let mut snapshots = Vec::new();
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(LiveEvent::Snapshot { tick, stats, .. }) =
                serde_json::from_str::<LiveEvent>(&l)
            {
                snapshots.push((tick, stats));
            }
        }
        Ok(snapshots)
    }

    pub fn compute_legends_hash(legends: &[Legend]) -> anyhow::Result<String> {
        let json = serde_json::to_string(legends)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
}

impl LiveEvent {
    pub fn to_ui_message(&self) -> (String, ratatui::style::Color) {
        use ratatui::style::Color;
        match self {
            LiveEvent::Birth { gen, id, .. } => (
                format!("Gen {} #{} born", gen, &id.to_string()[..4]),
                Color::Cyan,
            ),
            LiveEvent::Death { age, id, cause, .. } => {
                let msg = if cause.is_empty() {
                    format!("#{} died at age {}", &id.to_string()[..4], age)
                } else {
                    format!(
                        "#{} killed by {} at age {}",
                        &id.to_string()[..4],
                        cause,
                        age
                    )
                };
                (msg, Color::Red)
            }
            LiveEvent::ClimateShift { from, to, .. } => {
                let effect = match to.as_str() {
                    "Temperate" => "☀️ Temperate - Metabolism ×1.0 (stable)",
                    "Warm" => "🔥 Warm - Metabolism ×1.5 (faster drain)",
                    "Hot" => "🌋 Hot - Metabolism ×2.0 (high stress)",
                    "Scorching" => "☀️ SCORCHING - Metabolism ×3.0 (DANGER!)",
                    _ => to.as_str(),
                };
                let direction = if from == "Temperate" && to != "Temperate" {
                    "⬆️"
                } else if to == "Temperate" {
                    "⬇️"
                } else {
                    "→"
                };
                (
                    format!("{} Climate {} {}", direction, direction, effect),
                    Color::Yellow,
                )
            }
            LiveEvent::Extinction { tick, .. } => {
                (format!("Extinction at tick {}", tick), Color::Magenta)
            }
            LiveEvent::EcoAlert { message, .. } => (format!("⚠️ {}", message), Color::Yellow),
            LiveEvent::Snapshot { tick, .. } => (
                format!("🏛️ Snapshot saved at tick {}", tick),
                Color::DarkGray,
            ),
        }
    }
}
