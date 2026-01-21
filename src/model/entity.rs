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
    Juvenile, // NEW: Too young to reproduce
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
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub role: EntityRole, // NEW: Trophic role
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
    pub energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub generation: u32,
    pub birth_tick: u64,
    pub offspring_count: u32,
    pub brain: Brain,
    #[serde(skip)]
    pub last_aggression: f32,
    #[serde(skip)]
    pub last_share_intent: f32, // NEW: Share intent output
    // Territorial behavior
    pub home_x: f64, // NEW: Birth location X
    pub home_y: f64, // NEW: Birth location Y
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
            role,
            x,
            y,
            vx,
            vy,
            r,
            g,
            b,
            symbol: '●',
            energy: 100.0,
            max_energy: 200.0,
            peak_energy: 100.0,
            generation: 1,
            birth_tick: tick,
            offspring_count: 0,
            brain: Brain::new_random(),
            last_aggression: 0.0,
            last_share_intent: 0.0,
            home_x: x, // Birth location is home
            home_y: y,
        }
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    pub fn status(
        &self,
        reproduction_threshold: f64,
        current_tick: u64,
        maturity_age: u64,
    ) -> EntityStatus {
        if self.energy / self.max_energy < 0.2 {
            EntityStatus::Starving
        } else if (current_tick - self.birth_tick) < maturity_age {
            EntityStatus::Juvenile // NEW: Juvenile state
        } else if self.last_share_intent > 0.5 && self.can_share() {
            EntityStatus::Sharing
        } else if self.last_aggression > 0.5 {
            EntityStatus::Hunting
        } else if self.energy > reproduction_threshold {
            EntityStatus::Mating
        } else {
            EntityStatus::Foraging
        }
    }

    pub fn symbol_for_status(&self, status: EntityStatus) -> char {
        match status {
            EntityStatus::Starving => '†',
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
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255), // Light Blue/Silver
            EntityStatus::Sharing => Color::Rgb(100, 200, 100), // Green
            EntityStatus::Mating => Color::Rgb(255, 105, 180), // Pink
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),   // Red-Orange
            EntityStatus::Foraging => self.color(),
        }
    }

    pub fn is_mature(&self, current_tick: u64, maturity_age: u64) -> bool {
        (current_tick - self.birth_tick) >= maturity_age
    }

    // === NEW SOCIAL METHODS ===

    /// Check if entity can share energy (>70% full)
    pub fn can_share(&self) -> bool {
        self.energy > self.max_energy * 0.7
    }

    /// Share energy with another entity, returns amount shared
    pub fn share_energy(&mut self, max_amount: f64) -> f64 {
        let share = max_amount.min(self.energy * 0.15); // Share up to 15%
        self.energy -= share;
        share
    }

    /// Calculate territorial aggression bonus based on distance from home
    pub fn territorial_aggression(&self) -> f64 {
        let dist_from_home =
            ((self.x - self.home_x).powi(2) + (self.y - self.home_y).powi(2)).sqrt();
        if dist_from_home < 8.0 {
            1.5 // 50% more aggressive near home
        } else {
            1.0
        }
    }

    /// Check if another entity is in the same tribe (similar color)
    pub fn same_tribe(&self, other: &Entity) -> bool {
        let color_dist = (self.r as i32 - other.r as i32).abs()
            + (self.g as i32 - other.g as i32).abs()
            + (self.b as i32 - other.b as i32).abs();
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

        let role_prefix = match self.role {
            EntityRole::Herbivore => "H-",
            EntityRole::Carnivore => "C-",
        };

        format!(
            "{}{}{}{}-Gen{}",
            role_prefix, prefix[p_idx], syllables[s1_idx], syllables[s2_idx], self.generation
        )
    }

    pub fn reproduce(&mut self, tick: u64, config: &EvolutionConfig) -> Self {
        let mut rng = rand::thread_rng();

        let child_energy = self.energy / 2.0;
        self.energy = child_energy;
        self.offspring_count += 1;

        let mut child_brain = self.brain.clone();
        child_brain.mutate_with_config(config);

        let r = {
            let change = rng.gen_range(-15..=15);
            (self.r as i16 + change).clamp(0, 255) as u8
        };
        let g = {
            let change = rng.gen_range(-15..=15);
            (self.g as i16 + change).clamp(0, 255) as u8
        };
        let b = {
            let change = rng.gen_range(-15..=15);
            (self.b as i16 + change).clamp(0, 255) as u8
        };

        let mut child_role = self.role;
        if rng.gen::<f32>() < config.speciation_rate {
            child_role = match self.role {
                EntityRole::Herbivore => EntityRole::Carnivore,
                EntityRole::Carnivore => EntityRole::Herbivore,
            };
        }

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            role: child_role,
            x: self.x,
            y: self.y,
            vx: self.vx,
            vy: self.vy,
            r,
            g,
            b,
            symbol: '●',
            energy: child_energy,
            max_energy: self.max_energy,
            peak_energy: child_energy,
            generation: self.generation + 1,
            birth_tick: tick,
            offspring_count: 0,
            brain: child_brain,
            last_aggression: 0.0,
            last_share_intent: 0.0,
            home_x: self.x, // Child's home is birth location
            home_y: self.y,
        }
    }

    pub fn reproduce_with_mate(
        &mut self,
        tick: u64,
        child_brain: Brain,
        speciation_rate: f32,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let child_energy = self.energy / 2.0;
        self.energy = child_energy;
        self.offspring_count += 1;

        let mut child_role = self.role;
        if rng.gen::<f32>() < speciation_rate {
            child_role = match self.role {
                EntityRole::Herbivore => EntityRole::Carnivore,
                EntityRole::Carnivore => EntityRole::Herbivore,
            };
        }

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            role: child_role,
            x: self.x,
            y: self.y,
            vx: self.vx,
            vy: self.vy,
            r: self.r,
            g: self.g,
            b: self.b,
            symbol: '●',
            energy: child_energy,
            max_energy: self.max_energy,
            peak_energy: child_energy,
            generation: self.generation + 1,
            birth_tick: tick,
            offspring_count: 0,
            brain: child_brain,
            last_aggression: 0.0,
            last_share_intent: 0.0,
            home_x: self.x,
            home_y: self.y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_new_has_valid_initial_state() {
        let entity = Entity::new(50.0, 25.0, 100);

        assert_eq!(entity.x, 50.0);
        assert_eq!(entity.y, 25.0);
        assert_eq!(entity.birth_tick, 100);
        assert_eq!(entity.generation, 1);
        assert_eq!(entity.energy, 100.0);
        assert_eq!(entity.max_energy, 200.0);
        assert_eq!(entity.offspring_count, 0);
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
        parent.energy = 200.0;

        let child = parent.reproduce(100, &config);

        // Energy should be split
        assert_eq!(parent.energy, 100.0, "Parent should have half energy");
        assert_eq!(child.energy, 100.0, "Child should have half energy");

        // Parent stats updated
        assert_eq!(
            parent.offspring_count, 1,
            "Parent offspring count should increase"
        );

        // Child properties
        assert_eq!(
            child.parent_id,
            Some(parent.id),
            "Child should reference parent"
        );
        assert_eq!(child.generation, 2, "Child generation should be parent+1");
        assert_eq!(
            child.birth_tick, 100,
            "Child birth tick should be current tick"
        );
    }

    #[test]
    fn test_entity_status_starving() {
        let mut entity = Entity::new(50.0, 25.0, 0);
        entity.energy = 10.0; // 5% of max (200)

        let status = entity.status(150.0, 200, 150);
        assert_eq!(status, EntityStatus::Starving);
    }

    #[test]
    fn test_entity_status_juvenile() {
        let mut entity = Entity::new(50.0, 25.0, 100);
        entity.energy = 100.0;

        // Current tick 200, born at 100, maturity 150 -> age 100 < 150
        let status = entity.status(150.0, 200, 150);
        assert_eq!(status, EntityStatus::Juvenile);
        assert!(!entity.is_mature(200, 150));
    }

    #[test]
    fn test_entity_status_mature_hunting() {
        let mut entity = Entity::new(50.0, 25.0, 0);
        entity.energy = 100.0;
        entity.last_aggression = 0.8;

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
        parent.role = EntityRole::Carnivore;

        let child = parent.reproduce(100, &config);
        assert_eq!(child.role, EntityRole::Carnivore);
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
        parent.role = EntityRole::Herbivore;

        let child = parent.reproduce(100, &config);
        assert_eq!(child.role, EntityRole::Carnivore);
    }

    #[test]
    fn test_entity_same_tribe_similar_colors() {
        let mut entity1 = Entity::new(0.0, 0.0, 0);
        entity1.r = 100;
        entity1.g = 100;
        entity1.b = 100;

        let mut entity2 = Entity::new(0.0, 0.0, 0);
        entity2.r = 110; // Diff 10
        entity2.g = 105; // Diff 5
        entity2.b = 120; // Diff 20 = Total 35 < 60

        assert!(
            entity1.same_tribe(&entity2),
            "Similar colors should be same tribe"
        );
    }

    #[test]
    fn test_entity_different_tribe_different_colors() {
        let mut entity1 = Entity::new(0.0, 0.0, 0);
        entity1.r = 100;
        entity1.g = 100;
        entity1.b = 100;

        let mut entity2 = Entity::new(0.0, 0.0, 0);
        entity2.r = 200; // Diff 100
        entity2.g = 50; // Diff 50
        entity2.b = 150; // Diff 50 = Total 200 > 60

        assert!(
            !entity1.same_tribe(&entity2),
            "Different colors should be different tribe"
        );
    }

    #[test]
    fn test_entity_can_share_high_energy() {
        let mut entity = Entity::new(0.0, 0.0, 0);
        entity.energy = 160.0; // 80% of 200

        assert!(
            entity.can_share(),
            "High energy entity should be able to share"
        );
    }

    #[test]
    fn test_entity_cannot_share_low_energy() {
        let mut entity = Entity::new(0.0, 0.0, 0);
        entity.energy = 100.0; // 50% of 200

        assert!(!entity.can_share(), "Low energy entity should not share");
    }

    #[test]
    fn test_entity_territorial_aggression_near_home() {
        let mut entity = Entity::new(50.0, 50.0, 0);
        entity.home_x = 50.0;
        entity.home_y = 50.0;
        entity.x = 52.0; // 2 units from home
        entity.y = 52.0;

        let bonus = entity.territorial_aggression();
        assert_eq!(bonus, 1.5, "Should have 1.5x aggression near home");
    }

    #[test]
    fn test_entity_territorial_aggression_far_from_home() {
        let mut entity = Entity::new(50.0, 50.0, 0);
        entity.home_x = 50.0;
        entity.home_y = 50.0;
        entity.x = 70.0; // 20 units from home
        entity.y = 50.0;

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
