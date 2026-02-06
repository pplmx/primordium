//! Configuration management for simulation parameters.
//!
//! This module provides strongly-typed configuration structures that map to
//! the `config.toml` file. All simulation parameters can be customized through
//! this configuration system.
//!
//! ## Configuration Hierarchy
//!
//! 1. Default values (hardcoded in `Default` impl)
//! 2. `config.toml` file (overrides defaults)
//! 3. Environment variables (future enhancement)
//!
//! ## Example `config.toml`
//!
//! ```toml
//! [world]
//! width = 100
//! height = 50
//! initial_population = 100
//! seed = 42
//! deterministic = true
//!
//! [metabolism]
//! base_move_cost = 1.0
//! reproduction_threshold = 150.0
//!
//! [evolution]
//! mutation_rate = 0.1
//! ```

use serde::{Deserialize, Serialize};
use std::fs;

/// World-level simulation configuration.
///
/// Defines the fundamental parameters of the simulation world including
/// dimensions, initial population, and hardware-coupled environmental triggers.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub initial_population: usize,
    pub initial_food: usize,
    pub max_food: usize,
    pub disaster_chance: f32,
    pub heat_wave_cpu: f32,
    pub ice_age_cpu: f32,
    pub abundance_ram: f32,
    pub apex_fitness_req: f64,
    pub seed: Option<u64>,
    pub deterministic: bool,
    pub fossil_interval: u64,
    pub power_grid_interval: u64,
}

/// Entity metabolism and energy management configuration.
///
/// Controls energy costs, consumption rates, and life-cycle thresholds
/// that govern entity survival and reproduction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetabolismConfig {
    pub base_move_cost: f64,
    pub base_idle_cost: f64,
    pub reproduction_threshold: f64,
    pub food_value: f64,
    pub maturity_age: u64,
    pub birth_energy_multiplier: f64,
    pub oxygen_consumption_rate: f64,
    pub adult_energy_multiplier: f64,
    pub adult_speed_multiplier: f64,
    pub adult_sensing_multiplier: f64,
    pub metamorphosis_trigger_maturity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvolutionConfig {
    pub mutation_rate: f32,
    pub mutation_amount: f32,
    pub drift_rate: f32,
    pub drift_amount: f32,
    pub speciation_rate: f32,
    pub speciation_threshold: f32,
    pub population_aware: bool,
    pub bottleneck_threshold: usize,
    pub stasis_threshold: usize,
    pub crowding_threshold: f32,
    pub crowding_normalization: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Standard,
    Cooperative,
    BattleRoyale,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrainConfig {
    pub hidden_node_cost: f64,
    pub connection_cost: f64,
    pub activation_threshold: f32,
    pub learning_rate_max: f32,
    pub learning_reinforcement: f32,
    pub coupling_spring_constant: f64,
    pub alpha_following_force: f64,
    pub pruning_threshold: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocialConfig {
    pub rank_weights: [f32; 4],
    pub soldier_damage_mult: f64,
    pub war_zone_mult: f64,
    pub sharing_threshold: f32,
    pub sharing_fraction: f64,
    pub bond_break_dist: f64,
    pub relatedness_half_life: f64,
    pub territorial_range: f64,
    pub tribe_color_threshold: i32,
    pub age_rank_normalization: f32,
    pub offspring_rank_normalization: f32,
    pub specialization_threshold: f32,
    pub silo_energy_capacity: f32,
    pub outpost_energy_capacity: f32,
    pub aggression_threshold: f32,
    pub energy_sharing_low_threshold: f32,
    pub defense_per_ally_reduction: f64,
    pub min_defense_multiplier: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerraformConfig {
    pub dig_cost: f64,
    pub build_cost: f64,
    pub canal_cost: f64,
    pub engineer_discount: f64,
    pub nest_energy_req: f64,
    pub dig_oxygen_cost: f64,
    pub build_oxygen_cost: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EcosystemConfig {
    pub carbon_emission_rate: f64,
    pub sequestration_rate: f64,
    pub oxygen_consumption_unit: f64,
    pub soil_depletion_unit: f32,
    pub corpse_fertility_mult: f32,
    pub base_spawn_chance: f32,
    pub nutrient_niche_multiplier: f32,
    pub predation_energy_gain_fraction: f64,
    pub predation_competition_scale: f64,
    pub predation_min_efficiency: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub world: WorldConfig,
    pub metabolism: MetabolismConfig,
    pub evolution: EvolutionConfig,
    pub brain: BrainConfig,
    pub social: SocialConfig,
    pub terraform: TerraformConfig,
    pub ecosystem: EcosystemConfig,
    pub target_fps: u64,
    pub game_mode: GameMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            world: WorldConfig {
                width: 100,
                height: 50,
                initial_population: 100,
                initial_food: 30,
                max_food: 50,
                disaster_chance: 0.01,
                heat_wave_cpu: 80.0,
                ice_age_cpu: 10.0,
                abundance_ram: 40.0,
                apex_fitness_req: 8000.0,
                seed: None,
                deterministic: false,
                fossil_interval: 1000,
                power_grid_interval: 10,
            },
            metabolism: MetabolismConfig {
                base_move_cost: 1.0,
                base_idle_cost: 0.5,
                reproduction_threshold: 150.0,
                food_value: 50.0,
                maturity_age: 150,
                birth_energy_multiplier: 1.2,
                oxygen_consumption_rate: 0.005,
                adult_energy_multiplier: 1.5,
                adult_speed_multiplier: 1.2,
                adult_sensing_multiplier: 1.2,
                metamorphosis_trigger_maturity: 0.8,
            },
            evolution: EvolutionConfig {
                mutation_rate: 0.1,
                mutation_amount: 0.2,
                drift_rate: 0.01,
                drift_amount: 0.5,
                speciation_rate: 0.02,
                speciation_threshold: 5.0,
                population_aware: true,
                bottleneck_threshold: 20,
                stasis_threshold: 500,
                crowding_threshold: 0.8,
                crowding_normalization: 10.0,
            },
            brain: BrainConfig {
                hidden_node_cost: 0.02,
                connection_cost: 0.005,
                activation_threshold: 0.5,
                learning_rate_max: 0.5,
                learning_reinforcement: 10.0,
                coupling_spring_constant: 0.05,
                alpha_following_force: 0.02,
                pruning_threshold: 0.01,
            },
            social: SocialConfig {
                rank_weights: [0.3, 0.3, 0.1, 0.3],
                soldier_damage_mult: 1.5,
                war_zone_mult: 2.0,
                sharing_threshold: 0.5,
                sharing_fraction: 0.05,
                bond_break_dist: 20.0,
                relatedness_half_life: 0.5,
                territorial_range: 8.0,
                tribe_color_threshold: 60,
                age_rank_normalization: 2000.0,
                offspring_rank_normalization: 20.0,
                specialization_threshold: 100.0,
                silo_energy_capacity: 5000.0,
                outpost_energy_capacity: 1000.0,
                aggression_threshold: 0.5,
                energy_sharing_low_threshold: 0.5,
                defense_per_ally_reduction: 0.15,
                min_defense_multiplier: 0.4,
            },
            terraform: TerraformConfig {
                dig_cost: 10.0,
                build_cost: 15.0,
                canal_cost: 30.0,
                engineer_discount: 0.5,
                nest_energy_req: 150.0,
                dig_oxygen_cost: 0.02,
                build_oxygen_cost: 0.03,
            },
            ecosystem: EcosystemConfig {
                carbon_emission_rate: 0.01,
                sequestration_rate: 0.00001,
                oxygen_consumption_unit: 0.05,
                soil_depletion_unit: 0.01,
                corpse_fertility_mult: 0.1,
                base_spawn_chance: 0.0083,
                nutrient_niche_multiplier: 1.5,
                predation_energy_gain_fraction: 0.5,
                predation_competition_scale: 10000.0,
                predation_min_efficiency: 0.5,
            },
            target_fps: 60,
            game_mode: GameMode::Standard,
        }
    }
}

impl AppConfig {
    #[must_use]
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.toml") {
            match toml::from_str(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Warning: Failed to parse config.toml: {}", e);
                    eprintln!("Falling back to default configuration.");
                }
            }
        }
        let default = Self::default();
        // Don't overwrite existing malformed config to preserve user data
        if !std::path::Path::new("config.toml").exists() {
            if let Ok(toml_str) = toml::to_string(&default) {
                let _ = fs::write("config.toml", toml_str);
            }
        }
        default
    }

    #[must_use]
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", self.metabolism).as_bytes());
        hasher.update(format!("{:?}", self.evolution).as_bytes());
        hasher.update(format!("{:?}", self.brain).as_bytes());
        hasher.update(format!("{:?}", self.social).as_bytes());
        hasher.update(format!("{:?}", self.terraform).as_bytes());
        hasher.update(format!("{:?}", self.ecosystem).as_bytes());
        hex::encode(hasher.finalize())
    }
}
