//! Pure data structures for the Primordium simulation.
//!
//! This crate contains serializable data types with no business logic,
//! shared between the simulation engine and WASM client.

pub mod data;

pub use data::entity::*;
pub use data::environment::*;
pub use data::genotype::*;
pub use data::terrain::*;
