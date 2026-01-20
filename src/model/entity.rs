use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status symbols for entity states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityStatus {
    Starving,  // < 20% energy
    Sharing,   // High energy, sharing with neighbors [NEW]
    Mating,    // > reproduction threshold
    Hunting,   // brain aggression > 0.5
    Foraging,  // normal
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
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

        Self {
            id: Uuid::new_v4(),
            parent_id: None,
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
            home_x: x,  // Birth location is home
            home_y: y,
        }
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }

    pub fn status(&self, reproduction_threshold: f64) -> EntityStatus {
        if self.energy / self.max_energy < 0.2 {
            EntityStatus::Starving
        } else if self.last_share_intent > 0.5 && self.can_share() {
            EntityStatus::Sharing  // NEW: Sharing status
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
            EntityStatus::Sharing => '♣',  // NEW: Sharing symbol
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
        }
    }

    pub fn color_for_status(&self, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50),   // Dim Red
            EntityStatus::Sharing => Color::Rgb(100, 200, 100),  // Green [NEW]
            EntityStatus::Mating => Color::Rgb(255, 105, 180),   // Pink
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),     // Red-Orange
            EntityStatus::Foraging => self.color(),
        }
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
        let dist_from_home = ((self.x - self.home_x).powi(2) +
                              (self.y - self.home_y).powi(2)).sqrt();
        if dist_from_home < 8.0 {
            1.5  // 50% more aggressive near home
        } else {
            1.0
        }
    }

    /// Check if another entity is in the same tribe (similar color)
    pub fn same_tribe(&self, other: &Entity) -> bool {
        let color_dist = (self.r as i32 - other.r as i32).abs() +
                         (self.g as i32 - other.g as i32).abs() +
                         (self.b as i32 - other.b as i32).abs();
        color_dist < 60  // Threshold for same tribe
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

        format!(
            "{}{}{}-Gen{}",
            prefix[p_idx], syllables[s1_idx], syllables[s2_idx], self.generation
        )
    }

    pub fn reproduce(&mut self, tick: u64, config: &EvolutionConfig) -> Self {
        let mut rng = rand::thread_rng();

        let child_energy = self.energy / 2.0;
        self.energy = child_energy;
        self.offspring_count += 1;

        let mut child_brain = self.brain.clone();
        child_brain.mutate_with_config(config);

        let mut mutate_color = |c: u8| -> u8 {
            let change = rng.gen_range(-15..=15);
            (c as i16 + change).max(0).min(255) as u8
        };

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
            x: self.x,
            y: self.y,
            vx: self.vx,
            vy: self.vy,
            r: mutate_color(self.r),
            g: mutate_color(self.g),
            b: mutate_color(self.b),
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
            home_x: self.x,  // Child's home is birth location
            home_y: self.y,
        }
    }

    pub fn reproduce_with_mate(&mut self, tick: u64, child_brain: Brain) -> Self {
        let child_energy = self.energy / 2.0;
        self.energy = child_energy;
        self.offspring_count += 1;

        Self {
            id: Uuid::new_v4(),
            parent_id: Some(self.id),
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
