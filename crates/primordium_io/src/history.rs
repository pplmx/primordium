use crate::lineage::AncestryTree;
use crate::storage::StorageManager;
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
        let path_buf = std::path::Path::new(path);
        let tmp_path = path_buf.with_extension("tmp");
        {
            let file = File::create(&tmp_path)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            let json = serde_json::to_string(self)?;
            encoder.write_all(json.as_bytes())?;
            encoder.finish()?;
        }
        std::fs::rename(tmp_path, path)?;
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

use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::registry::LineagePersistence;
use primordium_core::lineage_registry::LineageRegistry;

pub enum LogCommand {
    Event(LiveEvent),
    Legend(Box<Legend>),
    SaveLineages(LineageRegistry, String),
    SaveFossils(FossilRegistry, String),
    SyncToStorage(LineageRegistry, FossilRegistry),
    Stop,
}

pub struct HistoryLogger {
    sender: Option<Sender<LogCommand>>,
    log_dir: String,
    pub storage: Option<StorageManager>,
}

impl HistoryLogger {
    pub fn new() -> Result<Self> {
        Self::new_at("logs")
    }

    pub fn new_at(dir: &str) -> Result<Self> {
        if !std::path::Path::new(dir).exists() {
            std::fs::create_dir_all(dir)?;
        }

        let (tx, rx) = mpsc::channel::<LogCommand>();
        let dir_clone = dir.to_string();

        let storage = StorageManager::new(format!("{}/world.db", dir)).ok();
        let storage_sender = storage.as_ref().map(|s| s.clone_sender());

        thread::spawn(move || {
            let file_path = format!("{}/live.jsonl", dir_clone);
            let mut live_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)
                .map(BufWriter::new)
                .ok();

            let legend_path = format!("{}/legends.json", dir_clone);
            let mut legend_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(legend_path)
                .map(BufWriter::new)
                .ok();

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    LogCommand::Event(ev) => {
                        if let Some(ref mut f) = live_file {
                            if let Ok(json) = serde_json::to_string(&ev) {
                                let _ = writeln!(f, "{}", json);
                                let _ = f.flush();
                            }
                        }
                    }
                    LogCommand::Legend(lg) => {
                        if let Some(ref mut f) = legend_file {
                            if let Ok(json) = serde_json::to_string(&*lg) {
                                let _ = writeln!(f, "{}", json);
                                let _ = f.flush();
                            }
                        }
                        if let Some(ref tx) = storage_sender {
                            let _ = tx.send(crate::storage::StorageCommand::RecordFossil {
                                lineage_id: lg.lineage_id,
                                tick: lg.death_tick,
                                genotype: lg.genotype.to_hex(),
                                reason: "Legendary Organism".to_string(),
                            });
                        }
                    }
                    LogCommand::SaveLineages(reg, path) => {
                        if let Err(e) = reg.save(path) {
                            eprintln!("HistoryLogger: Error saving lineages: {}", e);
                        }
                    }
                    LogCommand::SaveFossils(reg, path) => {
                        if let Err(e) = reg.save(&path) {
                            eprintln!("HistoryLogger: Error saving fossils: {}", e);
                        }
                    }
                    LogCommand::SyncToStorage(lin_reg, fos_reg) => {
                        if let Some(ref tx) = storage_sender {
                            let _ =
                                tx.send(crate::storage::StorageCommand::BatchSyncLineages(lin_reg));
                            let _ =
                                tx.send(crate::storage::StorageCommand::BatchSyncFossils(fos_reg));
                        }
                    }
                    LogCommand::Stop => break,
                }
            }
        });

        Ok(Self {
            sender: Some(tx),
            log_dir: dir.to_string(),
            storage,
        })
    }

    pub fn new_dummy() -> Self {
        Self {
            sender: None,
            log_dir: "".to_string(),
            storage: None,
        }
    }

    pub fn log_event(&self, event: LiveEvent) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::Event(event));
        }
        Ok(())
    }

    pub fn archive_legend(&self, legend: Legend) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::Legend(Box::new(legend)));
        }
        Ok(())
    }

    pub fn save_lineages_async(&self, registry: LineageRegistry, path: String) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::SaveLineages(registry, path));
        }
        Ok(())
    }

    pub fn save_fossils_async(&self, registry: FossilRegistry, path: String) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::SaveFossils(registry, path));
        }
        Ok(())
    }

    pub fn sync_to_storage_async(
        &self,
        lin_reg: LineageRegistry,
        fos_reg: FossilRegistry,
    ) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::SyncToStorage(lin_reg, fos_reg));
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
