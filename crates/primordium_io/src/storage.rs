use anyhow::Result;
use primordium_core::lineage_registry::LineageRegistry;
use primordium_data::FossilRegistry;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::thread;
use uuid::Uuid;

pub enum StorageCommand {
    UpsertLineage {
        id: Uuid,
        start_tick: u64,
        civilization_level: u32,
        is_extinct: bool,
        best_genotype: String,
    },
    RecordFossil {
        lineage_id: Uuid,
        tick: u64,
        genotype: String,
        reason: String,
    },
    SaveSnapshot {
        tick: u64,
        pop_count: u32,
        carbon_level: f64,
        energy_total: f64,
        world_data: Vec<u8>,
    },
    BatchSyncLineages(LineageRegistry),
    BatchSyncFossils(FossilRegistry),
    QueryFossils(Uuid, Sender<Vec<(u64, String)>>),
    QueryHallOfFame(Sender<Vec<(Uuid, u32, bool)>>),
    Stop,
}

pub struct StorageManager {
    sender: Sender<StorageCommand>,
}

impl StorageManager {
    pub fn clone_sender(&self) -> Sender<StorageCommand> {
        self.sender.clone()
    }

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
                    StorageCommand::Stop => break,
                }
            }
        });

        Ok(Self { sender: tx })
    }

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

    pub fn record_fossil(&self, lineage_id: Uuid, tick: u64, genotype: String, reason: String) {
        let _ = self.sender.send(StorageCommand::RecordFossil {
            lineage_id,
            tick,
            genotype,
            reason,
        });
    }

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

    pub fn sync_lineages(&self, registry: LineageRegistry) {
        let _ = self
            .sender
            .send(StorageCommand::BatchSyncLineages(registry));
    }

    pub fn sync_fossils(&self, registry: FossilRegistry) {
        let _ = self.sender.send(StorageCommand::BatchSyncFossils(registry));
    }

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

    Ok(())
}
