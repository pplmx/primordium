use anyhow::Result;
use primordium_core::lineage_registry::LineageRegistry;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub trait LineagePersistence {
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn load<P: AsRef<Path>>(path: P) -> Result<LineageRegistry>;
}

impl LineagePersistence for LineageRegistry {
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let registry = serde_json::from_reader(reader)?;
        Ok(registry)
    }
}
