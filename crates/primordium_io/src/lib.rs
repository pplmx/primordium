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

/// Error types for I/O operations
pub mod error;
/// Historical event logging and fossil records
pub mod history;
/// Lineage tracking and persistence
pub mod lineage;
/// Network communication protocols
pub mod network;
/// Serialization and persistence utilities
pub mod persistence;
/// Entity and lineage registries
pub mod registry;
/// Robust serialization utilities with validation
pub mod serialization;
/// Storage backends and file management
pub mod storage;

pub use error::{IoError, Result};
pub use serialization::{
    from_hex_dna, from_json, is_valid_hex_dna, read_json_file, to_hex_dna, to_json, to_json_pretty,
    validate_json, write_json_file,
};
