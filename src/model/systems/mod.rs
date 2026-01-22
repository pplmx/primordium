//! Simulation systems module.
//!
//! This module contains the decomposed systems from the monolithic World::update.
//! Each system handles a specific aspect of the simulation:
//!
//! - `action`: Movement, velocity updates, and game mode effects
//! - `biological`: Infection processing, pathogen emergence, death handling
//! - `ecological`: Food spawning, feeding, and environmental sensing
//! - `social`: Predation, reproduction, and legendary archiving

pub mod action;
pub mod biological;
pub mod ecological;
pub mod social;
