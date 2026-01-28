use crate::model::history::PopulationStats;
use crate::model::state::environment::Environment;
use crate::model::state::lineage_registry::LineageRegistry;
use primordium_observer::SiliconScribe;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEvent {
    pub tick: u64,
    pub event_type: String,
    pub description: String,
    pub severity: f32,
}

pub struct WorldObserver {
    pub history: VecDeque<MacroEvent>,
    pub max_history: usize,
    pub scribe: SiliconScribe,
    last_population: usize,
    ticks_since_famine: u64,
}

impl Default for WorldObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldObserver {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            max_history: 100,
            scribe: SiliconScribe::new(),
            last_population: 0,
            ticks_since_famine: 0,
        }
    }

    pub fn observe(
        &mut self,
        tick: u64,
        stats: &PopulationStats,
        _registry: &LineageRegistry,
        env: &Environment,
    ) {
        let current_pop = stats.population;

        if self.last_population > 100 && current_pop < self.last_population / 2 {
            self.record_event(
                tick,
                "ExtinctionEvent",
                "A massive population collapse has occurred.",
                0.9,
            );
        }

        if matches!(
            env.resource_state(),
            crate::model::state::environment::ResourceState::Famine
        ) {
            self.ticks_since_famine += 1;
            if self.ticks_since_famine == 500 {
                self.record_event(
                    tick,
                    "GreatFamine",
                    "A prolonged resource famine is devastating the ecosystem.",
                    0.8,
                );
            }
        } else {
            self.ticks_since_famine = 0;
        }

        self.last_population = current_pop;
    }

    fn record_event(&mut self, tick: u64, etype: &str, desc: &str, severity: f32) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(MacroEvent {
            tick,
            event_type: etype.to_string(),
            description: desc.to_string(),
            severity,
        });

        // Use Silicon Scribe for narration
        self.scribe.narrate(tick, etype, desc, severity);
    }

    pub fn generate_macro_report(&self) -> String {
        let mut report = String::new();
        report.push_str("--- WORLD MACRO REPORT ---\n");
        if self.history.is_empty() {
            report.push_str("Status: Stable. No major events recorded.\n");
        } else {
            for ev in &self.history {
                report.push_str(&format!(
                    "[Tick {}] {}: {}\n",
                    ev.tick, ev.event_type, ev.description
                ));
            }
        }
        report.push_str("\n--- SILICON SCRIBE NARRATIONS ---\n");
        if self.scribe.narrations.is_empty() {
            report.push_str("No narrations yet.\n");
        } else {
            for n in &self.scribe.narrations {
                report.push_str(&format!("{}\n", n.text));
            }
        }
        report
    }

    pub fn consume_narrations(&mut self) -> Vec<primordium_observer::Narration> {
        self.scribe.consume_narrations()
    }
}
