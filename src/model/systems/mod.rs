//! Simulation systems module.
//!
//! This module contains the decomposed systems from the monolithic World::update.
//! Each system handles a specific aspect of the simulation:
//!
//! - `action`: Movement, velocity updates, and game mode effects
//! - `biological`: Infection processing, pathogen emergence, death handling
//! - `ecological`: Food spawning, feeding, and environmental sensing
//! - `social`: Predation, reproduction, and legendary archiving
//! - `environment`: Global environment logic (eras, seasons, disasters)
//! - `stats`: Population statistics and Hall of Fame updates
//! - `intel`: Neural network inference and brain evolution

pub mod action;
pub mod biological;
pub mod ecological;
pub mod environment;
pub mod intel;
pub mod interaction;
pub mod social;
pub mod stats;
