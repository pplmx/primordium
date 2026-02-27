pub mod macros;

use primordium_data::{Specialization, TerrainType};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::food::Food;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;
use rand::SeedableRng;
use std::sync::Arc;
use uuid::Uuid;

type TerrainMod = Box<dyn FnOnce(&mut World)>;

#[allow(dead_code)]
pub struct WorldBuilder {
    config: AppConfig,
    entities: Vec<primordium_data::Entity>,
    seed: Option<u64>,
    terrain_mods: Vec<TerrainMod>,
}

#[allow(dead_code)]
impl WorldBuilder {
    pub fn new() -> Self {
        let mut config = AppConfig::default();
        config.world.initial_population = 0;
        config.world.initial_food = 0;
        Self {
            config,
            entities: Vec::new(),
            seed: None,
            terrain_mods: Vec::new(),
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

    pub fn with_terrain(mut self, x: u16, y: u16, terrain_type: TerrainType) -> Self {
        self.terrain_mods.push(Box::new(move |world| {
            Arc::make_mut(&mut world.terrain).set_cell_type(x, y, terrain_type);
        }));
        self
    }

    pub fn with_outpost(mut self, x: u16, y: u16, owner_id: Uuid) -> Self {
        self.terrain_mods.push(Box::new(move |world| {
            let idx = world.terrain.index(x, y);
            let terrain = Arc::make_mut(&mut world.terrain);
            terrain.set_cell_type(x, y, TerrainType::Outpost);
            terrain.cells[idx].owner_id = Some(owner_id);
            terrain.cells[idx].energy_store = 500.0;
        }));
        self
    }

    pub fn with_fertility(mut self, fertility: f32) -> Self {
        self.terrain_mods.push(Box::new(move |world| {
            for cell in Arc::make_mut(&mut world.terrain).cells.iter_mut() {
                cell.fertility = fertility;
            }
        }));
        self
    }

    pub fn with_food_spawn_logic(mut self) -> Self {
        self.terrain_mods.push(Box::new(|world| {
            world.food_dirty = true;
        }));
        self
    }

    pub fn with_food(mut self, x: f64, y: f64, nutrient_type: f32) -> Self {
        self.terrain_mods.push(Box::new(move |world| {
            world.ecs.spawn((
                primordium_data::Position { x, y },
                primordium_data::MetabolicNiche(nutrient_type),
                Food::new(x as u16, y as u16, nutrient_type),
            ));
            world
                .food_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            world.food_dirty = true;
        }));
        self
    }

    pub fn with_memory(mut self, lineage_id: Uuid, key: &str, value: f32) -> Self {
        let key = key.to_string();
        self.terrain_mods.push(Box::new(move |world| {
            if !world.lineage_registry.lineages.contains_key(&lineage_id) {
                world.lineage_registry.record_birth(lineage_id, 0, 0);
            }
            world
                .lineage_registry
                .set_memory_value(&lineage_id, &key, value);
        }));
        self
    }

    pub fn build(self) -> (World, Environment) {
        let is_deterministic = self.config.world.deterministic;
        let mut world = World::new(0, self.config).expect("Failed to create world in test builder");
        let mut env = Environment::default();

        if is_deterministic {
            env.tick_deterministic(0);
        }

        for modifier in self.terrain_mods {
            modifier(&mut world);
        }

        for e in self.entities {
            world.spawn_entity(e);
        }

        (world, env)
    }
}

#[allow(dead_code)]
pub struct EntityBuilder {
    x: f64,
    y: f64,
    energy: f64,
    max_energy: f64,
    color: (u8, u8, u8),
    lineage_id: Option<Uuid>,
    specialization: Option<Specialization>,
    trophic_potential: f32,
    metabolic_niche: f32,
    brain_connections: Vec<primordium_lib::model::brain::Connection>,
    rank: Option<f32>,
    id: Option<Uuid>,
}

#[allow(dead_code)]
impl EntityBuilder {
    pub fn new() -> Self {
        Self {
            x: 10.0,
            y: 10.0,
            energy: 100.0,
            max_energy: 100.0,
            color: (100, 100, 100),
            lineage_id: None,
            specialization: None,
            trophic_potential: 0.5,
            metabolic_niche: 0.5,
            brain_connections: Vec::new(),
            rank: None,
            id: None,
        }
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
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

    pub fn specialization(mut self, spec: Specialization) -> Self {
        self.specialization = Some(spec);
        self
    }

    pub fn trophic(mut self, potential: f32) -> Self {
        self.trophic_potential = potential;
        self
    }

    pub fn niche(mut self, niche: f32) -> Self {
        self.metabolic_niche = niche;
        self
    }

    pub fn with_connection(mut self, from: usize, to: usize, weight: f32) -> Self {
        self.brain_connections
            .push(primordium_lib::model::brain::Connection {
                from,
                to,
                weight,
                enabled: true,
                innovation: 9999 + self.brain_connections.len(),
            });
        self
    }

    pub fn with_behavior(mut self, behavior: TestBehavior) -> Self {
        match behavior {
            TestBehavior::Aggressive => {
                self = self
                    .with_connection(0, 32, 10.0)
                    .with_connection(2, 32, 10.0);
            }
            TestBehavior::Altruist => {
                self = self.with_connection(2, 33, 10.0);
            }
            TestBehavior::BondBreaker => {
                self = self
                    .with_connection(2, 41, -10.0)
                    .with_connection(41, 37, 10.0);
            }
            TestBehavior::SiegeSoldier => {
                self = self
                    .specialization(Specialization::Soldier)
                    .with_connection(5, 32, 10.0);
            }
        }
        self
    }

    pub fn rank(mut self, rank: f32) -> Self {
        self.rank = Some(rank);
        self
    }

    pub fn build(self) -> primordium_data::Entity {
        // Use deterministic RNG seeded from entity fields to ensure reproducible brains.
        // This prevents flaky tests caused by thread_rng() producing different brain weights.
        let seed = self
            .id
            .map(|id| id.as_u128() as u64)
            .unwrap_or((self.x.to_bits() ^ self.y.to_bits()).wrapping_mul(0x517CC1B727220A95));
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let mut e = lifecycle::create_entity_deterministic(self.x, self.y, 0, &mut rng);
        if let Some(id) = self.id {
            e.identity.id = id;
        }
        e.metabolism.energy = self.energy;
        e.metabolism.max_energy = self.max_energy;
        e.metabolism.trophic_potential = self.trophic_potential;
        e.physics.r = self.color.0;
        e.physics.g = self.color.1;
        e.physics.b = self.color.2;
        e.intel.specialization = self.specialization;

        if let Some(lid) = self.lineage_id {
            e.metabolism.lineage_id = lid;
            std::sync::Arc::make_mut(&mut e.intel.genotype).lineage_id = lid;
        }

        std::sync::Arc::make_mut(&mut e.intel.genotype).metabolic_niche = self.metabolic_niche;
        std::sync::Arc::make_mut(&mut e.intel.genotype).trophic_potential = self.trophic_potential;

        if !self.brain_connections.is_empty() {
            let brain = &mut std::sync::Arc::make_mut(&mut e.intel.genotype).brain;
            brain.connections = self.brain_connections;
            use primordium_lib::model::brain::BrainLogic;
            brain.initialize_node_idx_map();
        }

        std::sync::Arc::make_mut(&mut e.intel.genotype).max_energy = self.max_energy;

        if let Some(r) = self.rank {
            e.intel.rank = r;
        }

        e
    }
}

#[allow(dead_code)]
pub enum TestBehavior {
    Aggressive,
    Altruist,
    BondBreaker,
    SiegeSoldier,
}
