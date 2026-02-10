//! # Primordium Core
//!
//! The core simulation engine for Primordium - a hardware-coupled artificial life simulation.
//!
//! This crate contains the deterministic simulation logic, including:
//! - Neural network brains (NEAT-lite architecture)
//! - Entity lifecycle management
//! - Environmental systems (terrain, climate, resources)
//! - Social and ecological interactions
//! - Spatial indexing and performance optimizations
//! - Metrics collection and structured logging
//!
//! ## Architecture
//!
//! The simulation follows a data-oriented design with:
//! - **Component-based entities**: Physics, Metabolism, Intel components
//! - **System-based updates**: Perception, Action, Biological, Social systems
//! - **Parallel processing**: Rayon-powered parallelization for 10,000+ entities
//! - **Deterministic simulation**: Seeded RNG for reproducible results
//!
//! ## Example
//!
//! ```
//! use primordium_core::brain::BrainLogic;
//! use primordium_data::Brain;
//! use rand::SeedableRng;
//! use rand_chacha::ChaCha8Rng;
//!
//! // Create a new brain with default topology
//! let mut rng = ChaCha8Rng::seed_from_u64(42);
//! let brain = Brain::new_random_with_rng(&mut rng);
//!
//! // Process inputs to get outputs
//! let inputs = [0.5; 29];
//! let hidden = [0.0; 6];
//! let (outputs, _) = brain.forward(inputs, hidden);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

/// Neural network brain implementation with NEAT-lite topology
pub mod brain;
/// Configuration management for simulation parameters
pub mod config;
/// Environmental state management (climate, seasons, disasters)
pub mod environment;
/// Food resource management and spawning
pub mod food;
/// Historical event logging and fossil records
pub mod history;
/// Influence maps for collective intelligence and social coordination
pub mod influence;
/// Entity interaction handling (combat, bonding, sharing)
pub mod interaction;
/// Entity lifecycle management (birth, growth, death)
pub mod lifecycle;
/// Lineage tracking and registry for macroevolution
pub mod lineage_registry;
/// Ancestry tree construction and visualization
pub mod lineage_tree;
/// Performance metrics collection and logging
pub mod metrics;
/// Pathogen simulation with contagion and immunity
pub mod pathogen;
/// Pheromone grid for chemical communication
pub mod pheromone;
/// Hardware-coupled pressure system (CPU/RAM metrics)
pub mod pressure;
/// Entity snapshots for parallel processing
pub mod snapshot;
/// Sound propagation and acoustic communication
pub mod sound;
/// Spatial hashing for O(1) proximity queries
pub mod spatial_hash;
/// Core simulation systems (Perception, Action, Biological, Social)
pub mod systems;
/// Terrain grid with biome simulation
pub mod terrain;

pub use brain::{BrainLogic, GenotypeLogic};
pub use influence::{InfluenceGrid, InfluenceSource};
pub use metrics::{init_logging, Metrics};
pub use primordium_data::{Connection, Node, NodeType};
pub use terrain::TerrainLogic;
pub mod blockchain;
