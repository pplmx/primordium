use crate::model::brain::BrainLogic;
use crate::model::infra::lineage_tree::AncestryTree;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
pub use primordium_data::{Entity, Fossil, HallOfFame, Legend, PopulationStats};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FossilRegistry {
    pub fossils: Vec<Fossil>,
}

impl FossilRegistry {
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        let json = serde_json::to_string(self)?;
        encoder.write_all(json.as_bytes())?;
        encoder.finish()?;
        Ok(())
    }

    pub fn load(path: &str) -> anyhow::Result<Self> {
        let path_gz = if path.ends_with(".gz") {
            path.to_string()
        } else {
            format!("{}.gz", path)
        };

        let target_path = if std::path::Path::new(&path_gz).exists() {
            &path_gz
        } else if std::path::Path::new(path).exists() {
            path
        } else {
            return Ok(Self::default());
        };

        let file = File::open(target_path)?;
        let mut decoder = GzDecoder::new(file);
        let mut decoded_data = Vec::new();
        if decoder.read_to_end(&mut decoded_data).is_ok() {
            let registry = serde_json::from_slice(&decoded_data)?;
            Ok(registry)
        } else {
            let data = std::fs::read_to_string(target_path)?;
            let registry = serde_json::from_str(&data)?;
            Ok(registry)
        }
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
        cause: String,
    },
    Metamorphosis {
        id: Uuid,
        name: String,
        tick: u64,
        timestamp: String,
    },
    TribalSplit {
        id: Uuid,
        lineage_id: Uuid,
        tick: u64,
        timestamp: String,
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
    Narration {
        tick: u64,
        text: String,
        severity: f32,
        timestamp: String,
    },
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
        Ok(hex::encode(hasher.finalize()))
    }
}

pub fn update_population_stats(
    stats: &mut PopulationStats,
    entities: &[Entity],
    food_count: usize,
    top_fitness: f64,
    carbon_level: f64,
    mutation_scale: f32,
    terrain: &crate::model::terrain::TerrainGrid,
) {
    stats.population = entities.len();
    stats.food_count = food_count;
    stats.top_fitness = top_fitness;
    stats.carbon_level = carbon_level;
    stats.mutation_scale = mutation_scale;
    stats.global_fertility = terrain.average_fertility();
    stats.max_generation = entities
        .iter()
        .map(|e| e.metabolism.generation)
        .max()
        .unwrap_or(0);
    stats.lineage_counts.clear();
    stats.biomass_h = 0.0;
    stats.biomass_c = 0.0;
    stats.biodiversity_hotspots = 0;

    if entities.is_empty() {
        stats.avg_brain_entropy = 0.0;
        stats.species_count = 0;
        return;
    }

    let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
    for e in entities {
        *stats
            .lineage_counts
            .entry(e.metabolism.lineage_id)
            .or_insert(0) += 1;
        let tp = e.metabolism.trophic_potential;
        if tp < 0.4 {
            stats.biomass_h += e.metabolism.energy;
        } else if tp > 0.6 {
            stats.biomass_c += e.metabolism.energy;
        }
        let sx = (e.physics.x / 10.0) as i32;
        let sy = (e.physics.y / 10.0) as i32;
        sectors
            .entry((sx, sy))
            .or_default()
            .insert(e.metabolism.lineage_id);
    }
    stats.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();

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
    stats.avg_brain_entropy = entropy;

    let mut representatives: Vec<&primordium_data::Brain> = Vec::new();
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
    stats.species_count = representatives.len();
}

pub fn record_stat_death(stats: &mut PopulationStats, lifespan: u64) {
    stats.recent_deaths.push_back(lifespan as f64);
    if stats.recent_deaths.len() > 100 {
        stats.recent_deaths.pop_front();
    }
    if !stats.recent_deaths.is_empty() {
        stats.avg_lifespan =
            stats.recent_deaths.iter().sum::<f64>() / stats.recent_deaths.len() as f64;
    }
}

pub fn record_stat_birth_distance(stats: &mut PopulationStats, distance: f32) {
    stats.recent_distances.push_back(distance);
    if stats.recent_distances.len() > 100 {
        stats.recent_distances.pop_front();
    }
    if !stats.recent_distances.is_empty() {
        stats.evolutionary_velocity =
            stats.recent_distances.iter().sum::<f32>() / stats.recent_distances.len() as f32;
    }
}

pub fn update_hall_of_fame(hof: &mut HallOfFame, entities: &[Entity], tick: u64) {
    let mut scores: Vec<(f64, Entity)> = entities
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
    hof.top_living = scores.into_iter().take(3).collect();
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
            LiveEvent::ClimateShift { from: _, to, .. } => {
                let effect = match to.as_str() {
                    "Temperate" => "☀️ Temperate - ×1.0",
                    "Warm" => "🔥 Warm - ×1.5",
                    "Hot" => "🌋 Hot - ×2.0",
                    "Scorching" => "☀️ SCORCHING - ×3.0",
                    _ => to.as_str(),
                };
                (format!("Climate: {}", effect), Color::Yellow)
            }
            LiveEvent::Extinction { tick, .. } => {
                (format!("Extinction at tick {}", tick), Color::Magenta)
            }
            LiveEvent::EcoAlert { message, .. } => (format!("⚠️ {}", message), Color::Yellow),
            LiveEvent::Metamorphosis { name, .. } => {
                (format!("✨ {} has metamorphosed!", name), Color::Yellow)
            }
            LiveEvent::TribalSplit { id, .. } => (
                format!("⚔️ #{} split into a new tribe!", &id.to_string()[..4]),
                Color::Magenta,
            ),
            LiveEvent::Snapshot { tick, .. } => (
                format!("🏛️ Snapshot saved at tick {}", tick),
                Color::DarkGray,
            ),
            LiveEvent::Narration { text, .. } => (format!("📜 {}", text), Color::Green),
        }
    }
}
