use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narration {
    pub tick: u64,
    pub event_type: String,
    pub text: String,
    pub severity: f32,
}

pub struct SiliconScribe {
    pub narrations: Vec<Narration>,
    pub max_history: usize,
}

impl Default for SiliconScribe {
    fn default() -> Self {
        Self::new()
    }
}

impl SiliconScribe {
    pub fn new() -> Self {
        Self {
            narrations: Vec::new(),
            max_history: 100,
        }
    }

    pub fn narrate(&mut self, tick: u64, event_type: &str, description: &str, severity: f32) {
        let prefix = if severity > 0.8 {
            "◈"
        } else if severity > 0.5 {
            "◇"
        } else {
            "○"
        };

        let text = match event_type {
            "ExtinctionEvent" => format!(
                "{} The Great Thinning: population collapsed. (Tick {})",
                prefix, tick
            ),
            "GreatFamine" => format!(
                "{} The Age of Hunger: resources have vanished. (Tick {})",
                prefix, tick
            ),
            "ClimateShift" => format!(
                "{} The Heavens Shift: global climate has transformed. (Tick {})",
                prefix, tick
            ),
            "NewEra" => format!(
                "{} A New Dawn: a macro-evolutionary era begins. (Tick {})",
                prefix, tick
            ),
            _ => format!("{} Epoch {}: {}", prefix, tick, description),
        };

        let narration = Narration {
            tick,
            event_type: event_type.to_string(),
            text,
            severity,
        };

        if self.narrations.len() >= self.max_history {
            self.narrations.remove(0);
        }
        self.narrations.push(narration);
    }

    pub fn consume_narrations(&mut self) -> Vec<Narration> {
        std::mem::take(&mut self.narrations)
    }
}
