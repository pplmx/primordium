use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub initial_population: usize,
    pub initial_food: usize,
    pub max_food: usize,
    /// NEW: Phase 47 - Disaster trigger probability (0.0 to 1.0)
    pub disaster_chance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetabolismConfig {
    pub base_move_cost: f64,
    pub base_idle_cost: f64,
    pub reproduction_threshold: f64,
    pub food_value: f64,
    pub maturity_age: u64, // NEW: Age in ticks before an entity can reproduce
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
pub struct AppConfig {
    pub world: WorldConfig,
    pub metabolism: MetabolismConfig,
    pub evolution: EvolutionConfig,
    pub target_fps: u64,
    pub game_mode: GameMode, // NEW
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
                disaster_chance: 0.01, // Default 1%
            },
            metabolism: MetabolismConfig {
                base_move_cost: 1.0,
                base_idle_cost: 0.5,
                reproduction_threshold: 150.0,
                food_value: 50.0,
                maturity_age: 150, // Default 150 ticks to reach adulthood
            },
            evolution: EvolutionConfig {
                mutation_rate: 0.1,
                mutation_amount: 0.2,
                drift_rate: 0.01,
                drift_amount: 0.5,
                speciation_rate: 0.02,
                speciation_threshold: 5.0, // Default threshold for new lineage
                population_aware: true,
                bottleneck_threshold: 20,
                stasis_threshold: 500,
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
        // Create default config file if missing
        if let Ok(toml_str) = toml::to_string(&default) {
            let _ = fs::write("config.toml", toml_str);
        }
        default
    }

    /// NEW: Phase 45 - Unique hash of physics/evolution constants
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        // Hash key constants that MUST match for stable migration
        hasher.update(format!("{:?}", self.metabolism).as_bytes());
        hasher.update(format!("{:?}", self.evolution).as_bytes());
        hex::encode(hasher.finalize())
    }
}
