use crate::model::brain::Brain;
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status symbols for entity states
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityStatus {
    /// Energy below 20% of maximum.
    Starving,
    /// Too young to reproduce (< maturity threshold).
    Juvenile,
    /// Carrying a pathogen infection.
    Infected,
    /// High energy, actively sharing with neighbors.
    Sharing,
    /// Above reproduction threshold, seeking mate.
    Mating,
    /// Brain aggression output > 0.5, hunting prey.
    Hunting,
    /// Default foraging behavior.
    Foraging,
    /// High Rank (>0.8) + Aggressive. Dedicated warrior.
    Soldier,
}

/// Physical properties: position, velocity, appearance, and home territory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Physics {
    /// X coordinate in world space.
    pub x: f64,
    /// Y coordinate in world space.
    pub y: f64,
    /// X velocity component.
    pub vx: f64,
    /// Y velocity component.
    pub vy: f64,
    /// Red color component for tribe identification.
    pub r: u8,
    /// Green color component for tribe identification.
    pub g: u8,
    /// Blue color component for tribe identification.
    pub b: u8,
    /// Display symbol in terminal renderer.
    pub symbol: char,
    /// Birth X coordinate (home territory center).
    pub home_x: f64,
    /// Birth Y coordinate (home territory center).
    pub home_y: f64,
    /// NEW: How far the entity can sense food/neighbors.
    pub sensing_range: f64,
    /// NEW: Maximum movement speed capability.
    pub max_speed: f64,
}

/// Metabolic state: energy, lifecycle, and reproduction tracking.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    /// Trophic preference (0.0 = Pure Herbivore, 1.0 = Pure Carnivore).
    pub trophic_potential: f32,
    /// Current energy level.
    pub energy: f64,
    /// Energy level from previous tick (for reinforcement learning).
    #[serde(skip)]
    pub prev_energy: f64,
    /// Maximum energy capacity (Stomach Size).
    pub max_energy: f64,
    /// Historical peak energy (fitness indicator).
    pub peak_energy: f64,
    /// Tick at which this entity was born.
    pub birth_tick: u64,
    /// Generation number (0 = original, 1+ = offspring).
    pub generation: u32,
    /// Number of offspring produced.
    pub offspring_count: u32,
    /// NEW: Ancestral lineage identifier.
    pub lineage_id: Uuid,
}

/// Health state: infection status and immunity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    /// Currently carried pathogen, if any.
    pub pathogen: Option<crate::model::state::pathogen::Pathogen>,
    /// Ticks remaining in current infection.
    pub infection_timer: u32,
    /// Immunity level (0.0 = vulnerable, 1.0 = immune).
    pub immunity: f32,
}

/// Intelligence component: neural network brain and decision state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intel {
    /// The inherited genotype (Physical traits + Brain).
    pub genotype: Genotype,
    /// Hidden layer activation from previous tick (recurrent memory).
    #[serde(skip)]
    pub last_hidden: [f32; 6],
    /// Last computed aggression output.
    #[serde(skip)]
    pub last_aggression: f32,
    pub last_share_intent: f32,
    pub last_signal: f32,
    /// NEW: Phase 48 - Vocalization output (0.0-1.0)
    pub last_vocalization: f32,
    /// NEW: Phase 46 - Social Reputation (0.0 to 1.0)
    pub reputation: f32,
    /// NEW: Phase 49 - Social Rank (0.0 to 1.0, 1.0 = Alpha)
    pub rank: f32,
    /// NEW: Phase 47 - Last inputs for learning (Now includes Hearing)
    #[serde(skip)]
    pub last_inputs: [f32; 15],
}

/// The full genetic payload of an entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genotype {
    pub brain: Brain,
    pub sensing_range: f64,
    pub max_speed: f64,
    pub max_energy: f64,
    /// NEW: Ancestral lineage tracking.
    pub lineage_id: Uuid,
    /// NEW: Metabolic niche (0.0 = Green expert, 1.0 = Blue expert)
    pub metabolic_niche: f32,
    /// NEW: Trophic Potential (0.0 = Herbivore, 1.0 = Carnivore)
    pub trophic_potential: f32,
    /// NEW: Ratio of energy given to child (0.1 to 0.9)
    pub reproductive_investment: f32,
    /// NEW: Maturity age modifier (ticks = maturity_age * maturity_gene)
    pub maturity_gene: f32,
    /// NEW: Sexual Selection Preference (0.0 = Prefers Herbivores, 1.0 = Prefers Carnivores)
    pub mate_preference: f32,
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
            + (self.trophic_potential - other.trophic_potential).abs();

        brain_dist + trait_dist
    }

    /// NEW: Phase 46 - Coefficient of relatedness (0.0 to 1.0)
    /// Derived from genetic distance: r = 2^(-dist * 0.5)
    pub fn relatedness(&self, other: &Genotype) -> f32 {
        let dist = self.distance(other);
        (2.0f32).powf(-dist * 0.5)
    }
}

/// A living entity in the simulation world.
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
                reputation: 1.0, // Start with clean slate
                rank: 0.5,       // Default rank (0.0 = Omega, 1.0 = Alpha)
                last_inputs: [0.0; 15],
            },
        }
    }

    pub fn color(&self) -> Color {
        let (r, g, b) = (self.physics.r, self.physics.g, self.physics.b);
        let signal = self.intel.last_signal;
        if signal > 0.0 {
            let factor = 1.0 + signal as f64 * 0.5;
            Color::Rgb(
                (r as f64 * factor).min(255.0) as u8,
                (g as f64 * factor).min(255.0) as u8,
                (b as f64 * factor).min(255.0) as u8,
            )
        } else {
            let factor = 1.0 + signal as f64 * 0.7;
            Color::Rgb(
                (r as f64 * factor).max(20.0) as u8,
                (g as f64 * factor).max(20.0) as u8,
                (b as f64 * factor).max(20.0) as u8,
            )
        }
    }

    pub fn status(
        &self,
        _reproduction_threshold: f64,
        current_tick: u64,
        maturity_age: u64,
    ) -> EntityStatus {
        let actual_maturity = (maturity_age as f32 * self.intel.genotype.maturity_gene) as u64;
        if self.metabolism.energy / self.metabolism.max_energy < 0.2 {
            EntityStatus::Starving
        } else if self.health.pathogen.is_some() {
            EntityStatus::Infected
        } else if (current_tick - self.metabolism.birth_tick) < actual_maturity {
            EntityStatus::Juvenile
        } else if self.intel.last_share_intent > 0.5
            && self.metabolism.energy > self.metabolism.max_energy * 0.7
        {
            EntityStatus::Sharing
        } else if self.intel.rank > 0.8 && self.intel.last_aggression > 0.5 {
            EntityStatus::Soldier
        } else if self.intel.last_aggression > 0.5 {
            EntityStatus::Hunting
        } else {
            EntityStatus::Foraging
        }
    }

    pub fn symbol_for_status(&self, status: EntityStatus) -> char {
        match status {
            EntityStatus::Starving => '†',
            EntityStatus::Infected => '☣',
            EntityStatus::Juvenile => '◦',
            EntityStatus::Sharing => '♣',
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
            EntityStatus::Soldier => '⚔',
        }
    }

    pub fn color_for_status(&self, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50),
            EntityStatus::Infected => Color::Rgb(154, 205, 50),
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255),
            EntityStatus::Sharing => Color::Rgb(100, 200, 100),
            EntityStatus::Mating => Color::Rgb(255, 105, 180),
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),
            EntityStatus::Foraging => self.color(),
            EntityStatus::Soldier => Color::Red,
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
            "H-" // Specialist Herbivore
        } else if tp < 0.45 {
            "hO-" // Herbivore-leaning Omnivore
        } else if tp < 0.55 {
            "O-" // True Omnivore
        } else if tp < 0.75 {
            "cO-" // Carnivore-leaning Omnivore
        } else {
            "C-" // Specialist Carnivore
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
        assert_eq!(entity.physics.y, 25.0);
        assert_eq!(entity.metabolism.generation, 1);
        assert_eq!(entity.metabolism.energy, 100.0);
    }
}
