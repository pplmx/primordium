//! # Primordium IO
//!
//! I/O and persistence layer for the Primordium simulation.
//!
//! This crate provides:
//! - Structured error handling with custom error types
//! - Serialization/deserialization (JSON, rkyv)
//! - Persistence and storage management
//! - Historical data logging
//! - Network communication protocols

/// Error types and result aliases for I/O operations
pub mod error;
/// Historical event logging, fossil records, and simulation history storage
pub mod history;
/// Lineage tracking, dynastic success metrics, and shared memory persistence
pub mod lineage;
/// P2P network communication protocols and message types
pub mod network;
/// Core persistence utilities and save file management
pub mod persistence;
/// In-memory and on-disk registries for entities and lineages
pub mod registry;
/// Validated serialization helpers for JSON and HexDNA formats
pub mod serialization;
/// Abstract storage backends including file-system and future database integrations
pub mod storage;

pub use error::{IoError, Result};
pub use serialization::{
    from_hex_dna, from_json, is_valid_hex_dna, read_json_file, to_hex_dna, to_json, to_json_pretty,
    validate_json, write_json_file,
};
