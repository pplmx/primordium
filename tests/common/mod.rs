use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use uuid::Uuid;

/// A builder for creating a configured simulation world for integration tests.
#[allow(dead_code)]
pub struct WorldBuilder {
    config: AppConfig,
    entities: Vec<primordium_data::Entity>,
    seed: Option<u64>,
}

#[allow(dead_code)]
impl WorldBuilder {
    pub fn new() -> Self {
        let mut config = AppConfig::default();
        config.world.initial_population = 0; // Default to manual spawning
        Self {
            config,
            entities: Vec::new(),
            seed: None,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self.config.world.seed = Some(seed);
        self
    }

    pub fn with_config<F>(mut self, modifier: F) -> Self
    where
        F: FnOnce(&mut AppConfig),
    {
        modifier(&mut self.config);
        self
    }

    pub fn with_entity(mut self, entity: primordium_data::Entity) -> Self {
        self.entities.push(entity);
        self
    }

    pub fn build(self) -> (World, Environment) {
        let mut world = World::new(0, self.config).expect("Failed to create world in test builder");
        let env = Environment::default();

        for e in self.entities {
            world.spawn_entity(e);
        }

        (world, env)
    }
}

/// A builder for creating entities with specific traits and behaviors.
pub struct EntityBuilder {
    x: f64,
    y: f64,
    energy: f64,
    max_energy: f64,
    color: (u8, u8, u8),
    lineage_id: Option<Uuid>,
    brain_connections: Vec<primordium_lib::model::brain::Connection>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            x: 10.0,
            y: 10.0,
            energy: 100.0,
            max_energy: 100.0,
            color: (100, 100, 100),
            lineage_id: None,
            brain_connections: Vec::new(),
        }
    }

    pub fn at(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn energy(mut self, amount: f64) -> Self {
        self.energy = amount;
        self
    }

    pub fn max_energy(mut self, amount: f64) -> Self {
        self.max_energy = amount;
        self
    }

    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = (r, g, b);
        self
    }

    pub fn lineage(mut self, id: Uuid) -> Self {
        self.lineage_id = Some(id);
        self
    }

    pub fn with_behavior(mut self, behavior: TestBehavior) -> Self {
        match behavior {
            TestBehavior::Aggressive => {
                // Connect density/bias to Aggression (output 32)
                self.brain_connections
                    .push(primordium_lib::model::brain::Connection {
                        from: 0, // Food DX (just a placeholder high input if food is near)
                        to: 32,  // Aggro
                        weight: 10.0,
                        enabled: true,
                        innovation: 9999,
                    });
                // Also connect bias/energy to ensure firing
                self.brain_connections
                    .push(primordium_lib::model::brain::Connection {
                        from: 2, // Energy
                        to: 32,  // Aggro
                        weight: 10.0,
                        enabled: true,
                        innovation: 10000,
                    });
            }
            TestBehavior::Altruist => {
                // Connect Energy (2) to Share (33)
                self.brain_connections
                    .push(primordium_lib::model::brain::Connection {
                        from: 2,
                        to: 33,
                        weight: 10.0,
                        enabled: true,
                        innovation: 10001,
                    });
            }
            TestBehavior::BondBreaker => {
                // Connect Energy (2) to Hidden (41) then to Bond (37->8)
                self.brain_connections
                    .push(primordium_lib::model::brain::Connection {
                        from: 2,
                        to: 41,
                        weight: -10.0,
                        enabled: true,
                        innovation: 1,
                    });
                self.brain_connections
                    .push(primordium_lib::model::brain::Connection {
                        from: 41,
                        to: 37,
                        weight: 10.0,
                        enabled: true,
                        innovation: 2,
                    });
            }
        }
        self
    }

    pub fn build(self) -> primordium_data::Entity {
        let mut e = lifecycle::create_entity(self.x, self.y, 0);
        e.metabolism.energy = self.energy;
        e.metabolism.max_energy = self.max_energy;
        e.physics.r = self.color.0;
        e.physics.g = self.color.1;
        e.physics.b = self.color.2;

        if let Some(lid) = self.lineage_id {
            e.metabolism.lineage_id = lid;
            e.intel.genotype.lineage_id = lid;
        }

        if !self.brain_connections.is_empty() {
            e.intel.genotype.brain.connections = self.brain_connections;
        }

        // Sync genotype max energy
        e.intel.genotype.max_energy = self.max_energy;

        e
    }
}

#[allow(dead_code)]
pub enum TestBehavior {
    Aggressive,
    Altruist,
    BondBreaker,
}
