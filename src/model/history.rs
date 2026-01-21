use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use uuid::Uuid;

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Legend {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub birth_tick: u64,
    pub death_tick: u64,
    pub lifespan: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub peak_energy: f64,
    pub birth_timestamp: String,
    pub death_timestamp: String,
    pub brain_dna: crate::model::brain::Brain,
    pub color_rgb: (u8, u8, u8),
}

pub struct PopulationStats {
    pub population: usize,
    pub avg_lifespan: f64,
    pub avg_brain_entropy: f64,
    pub species_count: usize,
    pub top_fitness: f64,
    recent_deaths: VecDeque<f64>,
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
            recent_deaths: VecDeque::with_capacity(100),
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

    pub fn update_snapshot(&mut self, entities: &[crate::model::entity::Entity], top_fitness: f64) {
        self.population = entities.len();
        self.top_fitness = top_fitness;

        if entities.is_empty() {
            self.avg_brain_entropy = 0.0;
            self.species_count = 0;
            return;
        }

        // 1. Recalculate Entropy (Shannon entropy of sampled brain weights)
        let mut weight_freq = HashMap::new();
        for e in entities {
            // Sample first 8 weights for performance
            for &w in &e.intel.brain.weights_ih[0..8] {
                let bin = (w * 5.0).round() as i32; // Bin into 0.2 increments
                *weight_freq.entry(bin).or_insert(0.0) += 1.0;
            }
        }
        let total_samples = weight_freq.values().sum::<f64>();
        let mut entropy = 0.0;
        for &count in weight_freq.values() {
            let p = count / total_samples;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        self.avg_brain_entropy = entropy;

        // 2. Count Species (Genotype distance clustering)
        let mut representatives: Vec<&crate::model::brain::Brain> = Vec::new();
        let threshold = 2.0;
        for e in entities {
            let mut found = false;
            for rep in &representatives {
                if e.intel.brain.genotype_distance(rep) < threshold {
                    found = true;
                    break;
                }
            }
            if !found {
                representatives.push(&e.intel.brain);
            }
        }
        self.species_count = representatives.len();
    }
}

pub struct HistoryLogger {
    live_file: BufWriter<File>,
}

impl HistoryLogger {
    pub fn new() -> anyhow::Result<Self> {
        if !std::path::Path::new("logs").exists() {
            std::fs::create_dir_all("logs")?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/live.jsonl")?;
        Ok(Self {
            live_file: BufWriter::new(file),
        })
    }

    pub fn log_event(&mut self, event: LiveEvent) -> anyhow::Result<()> {
        let json = serde_json::to_string(&event)?;
        writeln!(self.live_file, "{}", json)?;
        self.live_file.flush()?;
        Ok(())
    }

    pub fn archive_legend(&self, legend: Legend) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("logs/legends.json")?;

        let json = serde_json::to_string(&legend)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    pub fn get_all_legends(&self) -> anyhow::Result<Vec<Legend>> {
        let file = match File::open("logs/legends.json") {
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
                // Provide context on what the climate change means
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
        }
    }
}
