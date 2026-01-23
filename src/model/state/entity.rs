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
}

/// Dietary role determining food sources and aggression patterns.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityRole {
    /// Eats plants, generally low aggression.
    Herbivore,
    /// Eats other entities, high aggression potential.
    Carnivore,
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
    /// Dietary role (Herbivore or Carnivore).
    pub role: EntityRole,
    /// Current energy level.
    pub energy: f64,
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
    /// Last computed sharing intent output.
    #[serde(skip)]
    pub last_share_intent: f32,
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
}

impl Genotype {
    pub fn new_random() -> Self {
        Self {
            brain: Brain::new_random(),
            sensing_range: 5.0,
            max_speed: 1.0,
            max_energy: 200.0,
            lineage_id: Uuid::new_v4(),
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
}

/// A living entity in the simulation world.
///
/// Each entity is composed of four components following ECS-like patterns:
/// - [`Physics`]: Position, velocity, and appearance
/// - [`Metabolism`]: Energy and lifecycle state
/// - [`Health`]: Infection and immunity
/// - [`Intel`]: Neural network brain and decision outputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier.
    pub id: Uuid,
    /// Parent entity ID (None for original population).
    pub parent_id: Option<Uuid>,
    /// Physical properties.
    pub physics: Physics,
    /// Metabolic state.
    pub metabolism: Metabolism,
    /// Health state.
    pub health: Health,
    /// Intelligence component.
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

        let role = if rng.gen_bool(0.8) {
            EntityRole::Herbivore
        } else {
            EntityRole::Carnivore
        };

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
                role,
                energy: 100.0,
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
            },
        }
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.physics.r, self.physics.g, self.physics.b)
    }

    pub fn status(
        &self,
        reproduction_threshold: f64,
        current_tick: u64,
        maturity_age: u64,
    ) -> EntityStatus {
        if self.metabolism.energy / self.metabolism.max_energy < 0.2 {
            EntityStatus::Starving
        } else if self.health.pathogen.is_some() {
            EntityStatus::Infected
        } else if (current_tick - self.metabolism.birth_tick) < maturity_age {
            EntityStatus::Juvenile
        } else if self.intel.last_share_intent > 0.5
            && self.metabolism.energy > self.metabolism.max_energy * 0.7
        {
            EntityStatus::Sharing
        } else if self.intel.last_aggression > 0.5 {
            EntityStatus::Hunting
        } else if self.metabolism.energy > reproduction_threshold {
            EntityStatus::Mating
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
        }
    }

    pub fn is_mature(&self, current_tick: u64, maturity_age: u64) -> bool {
        (current_tick - self.metabolism.birth_tick) >= maturity_age
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

        let role_prefix = match self.metabolism.role {
            EntityRole::Herbivore => "H-",
            EntityRole::Carnivore => "C-",
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
        assert_eq!(entity.metabolism.birth_tick, 100);
        assert_eq!(entity.metabolism.generation, 1);
        assert_eq!(entity.metabolism.energy, 100.0);
        assert_eq!(entity.metabolism.max_energy, 200.0);
        assert_eq!(entity.metabolism.offspring_count, 0);
        assert!(entity.parent_id.is_none());
    }

    #[test]
    fn test_entity_name_is_deterministic() {
        let entity = Entity::new(0.0, 0.0, 0);
        let name1 = entity.name();
        let name2 = entity.name();

        assert_eq!(name1, name2, "Same entity should have same name");
        assert!(name1.contains("-Gen"), "Name should contain generation");
    }
}
