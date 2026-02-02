use crate::lineage::AncestryTree;
use anyhow::Result;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use primordium_data::{Entity, FossilRegistry, Legend, LiveEvent, PopulationStats};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

/// Trait for adding persistence capabilities to FossilRegistry
pub trait FossilPersistence {
    fn save(&self, path: &str) -> Result<()>;
    fn load(path: &str) -> Result<FossilRegistry>;
}

impl FossilPersistence for FossilRegistry {
    fn save(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        let json = serde_json::to_string(self)?;
        encoder.write_all(json.as_bytes())?;
        encoder.finish()?;
        Ok(())
    }

    fn load(path: &str) -> Result<FossilRegistry> {
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
            return Ok(FossilRegistry::default());
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
}

pub struct HistoryLogger {
    live_file: Option<BufWriter<File>>,
    log_dir: String,
}

impl HistoryLogger {
    pub fn new() -> Result<Self> {
        Self::new_at("logs")
    }

    pub fn new_at(dir: &str) -> Result<Self> {
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

    pub fn log_event(&mut self, event: LiveEvent) -> Result<()> {
        if let Some(ref mut file) = self.live_file {
            let json = serde_json::to_string(&event)?;
            writeln!(file, "{}", json)?;
            file.flush()?;
        }
        Ok(())
    }

    pub fn archive_legend(&self, legend: Legend) -> Result<()> {
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

    pub fn get_all_legends(&self) -> Result<Vec<Legend>> {
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

    pub fn get_ancestry_tree(&self, living: &[Entity]) -> Result<AncestryTree> {
        let legends = self.get_all_legends()?;
        Ok(AncestryTree::build(&legends, living))
    }

    pub fn get_snapshots(&self) -> Result<Vec<(u64, PopulationStats)>> {
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

    pub fn compute_legends_hash(legends: &[Legend]) -> Result<String> {
        let json = serde_json::to_string(legends)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }
}
