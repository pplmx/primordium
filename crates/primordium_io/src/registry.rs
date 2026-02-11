use anyhow::Result;
use primordium_core::lineage_registry::LineageRegistry;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

/// Trait for adding persistence capabilities to [`LineageRegistry`].
pub trait LineagePersistence {
    /// Saves the lineage registry to a JSON file.
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    /// Loads a lineage registry from a file.
    fn load<P: AsRef<Path>>(path: P) -> Result<LineageRegistry>;
}

impl LineagePersistence for LineageRegistry {
    /// Atomically saves the registry to disk using a temporary file.
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let tmp_path = path.with_extension("tmp");
        {
            let file = File::create(&tmp_path)?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, self)?;
        }
        std::fs::rename(tmp_path, path)?;
        Ok(())
    }

    /// Loads the registry from disk, returning a default instance if the file doesn't exist.
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
