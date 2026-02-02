use anyhow::Result;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::ser::Serializer;
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
    T::Archived: Deserialize<T, SharedDeserializeMap>
        + for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
    P: AsRef<Path>,
{
    let bytes = std::fs::read(path)?;
    let archived = rkyv::check_archived_root::<T>(&bytes)
        .map_err(|e| anyhow::anyhow!("Rkyv validation error: {:?}", e))?;
    let mut deserializer = SharedDeserializeMap::default();
    let deserialized: T = archived
        .deserialize(&mut deserializer)
        .map_err(|e| anyhow::anyhow!("Rkyv deserialization error: {:?}", e))?;
    Ok(deserialized)
}
