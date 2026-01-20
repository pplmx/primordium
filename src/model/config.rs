use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub width: u16,
    pub height: u16,
    pub initial_population: usize,
    pub initial_food: usize,
    pub max_food: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetabolismConfig {
    pub base_move_cost: f64,
    pub base_idle_cost: f64,
    pub reproduction_threshold: f64,
    pub food_value: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EvolutionConfig {
    pub mutation_rate: f32,
    pub mutation_amount: f32,
    pub drift_rate: f32,
    pub drift_amount: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub world: WorldConfig,
    pub metabolism: MetabolismConfig,
    pub evolution: EvolutionConfig,
    pub target_fps: u64,
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
            },
            metabolism: MetabolismConfig {
                base_move_cost: 1.0,
                base_idle_cost: 0.5,
                reproduction_threshold: 150.0,
                food_value: 50.0,
            },
            evolution: EvolutionConfig {
                mutation_rate: 0.1,
                mutation_amount: 0.2,
                drift_rate: 0.01,
                drift_amount: 0.5,
            },
            target_fps: 60,
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
        let _ = fs::write("config.toml", toml::to_string(&default).unwrap());
        default
    }
}
