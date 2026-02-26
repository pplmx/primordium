use anyhow::Result;
use primordium_core::lineage_registry::LineageRegistry;
use primordium_data::FossilRegistry;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::thread;
use uuid::Uuid;

/// A genome record in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeRecord {
    pub id: Uuid,
    pub lineage_id: Option<Uuid>,
    pub genotype: String,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub fitness_score: f64,
    pub offspring_count: u32,
    pub tick: u64,
    pub downloads: u32,
    pub created_at: String,
}

/// A seed (simulation config) record in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRecord {
    pub id: Uuid,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub config_json: String,
    pub avg_tick_time: f64,
    pub max_pop: u32,
    pub performance_summary: String,
    pub downloads: u32,
    pub created_at: String,
}

/// Commands for the background storage management thread.
pub enum StorageCommand {
    /// Inserts or updates a lineage record in the SQLite database.
    UpsertLineage {
        id: Uuid,
        start_tick: u64,
        civilization_level: u32,
        is_extinct: bool,
        best_genotype: String,
    },
    /// Records a fossil event (extinction or legend) in the database.
    RecordFossil {
        lineage_id: Uuid,
        tick: u64,
        genotype: String,
        reason: String,
    },
    /// Saves a macro-state snapshot of the world.
    SaveSnapshot {
        tick: u64,
        pop_count: u32,
        carbon_level: f64,
        energy_total: f64,
        world_data: Vec<u8>,
    },
    /// Batch synchronises the entire lineage registry.
    BatchSyncLineages(LineageRegistry),
    /// Batch synchronises the entire fossil registry.
    BatchSyncFossils(FossilRegistry),
    /// Queries all fossils for a specific lineage (async response via MPSC).
    QueryFossils(Uuid, Sender<Vec<(u64, String)>>),
    /// Queries the top lineages for the Hall of Fame.
    QueryHallOfFame(Sender<Vec<(Uuid, u32, bool)>>),
    /// Queries a specific world snapshot by tick.
    QuerySnapshot(u64, Sender<Option<Vec<u8>>>),
    /// Submits a genome to the marketplace.
    SubmitGenome {
        id: Uuid,
        lineage_id: Option<Uuid>,
        genotype: String,
        author: String,
        name: String,
        description: String,
        tags: String,
        fitness_score: f64,
        offspring_count: u32,
        tick: u64,
    },
    /// Submits a seed (simulation config) to the marketplace.
    SubmitSeed {
        id: Uuid,
        author: String,
        name: String,
        description: String,
        tags: String,
        config_json: String,
        avg_tick_time: f64,
        max_pop: u32,
        performance_summary: String,
    },
    /// Query genomes from marketplace.
    QueryGenomes {
        limit: Option<usize>,
        sort_by: Option<String>, // 'fitness', 'downloads', 'tick'
        reply_tx: Sender<Vec<GenomeRecord>>,
    },
    /// Query seeds from marketplace.
    QuerySeeds {
        limit: Option<usize>,
        sort_by: Option<String>, // 'pop', 'downloads'
        reply_tx: Sender<Vec<SeedRecord>>,
    },
    /// Shutdown the storage thread.
    Stop,
}

/// Thread-safe manager for the persistent SQLite storage backend.
///
/// Uses an asynchronous command pattern to prevent database latency from
/// affecting the simulation's tick rate.
pub struct StorageManager {
    sender: Sender<StorageCommand>,
}

impl StorageManager {
    /// Returns a new sender handle to communicate with the storage thread.
    pub fn clone_sender(&self) -> Sender<StorageCommand> {
        self.sender.clone()
    }

    /// Initialises a new storage manager and spawns its background worker thread.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let path = path.as_ref().to_owned();

        thread::spawn(move || {
            let mut conn = match Connection::open(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to open database: {}", e);
                    return;
                }
            };

            if let Err(e) = init_db(&mut conn) {
                eprintln!("Failed to initialize database: {}", e);
                return;
            }

            let _ = conn.execute("PRAGMA journal_mode=WAL", []);
            let _ = conn.execute("PRAGMA synchronous = NORMAL", []);
            let _ = conn.execute("PRAGMA mmap_size = 30000000000", []);

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    StorageCommand::UpsertLineage {
                        id,
                        start_tick,
                        civilization_level,
                        is_extinct,
                        best_genotype,
                    } => {
                        let _ = conn.execute(
                            "INSERT INTO lineages (id, start_tick, civilization_level, is_extinct, best_genotype)
                              VALUES (?1, ?2, ?3, ?4, ?5)
                              ON CONFLICT(id) DO UPDATE SET
                                 civilization_level = excluded.civilization_level,
                                 is_extinct = excluded.is_extinct,
                                 best_genotype = excluded.best_genotype",
                            params![id, start_tick, civilization_level, is_extinct, best_genotype],
                        );
                    }
                    StorageCommand::RecordFossil {
                        lineage_id,
                        tick,
                        genotype,
                        reason,
                    } => {
                        let _ = conn.execute(
                            "INSERT INTO fossils (lineage_id, tick, genotype, legend_reason)
                              VALUES (?1, ?2, ?3, ?4)",
                            params![lineage_id, tick, genotype, reason],
                        );
                    }
                    StorageCommand::SaveSnapshot {
                        tick,
                        pop_count,
                        carbon_level,
                        energy_total,
                        world_data,
                    } => {
                        let _ = conn.execute(
                            "INSERT INTO world_snapshots (tick, pop_count, carbon_level, energy_total, world_data)
                              VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![tick, pop_count, carbon_level, energy_total, world_data],
                        );
                    }

                    StorageCommand::BatchSyncLineages(registry) => {
                        let tx = match conn.transaction() {
                            Ok(t) => t,
                            Err(_) => continue,
                        };
                        for (id, record) in &registry.lineages {
                            let _ = tx.execute(
                                "INSERT INTO lineages (id, start_tick, civilization_level, is_extinct, best_genotype)
                                  VALUES (?1, ?2, ?3, ?4, ?5)
                                  ON CONFLICT(id) DO UPDATE SET
                                     civilization_level = excluded.civilization_level,
                                     is_extinct = excluded.is_extinct,
                                     best_genotype = excluded.best_genotype",
                                params![
                                    id,
                                    record.first_appearance_tick,
                                    record.civilization_level,
                                    record.is_extinct,
                                    record.max_fitness_genotype.as_ref().map(|g| g.to_hex()).unwrap_or_default()
                                ],
                            );
                        }
                        let _ = tx.commit();
                    }
                    StorageCommand::BatchSyncFossils(registry) => {
                        let tx = match conn.transaction() {
                            Ok(t) => t,
                            Err(_) => continue,
                        };
                        for fossil in &registry.fossils {
                            let _ = tx.execute(
                                "INSERT INTO fossils (lineage_id, tick, genotype, legend_reason)
                                  VALUES (?1, ?2, ?3, ?4)",
                                params![
                                    fossil.lineage_id,
                                    fossil.extinct_tick,
                                    fossil.genotype.to_hex(),
                                    "Extinction"
                                ],
                            );
                        }
                        let _ = tx.commit();
                    }
                    StorageCommand::QueryFossils(lineage_id, reply_tx) => {
                        let mut stmt = match conn.prepare(
                            "SELECT tick, legend_reason FROM fossils WHERE lineage_id = ?1 ORDER BY tick DESC"
                        ) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let rows = stmt
                            .query_map(params![lineage_id], |row| Ok((row.get(0)?, row.get(1)?)));

                        if let Ok(iter) = rows {
                            let results: Vec<(u64, String)> = iter.filter_map(Result::ok).collect();
                            let _ = reply_tx.send(results);
                        }
                    }
                    StorageCommand::QueryHallOfFame(reply_tx) => {
                        let mut stmt = match conn.prepare(
                            "SELECT id, civilization_level, is_extinct FROM lineages ORDER BY civilization_level DESC LIMIT 10"
                        ) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let rows = stmt.query_map([], |row| {
                            let id_str: String = row.get(0)?;
                            let id = Uuid::parse_str(&id_str).unwrap_or_default();
                            Ok((id, row.get(1)?, row.get(2)?))
                        });

                        if let Ok(iter) = rows {
                            let results: Vec<(Uuid, u32, bool)> =
                                iter.filter_map(Result::ok).collect();
                            let _ = reply_tx.send(results);
                        }
                    }
                    StorageCommand::QuerySnapshot(tick, reply_tx) => {
                        let mut stmt = match conn
                            .prepare("SELECT world_data FROM world_snapshots WHERE tick = ?1")
                        {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let result: Option<Vec<u8>> =
                            stmt.query_row(params![tick], |row| row.get(0)).ok();
                        let _ = reply_tx.send(result);
                    }
                    StorageCommand::SubmitGenome {
                        id,
                        lineage_id,
                        genotype,
                        author,
                        name,
                        description,
                        tags,
                        fitness_score,
                        offspring_count,
                        tick,
                    } => {
                        let lineage_id_str =
                            lineage_id.map(|id| id.to_string()).unwrap_or_default();
                        let _ = conn.execute(
                            "INSERT INTO genome_submissions (id, lineage_id, genotype, author, name, description, tags, fitness_score, offspring_count, tick)
                              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                            params![
                                id, lineage_id_str, genotype, author, name, description, tags, fitness_score, offspring_count, tick
                            ],
                        );
                    }
                    StorageCommand::SubmitSeed {
                        id,
                        author,
                        name,
                        description,
                        tags,
                        config_json,
                        avg_tick_time,
                        max_pop,
                        performance_summary,
                    } => {
                        let _ = conn.execute(
                            "INSERT INTO seed_submissions (id, author, name, description, tags, config_json, avg_tick_time, max_pop, performance_summary)
                              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            params![
                                id, author, name, description, tags, config_json, avg_tick_time, max_pop, performance_summary
                            ],
                        );
                    }
                    StorageCommand::QueryGenomes {
                        limit,
                        sort_by,
                        reply_tx,
                    } => {
                        let order_by = match sort_by.as_deref() {
                            Some("fitness") => "fitness_score DESC",
                            Some("downloads") => "downloads DESC",
                            Some("tick") => "tick DESC",
                            _ => "created_at DESC",
                        };

                        let limit_clause =
                            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
                        let query = format!(
                            "SELECT id, lineage_id, genotype, author, name, description, tags, fitness_score, offspring_count, tick, downloads, created_at
                             FROM genome_submissions ORDER BY {}{}",
                            order_by, limit_clause
                        );

                        let mut stmt = match conn.prepare(&query) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let rows = stmt.query_map([], |row| {
                            let lineage_id_str: Option<String> = row.get(1)?;
                            let lineage_id = lineage_id_str.and_then(|s| Uuid::parse_str(&s).ok());
                            Ok(GenomeRecord {
                                id: row.get(0)?,
                                lineage_id,
                                genotype: row.get(2)?,
                                author: row.get(3)?,
                                name: row.get(4)?,
                                description: row.get(5)?,
                                tags: row.get(6)?,
                                fitness_score: row.get(7)?,
                                offspring_count: row.get(8)?,
                                tick: row.get(9)?,
                                downloads: row.get(10)?,
                                created_at: row.get(11)?,
                            })
                        });

                        if let Ok(iter) = rows {
                            let results: Vec<GenomeRecord> = iter.filter_map(Result::ok).collect();
                            let _ = reply_tx.send(results);
                        }
                    }
                    StorageCommand::QuerySeeds {
                        limit,
                        sort_by,
                        reply_tx,
                    } => {
                        let order_by = match sort_by.as_deref() {
                            Some("pop") => "max_pop DESC",
                            Some("downloads") => "downloads DESC",
                            _ => "created_at DESC",
                        };

                        let limit_clause =
                            limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
                        let query = format!(
                            "SELECT id, author, name, description, tags, config_json, avg_tick_time, max_pop, performance_summary, downloads, created_at
                             FROM seed_submissions ORDER BY {}{}",
                            order_by, limit_clause
                        );

                        let mut stmt = match conn.prepare(&query) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        let rows = stmt.query_map([], |row| {
                            Ok(SeedRecord {
                                id: row.get(0)?,
                                author: row.get(1)?,
                                name: row.get(2)?,
                                description: row.get(3)?,
                                tags: row.get(4)?,
                                config_json: row.get(5)?,
                                avg_tick_time: row.get(6)?,
                                max_pop: row.get(7)?,
                                performance_summary: row.get(8)?,
                                downloads: row.get(9)?,
                                created_at: row.get(10)?,
                            })
                        });

                        if let Ok(iter) = rows {
                            let results: Vec<SeedRecord> = iter.filter_map(Result::ok).collect();
                            let _ = reply_tx.send(results);
                        }
                    }
                    StorageCommand::Stop => break,
                }
            }
        });

        Ok(Self { sender: tx })
    }

    /// Queues a lineage update.
    pub fn upsert_lineage(
        &self,
        id: Uuid,
        start_tick: u64,
        civilization_level: u32,
        is_extinct: bool,
        best_genotype: String,
    ) {
        let _ = self.sender.send(StorageCommand::UpsertLineage {
            id,
            start_tick,
            civilization_level,
            is_extinct,
            best_genotype,
        });
    }

    /// Queues a fossil record entry.
    pub fn record_fossil(&self, lineage_id: Uuid, tick: u64, genotype: String, reason: String) {
        let _ = self.sender.send(StorageCommand::RecordFossil {
            lineage_id,
            tick,
            genotype,
            reason,
        });
    }

    /// Queues a world snapshot save.
    pub fn save_snapshot(
        &self,
        tick: u64,
        pop_count: u32,
        carbon_level: f64,
        energy_total: f64,
        world_data: Vec<u8>,
    ) {
        let _ = self.sender.send(StorageCommand::SaveSnapshot {
            tick,
            pop_count,
            carbon_level,
            energy_total,
            world_data,
        });
    }

    /// Queues a full synchronisation of the lineage registry.
    pub fn sync_lineages(&self, registry: LineageRegistry) {
        let _ = self
            .sender
            .send(StorageCommand::BatchSyncLineages(registry));
    }

    /// Queues a full synchronisation of the fossil registry.
    pub fn sync_fossils(&self, registry: FossilRegistry) {
        let _ = self.sender.send(StorageCommand::BatchSyncFossils(registry));
    }

    /// Asynchronously queries fossils for a specific lineage.
    pub fn query_fossils_async(
        &self,
        lineage_id: Uuid,
    ) -> Option<mpsc::Receiver<Vec<(u64, String)>>> {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(StorageCommand::QueryFossils(lineage_id, tx))
            .is_ok()
        {
            Some(rx)
        } else {
            None
        }
    }

    /// Asynchronously queries the Hall of Fame.
    pub fn query_hall_of_fame_async(&self) -> Option<mpsc::Receiver<Vec<(Uuid, u32, bool)>>> {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(StorageCommand::QueryHallOfFame(tx))
            .is_ok()
        {
            Some(rx)
        } else {
            None
        }
    }

    /// Asynchronously queries a world snapshot by tick.
    pub fn query_snapshot_async(&self, tick: u64) -> Option<mpsc::Receiver<Option<Vec<u8>>>> {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(StorageCommand::QuerySnapshot(tick, tx))
            .is_ok()
        {
            Some(rx)
        } else {
            None
        }
    }

    // Phase 70: Marketplace submission functions
    #[allow(clippy::too_many_arguments)]
    /// Submits a genome to the marketplace.
    pub fn submit_genome(
        &self,
        id: Uuid,
        lineage_id: Option<Uuid>,
        genotype: String,
        author: String,
        name: String,
        description: String,
        tags: String,
        fitness_score: f64,
        offspring_count: u32,
        tick: u64,
    ) {
        let _ = self.sender.send(StorageCommand::SubmitGenome {
            id,
            lineage_id,
            genotype,
            author,
            name,
            description,
            tags,
            fitness_score,
            offspring_count,
            tick,
        });
    }

    #[allow(clippy::too_many_arguments)]
    /// Submits a seed (simulation config) to the marketplace.
    pub fn submit_seed(
        &self,
        id: Uuid,
        author: String,
        name: String,
        description: String,
        tags: String,
        config_json: String,
        avg_tick_time: f64,
        max_pop: u32,
        performance_summary: String,
    ) {
        let _ = self.sender.send(StorageCommand::SubmitSeed {
            id,
            author,
            name,
            description,
            tags,
            config_json,
            avg_tick_time,
            max_pop,
            performance_summary,
        });
    }

    /// Asynchronously queries genomes from marketplace.
    pub fn query_genomes_async(
        &self,
        limit: Option<usize>,
        sort_by: Option<String>,
    ) -> Option<mpsc::Receiver<Vec<GenomeRecord>>> {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(StorageCommand::QueryGenomes {
                limit,
                sort_by,
                reply_tx: tx,
            })
            .is_ok()
        {
            Some(rx)
        } else {
            None
        }
    }

    /// Asynchronously queries seeds from marketplace.
    pub fn query_seeds_async(
        &self,
        limit: Option<usize>,
        sort_by: Option<String>,
    ) -> Option<mpsc::Receiver<Vec<SeedRecord>>> {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(StorageCommand::QuerySeeds {
                limit,
                sort_by,
                reply_tx: tx,
            })
            .is_ok()
        {
            Some(rx)
        } else {
            None
        }
    }
}

fn init_db(conn: &mut Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS lineages (
            id TEXT PRIMARY KEY,
            start_tick INTEGER NOT NULL,
            civilization_level INTEGER NOT NULL DEFAULT 0,
            is_extinct BOOLEAN NOT NULL DEFAULT 0,
            best_genotype TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS fossils (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            lineage_id TEXT NOT NULL,
            tick INTEGER NOT NULL,
            genotype TEXT NOT NULL,
            legend_reason TEXT,
            FOREIGN KEY(lineage_id) REFERENCES lineages(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS world_snapshots (
            tick INTEGER PRIMARY KEY,
            pop_count INTEGER NOT NULL,
            carbon_level REAL NOT NULL,
            energy_total REAL NOT NULL,
            world_data BLOB NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_lineages_civ ON lineages(civilization_level)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_fossils_lineage ON fossils(lineage_id)",
        [],
    )?;

    // Phase 70: Marketplace tables for Global Registry
    conn.execute(
        "CREATE TABLE IF NOT EXISTS genome_submissions (
            id TEXT PRIMARY KEY,
            lineage_id TEXT,
            genotype TEXT NOT NULL,
            author TEXT,
            name TEXT NOT NULL,
            description TEXT,
            tags TEXT,
            fitness_score REAL,
            offspring_count INTEGER,
            tick INTEGER,
            downloads INTEGER DEFAULT 0,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(lineage_id) REFERENCES lineages(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS seed_submissions (
            id TEXT PRIMARY KEY,
            author TEXT,
            name TEXT NOT NULL,
            description TEXT,
            tags TEXT,
            config_json TEXT NOT NULL,
            avg_tick_time REAL,
            max_pop INTEGER,
            performance_summary TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            downloads INTEGER DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_genomes_fitness ON genome_submissions(fitness_score DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_seeds_pop ON seed_submissions(max_pop DESC)",
        [],
    )?;

    Ok(())
}
