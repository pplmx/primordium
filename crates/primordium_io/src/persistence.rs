use crate::error::{IoError, Result};
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::ser::Serializer;
use rkyv::util::AlignedVec;
use rkyv::{Archive, Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

/// Saves data to a file using rkyv serialization.
///
/// # Parameters
/// - `data`: The data to serialize
/// - `path`: The file path to save to
///
/// # Errors
/// Returns `IoError` if serialization fails or file cannot be written.
pub fn save_rkyv<T, P>(data: &T, path: P) -> Result<()>
where
    T: Serialize<AllocSerializer<4096>>,
    T: Archive,
    P: AsRef<Path>,
{
    let mut serializer = AllocSerializer::<4096>::default();
    serializer
        .serialize_value(data)
        .map_err(|e| IoError::rkyv(format!("serialization failed: {:?}", e)))?;
    let bytes = serializer.into_serializer().into_inner();
    let mut file = File::create(&path).map_err(|e| {
        IoError::FileSystem(e).with_context(format!("creating file: {:?}", path.as_ref()))
    })?;
    file.write_all(&bytes)
        .map_err(|e| IoError::FileSystem(e).with_context("writing data"))?;
    Ok(())
}

/// Loads data from a file using rkyv deserialization.
///
/// # Parameters
/// - `path`: The file path to load from
///
/// # Errors
/// Returns `IoError` if file cannot be read, validation fails, or deserialization fails.
pub fn load_rkyv<T, P>(path: P) -> Result<T>
where
    T: Archive,
    T::Archived: Deserialize<T, SharedDeserializeMap>
        + for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
    P: AsRef<Path>,
{
    let mut file = File::open(&path).map_err(|e| {
        IoError::FileSystem(e).with_context(format!("opening file: {:?}", path.as_ref()))
    })?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| {
        IoError::FileSystem(e).with_context(format!("reading file: {:?}", path.as_ref()))
    })?;

    let mut aligned = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(&bytes);

    // Ensure the AlignedVec has at least 8-byte alignment
    // (AlignedVec usually has 16, but let's be sure or check if it's the culprit)

    let archived = rkyv::check_archived_root::<T>(&aligned)
        .map_err(|e| IoError::rkyv(format!("validation failed: {:?}", e)))?;
    let mut deserializer = SharedDeserializeMap::default();
    let deserialized: T = archived
        .deserialize(&mut deserializer)
        .map_err(|e| IoError::rkyv(format!("deserialization failed: {:?}", e)))?;
    Ok(deserialized)
}
