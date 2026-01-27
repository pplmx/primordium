use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub initial_population: usize,
    pub initial_food: usize,
    pub max_food: usize,
    pub disaster_chance: f32,
    pub heat_wave_cpu: f32,    // Default 80.0
    pub ice_age_cpu: f32,      // Default 10.0
    pub abundance_ram: f32,    // Default 40.0
    pub apex_fitness_req: f64, // Default 8000.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetabolismConfig {
    pub base_move_cost: f64,
    pub base_idle_cost: f64,
    pub reproduction_threshold: f64,
    pub food_value: f64,
    pub maturity_age: u64, // NEW: Age in ticks before an entity can reproduce
    pub birth_energy_multiplier: f64, // NEW: Multiplier for initial energy
    /// NEW: Phase 56 - Base oxygen consumption per entity per tick
    pub oxygen_consumption_rate: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvolutionConfig {
    pub mutation_rate: f32,
    pub mutation_amount: f32,
    pub drift_rate: f32,
    pub drift_amount: f32,
    pub speciation_rate: f32, // NEW: Chance for offspring to flip trophic role
    pub speciation_threshold: f32, // NEW: Genetic distance threshold for automatic speciation
    /// NEW: Phase 39 - Scaling mutation based on population density
    pub population_aware: bool,
    pub bottleneck_threshold: usize, // e.g. 20
    pub stasis_threshold: usize,     // e.g. 500
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Standard,
    Cooperative,
    BattleRoyale,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrainConfig {
    pub hidden_node_cost: f64,       // Default 0.02
    pub connection_cost: f64,        // Default 0.005
    pub activation_threshold: f32,   // Default 0.5
    pub learning_rate_max: f32,      // Default 0.5
    pub learning_reinforcement: f32, // Default 10.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocialConfig {
    pub rank_weights: [f32; 4],     // [Energy, Age, Offspring, Reputation]
    pub soldier_damage_mult: f64,   // Default 1.5
    pub war_zone_mult: f64,         // Default 2.0
    pub sharing_threshold: f32,     // Default 0.5
    pub sharing_fraction: f64,      // Default 0.05
    pub bond_break_dist: f64,       // Default 20.0
    pub relatedness_half_life: f64, // Default 0.5
    pub territorial_range: f64,     // Default 8.0
    pub tribe_color_threshold: i32, // Default 60
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerraformConfig {
    pub dig_cost: f64,          // Default 10.0
    pub build_cost: f64,        // Default 15.0
    pub canal_cost: f64,        // Default 30.0
    pub engineer_discount: f64, // Default 0.5
    pub nest_energy_req: f64,   // Default 150.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EcosystemConfig {
    pub carbon_emission_rate: f64,    // Default 0.01
    pub sequestration_rate: f64,      // Default 0.00001
    pub oxygen_consumption_unit: f64, // Default 0.05
    pub soil_depletion_unit: f32,     // Default 0.01
    pub corpse_fertility_mult: f32,   // Default 0.1
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
            },
            metabolism: MetabolismConfig {
                base_move_cost: 1.0,
                base_idle_cost: 0.5,
                reproduction_threshold: 150.0,
                food_value: 50.0,
                maturity_age: 150,
                birth_energy_multiplier: 1.2,
                oxygen_consumption_rate: 0.005,
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
            },
            brain: BrainConfig {
                hidden_node_cost: 0.02,
                connection_cost: 0.005,
                activation_threshold: 0.5,
                learning_rate_max: 0.5,
                learning_reinforcement: 10.0,
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
            },
            terraform: TerraformConfig {
                dig_cost: 10.0,
                build_cost: 15.0,
                canal_cost: 30.0,
                engineer_discount: 0.5,
                nest_energy_req: 150.0,
            },
            ecosystem: EcosystemConfig {
                carbon_emission_rate: 0.01,
                sequestration_rate: 0.00001,
                oxygen_consumption_unit: 0.05,
                soil_depletion_unit: 0.01,
                corpse_fertility_mult: 0.1,
            },
            target_fps: 60,
            game_mode: GameMode::Standard,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        let default = Self::default();
        if let Ok(toml_str) = toml::to_string(&default) {
            let _ = fs::write("config.toml", toml_str);
        }
        default
    }

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
