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
        // Future: Here we would call a local LLM to turn raw data into prose
        let text = format!("Epoch {}: {}", tick, description);

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
}
