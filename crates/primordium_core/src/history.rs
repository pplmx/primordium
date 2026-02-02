use crate::lineage_tree::AncestryTree;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
pub use primordium_data::{Entity, Fossil, HallOfFame, Legend, PopulationStats};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
