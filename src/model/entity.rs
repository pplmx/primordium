use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status symbols for entity states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityStatus {
    Starving, // < 20% energy
    Juvenile, // Too young to reproduce
    Infected, // Carrying a pathogen [NEW]
    Sharing,  // High energy, sharing with neighbors
    Mating,   // > reproduction threshold
    Hunting,  // brain aggression > 0.5
    Foraging, // normal
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityRole {
    Herbivore, // Eats plants, low aggression
    Carnivore, // Eats entities, high aggression
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    pub role: EntityRole,
    pub energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub birth_tick: u64,
    pub generation: u32,
    pub offspring_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub pathogen: Option<crate::model::pathogen::Pathogen>,
    pub infection_timer: u32,
    pub immunity: f32, // 0.0 to 1.0 resistance
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intel {
    pub brain: Brain,
    #[serde(skip)]
    pub last_aggression: f32,
    #[serde(skip)]
    pub last_share_intent: f32,
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

        let role = if rng.gen_bool(0.8) {
            EntityRole::Herbivore
        } else {
            EntityRole::Carnivore
        };

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
            },
            metabolism: Metabolism {
                role,
                energy: 100.0,
                max_energy: 200.0,
                peak_energy: 100.0,
                birth_tick: tick,
                generation: 1,
                offspring_count: 0,
            },
            health: Health {
                pathogen: None,
                infection_timer: 0,
                immunity: rng.gen_range(0.0..0.3),
            },
            intel: Intel {
                brain: Brain::new_random(),
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
            EntityStatus::Infected // NEW: Infected status
        } else if (current_tick - self.metabolism.birth_tick) < maturity_age {
            EntityStatus::Juvenile // NEW: Juvenile state
        } else if self.intel.last_share_intent > 0.5 && self.can_share() {
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
            EntityStatus::Infected => '☣', // NEW: Biohazard symbol
            EntityStatus::Juvenile => '◦', // NEW: Juvenile symbol
            EntityStatus::Sharing => '♣',
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
        }
    }

    pub fn color_for_status(&self, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50), // Dim Red
            EntityStatus::Infected => Color::Rgb(154, 205, 50), // Yellow Green
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255), // Light Blue/Silver
            EntityStatus::Sharing => Color::Rgb(100, 200, 100), // Green
            EntityStatus::Mating => Color::Rgb(255, 105, 180), // Pink
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),   // Red-Orange
            EntityStatus::Foraging => self.color(),
        }
    }

    pub fn is_mature(&self, current_tick: u64, maturity_age: u64) -> bool {
        (current_tick - self.metabolism.birth_tick) >= maturity_age
    }

    // === PATHOGEN METHODS ===

    pub fn try_infect(&mut self, pathogen: &crate::model::pathogen::Pathogen) -> bool {
        if self.health.pathogen.is_some() {
            return false;
        }

        let mut rng = rand::thread_rng();
        // Roll for infection: virulence vs immunity
        let chance = (pathogen.virulence - self.health.immunity).max(0.01);
        if rng.gen::<f32>() < chance {
            self.health.pathogen = Some(pathogen.clone());
            self.health.infection_timer = pathogen.duration;
            return true;
        }
        false
    }

    pub fn process_infection(&mut self) {
        if let Some(p) = &self.health.pathogen {
            self.metabolism.energy -= p.lethality as f64;
            if self.health.infection_timer > 0 {
                self.health.infection_timer -= 1;
            } else {
                // Recovered! Gain immunity
                self.health.pathogen = None;
                self.health.immunity = (self.health.immunity + 0.1).min(1.0);
            }
        }
    }

    // === NEW SOCIAL METHODS ===

    /// Check if entity can share energy (>70% full)
    pub fn can_share(&self) -> bool {
        self.metabolism.energy > self.metabolism.max_energy * 0.7
    }

    /// Share energy with another entity, returns amount shared
    pub fn share_energy(&mut self, max_amount: f64) -> f64 {
        let share = max_amount.min(self.metabolism.energy * 0.15); // Share up to 15%
        self.metabolism.energy -= share;
        share
    }

    /// Calculate territorial aggression bonus based on distance from home
    pub fn territorial_aggression(&self) -> f64 {
        let dist_from_home = ((self.physics.x - self.physics.home_x).powi(2)
            + (self.physics.y - self.physics.home_y).powi(2))
        .sqrt();
        if dist_from_home < 8.0 {
            1.5 // 50% more aggressive near home
        } else {
            1.0
        }
    }

    /// Check if another entity is in the same tribe (similar color)
    pub fn same_tribe(&self, other: &Entity) -> bool {
        let color_dist = (self.physics.r as i32 - other.physics.r as i32).abs()
            + (self.physics.g as i32 - other.physics.g as i32).abs()
            + (self.physics.b as i32 - other.physics.b as i32).abs();
        color_dist < 60 // Threshold for same tribe
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

    pub fn reproduce(&mut self, tick: u64, config: &EvolutionConfig) -> Self {
        let mut rng = rand::thread_rng();

        let child_energy = self.metabolism.energy / 2.0;
        self.metabolism.energy = child_energy;
        self.metabolism.offspring_count += 1;

        let mut child_brain = self.intel.brain.clone();
        child_brain.mutate_with_config(config);

        let r = {
            let change = rng.gen_range(-15..=15);
            (self.physics.r as i16 + change).clamp(0, 255) as u8
        };
        let g = {
            let change = rng.gen_range(-15..=15);
            (self.physics.g as i16 + change).clamp(0, 255) as u8
        };
        let b = {
            let change = rng.gen_range(-15..=15);
            (self.physics.b as i16 + change).clamp(0, 255) as u8
        };

        let mut child_role = self.metabolism.role;
        if rng.gen::<f32>() < config.speciation_rate {
            child_role = match self.metabolism.role {
                EntityRole::Herbivore => EntityRole::Carnivore,
                EntityRole::Carnivore => EntityRole::Herbivore,
            };
        }

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            physics: Physics {
                x: self.physics.x,
                y: self.physics.y,
                vx: self.physics.vx,
                vy: self.physics.vy,
                r,
                g,
                b,
                symbol: '●',
                home_x: self.physics.x,
                home_y: self.physics.y,
            },
            metabolism: Metabolism {
                role: child_role,
                energy: child_energy,
                max_energy: self.metabolism.max_energy,
                peak_energy: child_energy,
                birth_tick: tick,
                generation: self.metabolism.generation + 1,
                offspring_count: 0,
            },
            health: Health {
                pathogen: None,
                infection_timer: 0,
                immunity: (self.health.immunity + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0),
            },
            intel: Intel {
                brain: child_brain,
                last_aggression: 0.0,
                last_share_intent: 0.0,
            },
        }
    }

    pub fn reproduce_with_mate(
        &mut self,
        tick: u64,
        child_brain: Brain,
        speciation_rate: f32,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let child_energy = self.metabolism.energy / 2.0;
        self.metabolism.energy = child_energy;
        self.metabolism.offspring_count += 1;

        let mut child_role = self.metabolism.role;
        if rng.gen::<f32>() < speciation_rate {
            child_role = match self.metabolism.role {
                EntityRole::Herbivore => EntityRole::Carnivore,
                EntityRole::Carnivore => EntityRole::Herbivore,
            };
        }

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            physics: Physics {
                x: self.physics.x,
                y: self.physics.y,
                vx: self.physics.vx,
                vy: self.physics.vy,
                r: self.physics.r,
                g: self.physics.g,
                b: self.physics.b,
                symbol: '●',
                home_x: self.physics.x,
                home_y: self.physics.y,
            },
            metabolism: Metabolism {
                role: child_role,
                energy: child_energy,
                max_energy: self.metabolism.max_energy,
                peak_energy: child_energy,
                birth_tick: tick,
                generation: self.metabolism.generation + 1,
                offspring_count: 0,
            },
            health: Health {
                pathogen: None,
                infection_timer: 0,
                immunity: (self.health.immunity + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0),
            },
            intel: Intel {
                brain: child_brain,
                last_aggression: 0.0,
                last_share_intent: 0.0,
            },
        }
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
    fn test_entity_reproduce_splits_energy() {
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 0.0,
            mutation_amount: 0.0,
            drift_rate: 0.0,
            drift_amount: 0.0,
            speciation_rate: 0.0,
        };

        let mut parent = Entity::new(50.0, 25.0, 0);
        parent.metabolism.energy = 200.0;

        let child = parent.reproduce(100, &config);

        // Energy should be split
        assert_eq!(
            parent.metabolism.energy, 100.0,
            "Parent should have half energy"
        );
        assert_eq!(
            child.metabolism.energy, 100.0,
            "Child should have half energy"
        );

        // Parent stats updated
        assert_eq!(
            parent.metabolism.offspring_count, 1,
            "Parent offspring count should increase"
        );

        // Child properties
        assert_eq!(
            child.parent_id,
            Some(parent.id),
            "Child should reference parent"
        );
        assert_eq!(
            child.metabolism.generation, 2,
            "Child generation should be parent+1"
        );
        assert_eq!(
            child.metabolism.birth_tick, 100,
            "Child birth tick should be current tick"
        );
    }

    #[test]
    fn test_entity_status_starving() {
        let mut entity = Entity::new(50.0, 25.0, 0);
        entity.metabolism.energy = 10.0; // 5% of max (200)

        let status = entity.status(150.0, 200, 150);
        assert_eq!(status, EntityStatus::Starving);
    }

    #[test]
    fn test_entity_status_juvenile() {
        let mut entity = Entity::new(50.0, 25.0, 100);
        entity.metabolism.energy = 100.0;

        // Current tick 200, born at 100, maturity 150 -> age 100 < 150
        let status = entity.status(150.0, 200, 150);
        assert_eq!(status, EntityStatus::Juvenile);
        assert!(!entity.is_mature(200, 150));
    }

    #[test]
    fn test_entity_status_mature_hunting() {
        let mut entity = Entity::new(50.0, 25.0, 0);
        entity.metabolism.energy = 100.0;
        entity.intel.last_aggression = 0.8;

        // Age 200 > maturity 150
        let status = entity.status(150.0, 200, 150);
        assert_eq!(status, EntityStatus::Hunting);
        assert!(entity.is_mature(200, 150));
    }

    #[test]
    fn test_entity_role_inheritance() {
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 0.0,
            mutation_amount: 0.0,
            drift_rate: 0.0,
            drift_amount: 0.0,
            speciation_rate: 0.0, // No role switching
        };

        let mut parent = Entity::new(0.0, 0.0, 0);
        parent.metabolism.role = EntityRole::Carnivore;

        let child = parent.reproduce(100, &config);
        assert_eq!(child.metabolism.role, EntityRole::Carnivore);
    }

    #[test]
    fn test_entity_speciation() {
        let config = crate::model::config::EvolutionConfig {
            mutation_rate: 0.0,
            mutation_amount: 0.0,
            drift_rate: 0.0,
            drift_amount: 0.0,
            speciation_rate: 1.0, // Guaranteed role switching
        };

        let mut parent = Entity::new(0.0, 0.0, 0);
        parent.metabolism.role = EntityRole::Herbivore;

        let child = parent.reproduce(100, &config);
        assert_eq!(child.metabolism.role, EntityRole::Carnivore);
    }

    #[test]
    fn test_entity_same_tribe_similar_colors() {
        let mut entity1 = Entity::new(0.0, 0.0, 0);
        entity1.physics.r = 100;
        entity1.physics.g = 100;
        entity1.physics.b = 100;

        let mut entity2 = Entity::new(0.0, 0.0, 0);
        entity2.physics.r = 110; // Diff 10
        entity2.physics.g = 105; // Diff 5
        entity2.physics.b = 120; // Diff 20 = Total 35 < 60

        assert!(
            entity1.same_tribe(&entity2),
            "Similar colors should be same tribe"
        );
    }

    #[test]
    fn test_entity_different_tribe_different_colors() {
        let mut entity1 = Entity::new(0.0, 0.0, 0);
        entity1.physics.r = 100;
        entity1.physics.g = 100;
        entity1.physics.b = 100;

        let mut entity2 = Entity::new(0.0, 0.0, 0);
        entity2.physics.r = 200; // Diff 100
        entity2.physics.g = 50; // Diff 50
        entity2.physics.b = 150; // Diff 50 = Total 200 > 60

        assert!(
            !entity1.same_tribe(&entity2),
            "Different colors should be different tribe"
        );
    }

    #[test]
    fn test_entity_can_share_high_energy() {
        let mut entity = Entity::new(0.0, 0.0, 0);
        entity.metabolism.energy = 160.0; // 80% of 200

        assert!(
            entity.can_share(),
            "High energy entity should be able to share"
        );
    }

    #[test]
    fn test_entity_cannot_share_low_energy() {
        let mut entity = Entity::new(0.0, 0.0, 0);
        entity.metabolism.energy = 100.0; // 50% of 200

        assert!(!entity.can_share(), "Low energy entity should not share");
    }

    #[test]
    fn test_entity_territorial_aggression_near_home() {
        let mut entity = Entity::new(50.0, 50.0, 0);
        entity.physics.home_x = 50.0;
        entity.physics.home_y = 50.0;
        entity.physics.x = 52.0; // 2 units from home
        entity.physics.y = 52.0;

        let bonus = entity.territorial_aggression();
        assert_eq!(bonus, 1.5, "Should have 1.5x aggression near home");
    }

    #[test]
    fn test_entity_territorial_aggression_far_from_home() {
        let mut entity = Entity::new(50.0, 50.0, 0);
        entity.physics.home_x = 50.0;
        entity.physics.home_y = 50.0;
        entity.physics.x = 70.0; // 20 units from home
        entity.physics.y = 50.0;

        let bonus = entity.territorial_aggression();
        assert_eq!(bonus, 1.0, "Should have normal aggression far from home");
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
