use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
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
}
