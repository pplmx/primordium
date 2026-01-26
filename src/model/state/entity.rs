use crate::model::brain::Brain;
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status symbols for entity states
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityStatus {
    Starving,
    Larva,
    Juvenile,
    Infected,
    Sharing,
    Mating,
    Hunting,
    Foraging,
    Soldier,
    Bonded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Specialization {
    Soldier,
    Engineer,
    Provider,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Physics {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
    pub home_x: f64,
    pub home_y: f64,
    pub sensing_range: f64,
    pub max_speed: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    pub trophic_potential: f32,
    pub energy: f64,
    #[serde(skip)]
    pub prev_energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub birth_tick: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub lineage_id: Uuid,
    pub has_metamorphosed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub pathogen: Option<crate::model::state::pathogen::Pathogen>,
    pub infection_timer: u32,
    pub immunity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intel {
    pub genotype: Genotype,
    #[serde(skip)]
    pub last_hidden: [f32; 6],
    #[serde(skip)]
    pub last_aggression: f32,
    pub last_share_intent: f32,
    pub last_signal: f32,
    pub last_vocalization: f32,
    pub reputation: f32,
    pub rank: f32,
    pub bonded_to: Option<Uuid>,
    #[serde(skip)]
    pub last_inputs: [f32; 16],
    pub specialization: Option<Specialization>,
    pub spec_meters: std::collections::HashMap<Specialization, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genotype {
    pub brain: Brain,
    pub sensing_range: f64,
    pub max_speed: f64,
    pub max_energy: f64,
    pub lineage_id: Uuid,
    pub metabolic_niche: f32,
    pub trophic_potential: f32,
    pub reproductive_investment: f32,
    pub maturity_gene: f32,
    pub mate_preference: f32,
    pub pairing_bias: f32,
    pub specialization_bias: [f32; 3], // Soldier, Engineer, Provider
}

impl Genotype {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            brain: Brain::new_random(),
            sensing_range: 5.0,
            max_speed: 1.0,
            max_energy: 200.0,
            lineage_id: Uuid::new_v4(),
            metabolic_niche: rng.gen_range(0.0..1.0),
            trophic_potential: rng.gen_range(0.0..1.0),
            reproductive_investment: 0.5,
            maturity_gene: 1.0,
            mate_preference: rng.gen_range(0.0..1.0),
            pairing_bias: rng.gen_range(0.0..1.0),
            specialization_bias: [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
            ],
        }
    }

    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let gt = serde_json::from_slice(&bytes)?;
        Ok(gt)
    }

    pub fn distance(&self, other: &Genotype) -> f32 {
        let brain_dist = self.brain.genotype_distance(&other.brain);
        let trait_dist = (self.sensing_range - other.sensing_range).abs() as f32 / 5.0
            + (self.max_speed - other.max_speed).abs() as f32 / 1.0
            + (self.max_energy - other.max_energy).abs() as f32 / 100.0
            + (self.metabolic_niche - other.metabolic_niche).abs()
            + (self.trophic_potential - other.trophic_potential).abs()
            + (self.pairing_bias - other.pairing_bias).abs()
            + (self.specialization_bias[0] - other.specialization_bias[0]).abs()
            + (self.specialization_bias[1] - other.specialization_bias[1]).abs()
            + (self.specialization_bias[2] - other.specialization_bias[2]).abs();
        brain_dist + trait_dist
    }

    pub fn relatedness(&self, other: &Genotype) -> f32 {
        let dist = self.distance(other);
        (2.0f32).powf(-dist * 0.5)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub physics: Physics,
    pub metabolism: Metabolism,
    pub health: Health,
    pub intel: Intel,
}

impl Entity {
    pub fn new(x: f64, y: f64, tick: u64) -> Self {
        let mut rng = rand::thread_rng();
        let vx = rng.gen_range(-0.5..0.5);
        let vy = rng.gen_range(-0.5..0.5);
        let r = rng.gen_range(100..255);
        let g = rng.gen_range(100..255);
        let b = rng.gen_range(100..255);
        let genotype = Genotype::new_random();

        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            physics: Physics {
                x,
                y,
                vx,
                vy,
                r,
                g,
                b,
                symbol: '●',
                home_x: x,
                home_y: y,
                sensing_range: genotype.sensing_range,
                max_speed: genotype.max_speed,
            },
            metabolism: Metabolism {
                trophic_potential: genotype.trophic_potential,
                energy: 100.0,
                prev_energy: 100.0,
                max_energy: genotype.max_energy,
                peak_energy: 100.0,
                birth_tick: tick,
                generation: 1,
                offspring_count: 0,
                lineage_id: genotype.lineage_id,
                has_metamorphosed: false,
            },
            health: Health {
                pathogen: None,
                infection_timer: 0,
                immunity: rng.gen_range(0.0..0.3),
            },
            intel: Intel {
                genotype,
                last_hidden: [0.0; 6],
                last_aggression: 0.0,
                last_share_intent: 0.0,
                last_signal: 0.0,
                last_vocalization: 0.0,
                reputation: 1.0,
                rank: 0.5,
                bonded_to: None,
                last_inputs: [0.0; 16],
                specialization: None,
                spec_meters: std::collections::HashMap::new(),
            },
        }
    }

    pub fn color(&self) -> Color {
        let (r, g, b) = (self.physics.r, self.physics.g, self.physics.b);
        let signal = self.intel.last_signal;
        let factor = if signal > 0.0 {
            1.0 + signal as f64 * 0.5
        } else {
            1.0 + signal as f64 * 0.7
        };
        Color::Rgb(
            (r as f64 * factor).clamp(20.0, 255.0) as u8,
            (g as f64 * factor).clamp(20.0, 255.0) as u8,
            (b as f64 * factor).clamp(20.0, 255.0) as u8,
        )
    }

    pub fn status(&self, threshold: f32, current_tick: u64, maturity_age: u64) -> EntityStatus {
        let actual_maturity = (maturity_age as f32 * self.intel.genotype.maturity_gene) as u64;
        if self.metabolism.energy / self.metabolism.max_energy < 0.2 {
            EntityStatus::Starving
        } else if self.health.pathogen.is_some() {
            EntityStatus::Infected
        } else if !self.metabolism.has_metamorphosed {
            EntityStatus::Larva
        } else if (current_tick - self.metabolism.birth_tick) < actual_maturity {
            EntityStatus::Juvenile
        } else if self.intel.bonded_to.is_some() {
            EntityStatus::Bonded
        } else if self.intel.last_share_intent > threshold
            && self.metabolism.energy > self.metabolism.max_energy * 0.7
        {
            EntityStatus::Sharing
        } else if self.intel.rank > 0.8 && self.intel.last_aggression > threshold {
            EntityStatus::Soldier
        } else if self.intel.last_aggression > threshold {
            EntityStatus::Hunting
        } else {
            EntityStatus::Foraging
        }
    }

    pub fn symbol_for_status(&self, status: EntityStatus) -> char {
        match status {
            EntityStatus::Starving => '†',
            EntityStatus::Infected => '☣',
            EntityStatus::Larva => '⋯',
            EntityStatus::Juvenile => '◦',
            EntityStatus::Sharing => '♣',
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
            EntityStatus::Soldier => '⚔',
            EntityStatus::Bonded => '⚭',
        }
    }

    pub fn color_for_status(&self, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50),
            EntityStatus::Infected => Color::Rgb(154, 205, 50),
            EntityStatus::Larva => Color::Rgb(180, 180, 180),
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255),
            EntityStatus::Sharing => Color::Rgb(100, 200, 100),
            EntityStatus::Mating => Color::Rgb(255, 105, 180),
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),
            EntityStatus::Foraging => self.color(),
            EntityStatus::Soldier => Color::Red,
            EntityStatus::Bonded => Color::Rgb(255, 215, 0),
        }
    }

    pub fn is_mature(&self, current_tick: u64, maturity_age: u64) -> bool {
        let actual_maturity = (maturity_age as f32 * self.intel.genotype.maturity_gene) as u64;
        (current_tick - self.metabolism.birth_tick) >= actual_maturity
    }

    pub fn name(&self) -> String {
        let id_str = self.id.to_string();
        let bytes = id_str.as_bytes();
        let syllables = [
            "ae", "ba", "co", "da", "el", "fa", "go", "ha", "id", "jo", "ka", "lu", "ma", "na",
            "os", "pe", "qu", "ri", "sa", "tu", "vi", "wu", "xi", "yo", "ze",
        ];
        let prefix = [
            "Aethel", "Bel", "Cor", "Dag", "Eld", "Fin", "Grom", "Had", "Ith", "Jor", "Kael",
            "Luv", "Mor", "Nar", "Oth", "Pyr", "Quas", "Rhun", "Syl", "Tor", "Val", "Wun", "Xer",
            "Yor", "Zan",
        ];
        let p_idx = (bytes[0] as usize) % prefix.len();
        let s1_idx = (bytes[1] as usize) % syllables.len();
        let s2_idx = (bytes[2] as usize) % syllables.len();
        let tp = self.metabolism.trophic_potential;
        let role_prefix = if tp < 0.25 {
            "H-"
        } else if tp < 0.45 {
            "hO-"
        } else if tp < 0.55 {
            "O-"
        } else if tp < 0.75 {
            "cO-"
        } else {
            "C-"
        };
        format!(
            "{}{}{}{}-Gen{}",
            role_prefix,
            prefix[p_idx],
            syllables[s1_idx],
            syllables[s2_idx],
            self.metabolism.generation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_entity_new_has_valid_initial_state() {
        let entity = Entity::new(50.0, 25.0, 100);
        assert_eq!(entity.physics.x, 50.0);
        assert_eq!(entity.metabolism.generation, 1);
        assert_eq!(entity.metabolism.energy, 100.0);
    }
}
