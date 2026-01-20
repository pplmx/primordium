use crate::model::brain::Brain;
use crate::model::config::EvolutionConfig;
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        }
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
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
        }
    }
}
