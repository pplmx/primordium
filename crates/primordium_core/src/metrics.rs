//! Performance metrics collection for the simulation.
//!
//! Provides structured logging and metrics tracking for monitoring
//! simulation performance and health.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Global metrics collector for simulation statistics.
pub struct Metrics {
    tick_count: AtomicU64,
    entity_count: AtomicU64,
    food_count: AtomicU64,
    pub counters: Mutex<HashMap<String, AtomicU64>>,
    start_time: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Creates a new metrics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tick_count: AtomicU64::new(0),
            entity_count: AtomicU64::new(0),
            food_count: AtomicU64::new(0),
            counters: Mutex::new(HashMap::new()),
            start_time: Instant::now(),
        }
    }

    /// Records a completed tick with its duration.
    pub fn record_tick(&self, duration: Duration, entities: usize, food: usize) {
        self.tick_count.fetch_add(1, Ordering::Relaxed);
        self.entity_count.store(entities as u64, Ordering::Relaxed);
        self.food_count.store(food as u64, Ordering::Relaxed);

        // Log at info level every 1000 ticks
        let tick = self.tick_count.load(Ordering::Relaxed);
        if tick.is_multiple_of(1000) {
            tracing::info!(
                tick = tick,
                entities = entities,
                food = food,
                duration_ms = duration.as_millis() as u64,
                "Simulation tick"
            );
        }
    }

    /// Increments a named counter.
    pub fn increment_counter(&self, name: &str) {
        let mut counters = self.counters.lock().unwrap_or_else(|e| e.into_inner());
        counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the current tick count.
    #[must_use]
    pub fn tick_count(&self) -> u64 {
        self.tick_count.load(Ordering::Relaxed)
    }

    /// Gets the current entity count.
    #[must_use]
    pub fn entity_count(&self) -> u64 {
        self.entity_count.load(Ordering::Relaxed)
    }

    /// Gets elapsed time since metrics creation.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Logs a simulation event.
    pub fn log_event(&self, event_type: &str, details: &str) {
        tracing::info!(
            event_type = event_type,
            details = details,
            "Simulation event"
        );
    }

    /// Logs a warning.
    pub fn log_warning(&self, message: &str) {
        tracing::warn!(message);
    }

    /// Logs an error.
    pub fn log_error(&self, message: &str) {
        tracing::error!(message);
    }
}

/// Initialize tracing subscriber for logging.
pub fn init_logging() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        assert_eq!(metrics.tick_count(), 0);
    }

    #[test]
    fn test_record_tick() {
        let metrics = Metrics::new();
        metrics.record_tick(Duration::from_millis(16), 100, 50);
        assert_eq!(metrics.tick_count(), 1);
        assert_eq!(metrics.entity_count(), 100);
    }

    #[test]
    fn test_increment_counter() {
        let metrics = Metrics::new();
        metrics.increment_counter("test");
        metrics.increment_counter("test");
        // Counter value is not directly accessible, but function should not panic
    }
}
