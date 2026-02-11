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

/// Trait for adding persistence capabilities to [`FossilRegistry`].
pub trait FossilPersistence {
    /// Saves the fossil registry to a Gzip-compressed JSON file.
    fn save(&self, path: &str) -> Result<()>;
    /// Loads a fossil registry from a file (supports both compressed and raw JSON).
    fn load(path: &str) -> Result<FossilRegistry>;
}

impl FossilPersistence for FossilRegistry {
    /// Saves the registry using Gzip compression for efficient long-term storage.
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

    /// Loads the registry, automatically detecting if the target is compressed.
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

/// Commands for the background logging thread.
pub enum LogCommand {
    /// Log a live event to `live.jsonl`.
    Event(LiveEvent),
    /// Archive a legendary entity to `legends.json`.
    Legend(Box<Legend>),
    /// Asynchronously save the lineage registry.
    SaveLineages(LineageRegistry, String),
    /// Asynchronously save the fossil registry.
    SaveFossils(FossilRegistry, String),
    /// Synchronise both registries to the SQLite storage backend.
    SyncToStorage(LineageRegistry, FossilRegistry),
    /// Shutdown the logging thread.
    Stop,
}

/// Asynchronous logger for simulation events and historical records.
///
/// Uses a background thread to prevent disk I/O from blocking the main simulation loop.
pub struct HistoryLogger {
    sender: Option<Sender<LogCommand>>,
    log_dir: String,
    /// Handle to the SQLite storage manager, if available.
    pub storage: Option<StorageManager>,
}

impl HistoryLogger {
    /// Creates a new logger using the default "logs" directory.
    pub fn new() -> Result<Self> {
        Self::new_at("logs")
    }

    /// Creates a new logger at the specified directory.
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

    /// Creates a dummy logger that discards all logs. Useful for headless tests.
    pub fn new_dummy() -> Self {
        Self {
            sender: None,
            log_dir: "".to_string(),
            storage: None,
        }
    }

    /// Queues a live event for logging.
    pub fn log_event(&self, event: LiveEvent) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::Event(event));
        }
        Ok(())
    }

    /// Archives a legendary entity.
    pub fn archive_legend(&self, legend: Legend) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::Legend(Box::new(legend)));
        }
        Ok(())
    }

    /// Triggers an asynchronous save of the lineage registry.
    pub fn save_lineages_async(&self, registry: LineageRegistry, path: String) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::SaveLineages(registry, path));
        }
        Ok(())
    }

    /// Triggers an asynchronous save of the fossil registry.
    pub fn save_fossils_async(&self, registry: FossilRegistry, path: String) -> Result<()> {
        if let Some(ref tx) = self.sender {
            let _ = tx.send(LogCommand::SaveFossils(registry, path));
        }
        Ok(())
    }

    /// Synchronises registries with persistent storage in the background.
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

    /// Retrieves all archived legends from the filesystem.
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

    /// Constructs an ancestry tree from current living genotypes and historical records.
    pub fn get_ancestry_tree_from_genotypes(
        &self,
        living: &[std::sync::Arc<primordium_data::Genotype>],
    ) -> Result<AncestryTree> {
        let legends = self.get_all_legends()?;
        let raw_genotypes: Vec<_> = living.iter().map(|g| (**g).clone()).collect();
        Ok(AncestryTree::build(&legends, &raw_genotypes))
    }

    /// Constructs an ancestry tree for the provided living entities.
    pub fn get_ancestry_tree(&self, living: &[Entity]) -> Result<AncestryTree> {
        let genotypes: Vec<_> = living.iter().map(|e| (*e.intel.genotype).clone()).collect();
        let legends = self.get_all_legends()?;
        Ok(AncestryTree::build(&legends, &genotypes))
    }

    /// Loads all historical snapshots from the event log.
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

    /// Loads the most recent N snapshots from the event log.
    pub fn get_snapshots_recent(&self, limit: usize) -> Result<Vec<(u64, PopulationStats)>> {
        let file_path = format!("{}/live.jsonl", self.log_dir);
        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(_) => return Ok(vec![]),
        };
        let reader = BufReader::new(file);
        let mut all_snapshots = Vec::new();
        for l in reader.lines().map_while(Result::ok) {
            if let Ok(LiveEvent::Snapshot { tick, stats, .. }) =
                serde_json::from_str::<LiveEvent>(&l)
            {
                all_snapshots.push((tick, stats));
            }
        }
        if all_snapshots.len() > limit {
            Ok(all_snapshots.into_iter().rev().take(limit).collect())
        } else {
            Ok(all_snapshots)
        }
    }

    /// Computes a cryptographic hash of all legends for integrity verification.
    pub fn compute_legends_hash(legends: &[Legend]) -> Result<String> {
        let json = serde_json::to_string(legends)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }
}
