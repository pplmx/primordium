use rand::Rng;
use serde::{Deserialize, Serialize};

/// Represents a strain of pathogen in the digital ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pathogen {
    pub id: uuid::Uuid,
    pub lethality: f32,                              // Energy drain per tick
    pub transmission: f32,                           // Chance to spread to neighbors
    pub duration: u32,                               // How many ticks it lasts
    pub virulence: f32,                              // Initial strength
    pub behavior_manipulation: Option<(usize, f32)>, // (Output index, offset)
}

impl Default for Pathogen {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            lethality: 0.2,
            transmission: 0.05,
            duration: 500,
            virulence: 1.0,
            behavior_manipulation: None,
        }
    }
}

impl Pathogen {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let manipulation = if rng.gen_bool(0.3) {
            // 30% chance of being a behavioral parasite
            Some((rng.gen_range(22..33), 1.0))
        } else {
            None
        };
        Self {
            id: uuid::Uuid::new_v4(),
            lethality: rng.gen_range(0.05..0.5),
            transmission: rng.gen_range(0.01..0.1),
            duration: rng.gen_range(200..800),
            virulence: rng.gen_range(0.5..1.5),
            behavior_manipulation: manipulation,
        }
    }

    /// Mutate pathogen traits (evolution of the virus itself)
    pub fn mutate(&mut self) {
        let mut rng = rand::thread_rng();
        self.lethality = (self.lethality + rng.gen_range(-0.02..0.02)).clamp(0.01, 1.0);
        self.transmission = (self.transmission + rng.gen_range(-0.01..0.01)).clamp(0.005, 0.2);
        if rng.gen_bool(0.1) {
            // Pathogen can change its manipulation target
            self.behavior_manipulation = Some((rng.gen_range(22..33), 1.0));
        }
    }
}
