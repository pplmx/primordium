//! The Silicon Scribe narrative system for the Primordium simulation.
//!
//! Provides async narration generation and history management via mpsc channels.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// A single narrative entry describing a simulation event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narration {
    /// The simulation tick when this event occurred.
    pub tick: u64,
    /// The category or type of event (e.g., "ExtinctionEvent", "NewEra").
    pub event_type: String,
    /// The human-readable narrative text describing the event.
    pub text: String,
    /// The severity or importance of the event (0.0 to 1.0).
    pub severity: f32,
}

/// Trait for generating narrative text from simulation events.
#[async_trait]
pub trait Narrator: Send + Sync {
    /// Generates a narrative string for a given simulation event.
    async fn generate_narration(
        &self,
        tick: u64,
        event_type: &str,
        description: &str,
        severity: f32,
    ) -> String;
}

/// A template-based narrator that generates stylized narratives for known event types.
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
            "MigrationEvent" => format!(
                "{} The Great Wandering: massive clusters are moving south. (Tick {})",
                prefix, tick
            ),
            "WarEvent" => format!(
                "{} Tribal Strife: conflict has erupted between dominant lineages. (Tick {})",
                prefix, tick
            ),
            "CivilizationLevelUp" => format!(
                "{} Civilizational Leap: a lineage has achieved a new tier of organization. (Tick {})",
                prefix, tick
            ),
            _ => format!("{} Epoch {}: {}", prefix, tick, description),
        }
    }
}

/// Async narrative engine that manages narration generation and history via mpsc channels.
pub struct SiliconScribe {
    /// Thread-safe collection of generated narrations.
    pub narrations: Arc<Mutex<Vec<Narration>>>,
    /// Maximum number of narrations to retain in history.
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
    /// Creates a new SiliconScribe with the given narrator implementation.
    pub fn new(narrator: Box<dyn Narrator>) -> Self {
        let narrations = Arc::new(Mutex::new(Vec::new()));
        let (tx, mut rx) = mpsc::unbounded_channel::<NarrationRequest>();

        let narrations_clone = Arc::clone(&narrations);
        let max_history = 100;

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                while let Some(req) = rx.recv().await {
                    let text = narrator
                        .generate_narration(
                            req.tick,
                            &req.event_type,
                            &req.description,
                            req.severity,
                        )
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
        }

        Self {
            narrations,
            max_history,
            tx,
        }
    }

    /// Queues a narration request for async processing.
    pub fn narrate(&self, tick: u64, event_type: &str, description: &str, severity: f32) {
        let _ = self.tx.send(NarrationRequest {
            tick,
            event_type: event_type.to_string(),
            description: description.to_string(),
            severity,
        });
    }

    /// Consumes and returns all generated narrations, clearing the history.
    pub fn consume_narrations(&self) -> Vec<Narration> {
        if let Ok(mut list) = self.narrations.lock() {
            std::mem::take(&mut *list)
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heuristic_narrator_extinction_event() {
        let narrator = HeuristicNarrator;
        let text = narrator
            .generate_narration(100, "ExtinctionEvent", "Population collapsed", 0.9)
            .await;
        assert!(text.contains("The Great Thinning"));
        assert!(text.contains("Tick 100"));
        assert!(text.starts_with("◈"));
    }

    #[tokio::test]
    async fn test_heuristic_narrator_great_famine() {
        let narrator = HeuristicNarrator;
        let text = narrator
            .generate_narration(200, "GreatFamine", "Resources vanished", 0.7)
            .await;
        assert!(text.contains("The Age of Hunger"));
        assert!(text.starts_with("◇"));
    }

    #[tokio::test]
    async fn test_heuristic_narrator_climate_shift() {
        let narrator = HeuristicNarrator;
        let text = narrator
            .generate_narration(300, "ClimateShift", "Climate changed", 0.6)
            .await;
        assert!(text.contains("The Heavens Shift"));
        assert!(text.starts_with("◇"));
    }

    #[tokio::test]
    async fn test_heuristic_narrator_new_era() {
        let narrator = HeuristicNarrator;
        let text = narrator
            .generate_narration(400, "NewEra", "New era begins", 0.85)
            .await;
        assert!(text.contains("A New Dawn"));
        assert!(text.starts_with("◈"));
    }

    #[tokio::test]
    async fn test_heuristic_narrator_default_event() {
        let narrator = HeuristicNarrator;
        let text = narrator
            .generate_narration(500, "CustomEvent", "Something happened", 0.3)
            .await;
        assert!(text.contains("Epoch 500"));
        assert!(text.contains("Something happened"));
        assert!(text.starts_with("○"));
    }

    #[test]
    fn test_narration_struct_creation() {
        let narration = Narration {
            tick: 100,
            event_type: "TestEvent".to_string(),
            text: "Test text".to_string(),
            severity: 0.5,
        };
        assert_eq!(narration.tick, 100);
        assert_eq!(narration.event_type, "TestEvent");
        assert_eq!(narration.text, "Test text");
        assert_eq!(narration.severity, 0.5);
    }

    #[tokio::test]
    async fn test_silicon_scribe_queue_and_consume() {
        let scribe = SiliconScribe::default();
        scribe.narrate(10, "ExtinctionEvent", "Collapse", 0.9);
        scribe.narrate(20, "NewEra", "Beginning", 0.8);

        // Give it a moment to process the async messages
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let narrations = scribe.consume_narrations();
        assert_eq!(narrations.len(), 2);
        assert_eq!(narrations[0].tick, 10);
        assert_eq!(narrations[1].tick, 20);

        // Ensure consume cleared the list
        let empty = scribe.consume_narrations();
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_custom_narrator_implementation() {
        struct MockNarrator;
        #[async_trait]
        impl Narrator for MockNarrator {
            async fn generate_narration(
                &self,
                _tick: u64,
                _etype: &str,
                _desc: &str,
                _sev: f32,
            ) -> String {
                "Custom".to_string()
            }
        }

        let scribe = SiliconScribe::new(Box::new(MockNarrator));
        scribe.narrate(1, "Type", "Desc", 0.5);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let narrations = scribe.consume_narrations();
        assert_eq!(narrations.len(), 1);
        assert_eq!(narrations[0].text, "Custom");
    }

    #[tokio::test]
    async fn test_narration_severity_filtering_concept() {
        let scribe = SiliconScribe::default();
        scribe.narrate(1, "Low", "LowSev", 0.1);
        scribe.narrate(2, "High", "HighSev", 0.9);

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let narrations = scribe.consume_narrations();
        assert_eq!(narrations.len(), 2);
        assert!(narrations.iter().any(|n| n.severity > 0.8));
        assert!(narrations.iter().any(|n| n.severity < 0.2));
    }
}
