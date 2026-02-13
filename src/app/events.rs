use primordium_core::environment::ClimateState;
use uuid::Uuid;

/// World events that can be subscribed to by UI, Audio, Network systems
#[derive(Debug, Clone)]
pub enum WorldEvent {
    // Entity lifecycle
    EntityBorn {
        x: f64,
        y: f64,
    },
    EntityDied {
        x: f64,
        y: f64,
        generation: u32,
    },

    // Interactions
    Predation {
        predator: Uuid,
        prey: Uuid,
    },
    Mating {
        entity1: Uuid,
        entity2: Uuid,
    },
    BondFormed {
        entity1: Uuid,
        entity2: Uuid,
    },
    BondBroken {
        entity1: Uuid,
        entity2: Uuid,
    },

    // Environment
    ClimateShift {
        from: ClimateState,
        to: ClimateState,
    },
    Disaster {
        kind: DisasterKind,
        x: u16,
        y: u16,
    },

    // Ecosystem
    FoodSpawned {
        x: u16,
        y: u16,
        amount: f32,
    },
    SpeciesExtinct {
        lineage_id: Uuid,
    },

    // Player actions
    DivineIntervention {
        kind: DivineKind,
    },

    // Milestones
    EraChanged {
        from: String,
        to: String,
    },
    PopulationMilestone {
        count: usize,
    },
}

#[derive(Debug, Clone)]
pub enum DisasterKind {
    Plague,
    HeatWave,
    IceAge,
    DustBowl,
}

#[derive(Debug, Clone)]
pub enum DivineKind {
    Mutation,
    Smite,
    Reincarnate,
    Relief,
    MassExtinction,
    ResourceBoom,
}

/// Event handler type
pub type EventHandler = Box<dyn Fn(&WorldEvent)>;

/// Simple event bus for decoupled communication between systems
pub struct EventBus {
    handlers: Vec<EventHandler>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self {
            handlers: Vec::with_capacity(8),
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to all world events
    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(&WorldEvent) + 'static,
    {
        self.handlers.push(Box::new(handler));
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: WorldEvent) {
        for handler in &self.handlers {
            handler(&event);
        }
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.handlers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_event_bus_subscribe_and_publish() {
        let mut bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish(WorldEvent::EntityBorn { x: 0.0, y: 0.0 });
        bus.publish(WorldEvent::EntityBorn { x: 1.0, y: 1.0 });

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_event_bus_multiple_subscribers() {
        let mut bus = EventBus::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let c1 = counter1.clone();
        bus.subscribe(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });

        let c2 = counter2.clone();
        bus.subscribe(move |_| {
            c2.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish(WorldEvent::EntityDied {
            x: 0.0,
            y: 0.0,
            generation: 1,
        });

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_bus_no_subscribers() {
        let bus = EventBus::new();
        // Should not panic
        bus.publish(WorldEvent::EntityBorn { x: 0.0, y: 0.0 });
    }
}
