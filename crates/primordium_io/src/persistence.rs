use anyhow::Result;
use rkyv::ser::{serializers::AllocSerializer, Serializer};
use rkyv::{Archive, Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn save_rkyv<T, P>(data: &T, path: P) -> Result<()>
where
    T: Serialize<AllocSerializer<4096>>,
    T: Archive,
    P: AsRef<Path>,
{
    let mut serializer = AllocSerializer::<4096>::default();
    serializer
        .serialize_value(data)
        .map_err(|e| anyhow::anyhow!("Rkyv serialization error: {:?}", e))?;
    let bytes = serializer.into_serializer().into_inner();
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

pub fn load_rkyv<T, P>(path: P) -> Result<T>
where
    T: Archive,
    T::Archived: Deserialize<T, rkyv::Infallible>,
    P: AsRef<Path>,
{
    let bytes = std::fs::read(path)?;
    // Unsafe access for now, assuming file integrity.
    // In production we should use check_archived_root with validation.
    let archived = unsafe { rkyv::archived_root::<T>(&bytes) };
    let deserialized: T = archived.deserialize(&mut rkyv::Infallible).unwrap();
    Ok(deserialized)
}
