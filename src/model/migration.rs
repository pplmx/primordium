use crate::model::state::entity::Entity;
use crate::model::world::World;
use rand::Rng;

impl World {
    /// Spawn an entity migrating from another world
    pub fn import_migrant(&mut self, dna: String, energy: f32, generation: u32) {
        let mut rng = rand::thread_rng();

        // Spawn at random edge
        let (x, y) = if rng.gen_bool(0.5) {
            (
                if rng.gen_bool(0.5) {
                    1.0
                } else {
                    (self.width - 2) as f64
                },
                rng.gen_range(1.0..(self.height - 2) as f64),
            )
        } else {
            (
                rng.gen_range(1.0..(self.width - 2) as f64),
                if rng.gen_bool(0.5) {
                    1.0
                } else {
                    (self.height - 2) as f64
                },
            )
        };

        let mut entity = Entity::new(x, y, self.tick);
        entity.metabolism.energy = energy as f64;
        entity.metabolism.generation = generation;

        // Deserialize DNA (Genotype)
        if let Ok(genotype) = crate::model::state::entity::Genotype::from_hex(&dna) {
            entity.intel.genotype = genotype;
            // Sync phenotype
            entity.physics.sensing_range = entity.intel.genotype.sensing_range;
            entity.physics.max_speed = entity.intel.genotype.max_speed;
            entity.metabolism.max_energy = entity.intel.genotype.max_energy;
        }

        self.entities.push(entity);
    }
}
