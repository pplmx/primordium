use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

pub struct HistoryLogger {
    live_file: BufWriter<File>,
}

impl HistoryLogger {
    pub fn new() -> anyhow::Result<Self> {
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
            Err(_) => return Ok(vec![]), // No legends yet
        };
        let reader = BufReader::new(file);
        let mut legends = Vec::new();
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Ok(legend) = serde_json::from_str::<Legend>(&l) {
                    legends.push(legend);
                }
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
            LiveEvent::Death { age, id, .. } => (
                format!("#{} died at age {}", &id.to_string()[..4], age),
                Color::Red,
            ),
            LiveEvent::ClimateShift { to, .. } => (format!("Climate: {}", to), Color::Yellow),
            LiveEvent::Extinction { tick, .. } => {
                (format!("Extinction at tick {}", tick), Color::Magenta)
            }
        }
    }
}
