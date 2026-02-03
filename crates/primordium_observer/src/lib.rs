use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narration {
    pub tick: u64,
    pub event_type: String,
    pub text: String,
    pub severity: f32,
}

#[async_trait]
pub trait Narrator: Send + Sync {
    async fn generate_narration(
        &self,
        tick: u64,
        event_type: &str,
        description: &str,
        severity: f32,
    ) -> String;
}

pub struct HeuristicNarrator;

#[async_trait]
impl Narrator for HeuristicNarrator {
    async fn generate_narration(
        &self,
        tick: u64,
        event_type: &str,
        description: &str,
        severity: f32,
    ) -> String {
        let prefix = if severity > 0.8 {
            "◈"
        } else if severity > 0.5 {
            "◇"
        } else {
            "○"
        };

        match event_type {
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
        }
    }
}

pub struct SiliconScribe {
    pub narrations: Arc<Mutex<Vec<Narration>>>,
    pub max_history: usize,
    tx: mpsc::UnboundedSender<NarrationRequest>,
}

struct NarrationRequest {
    tick: u64,
    event_type: String,
    description: String,
    severity: f32,
}

impl Default for SiliconScribe {
    fn default() -> Self {
        Self::new(Box::new(HeuristicNarrator))
    }
}

impl SiliconScribe {
    pub fn new(narrator: Box<dyn Narrator>) -> Self {
        let narrations = Arc::new(Mutex::new(Vec::new()));
        let (tx, mut rx) = mpsc::unbounded_channel::<NarrationRequest>();

        let narrations_clone = Arc::clone(&narrations);
        let max_history = 100;

        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                let text = narrator
                    .generate_narration(req.tick, &req.event_type, &req.description, req.severity)
                    .await;

                let narration = Narration {
                    tick: req.tick,
                    event_type: req.event_type,
                    text,
                    severity: req.severity,
                };

                if let Ok(mut list) = narrations_clone.lock() {
                    if list.len() >= max_history {
                        list.remove(0);
                    }
                    list.push(narration);
                }
            }
        });

        Self {
            narrations,
            max_history,
            tx,
        }
    }

    pub fn narrate(&self, tick: u64, event_type: &str, description: &str, severity: f32) {
        let _ = self.tx.send(NarrationRequest {
            tick,
            event_type: event_type.to_string(),
            description: description.to_string(),
            severity,
        });
    }

    pub fn consume_narrations(&self) -> Vec<Narration> {
        if let Ok(mut list) = self.narrations.lock() {
            std::mem::take(&mut *list)
        } else {
            Vec::new()
        }
    }
}
