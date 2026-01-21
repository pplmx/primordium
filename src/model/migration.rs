use crate::model::entity::Entity;
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
        entity.energy = energy as f64;
        entity.generation = generation;

        // Deserialize DNA (Brain)
        if let Ok(brain) = crate::model::brain::Brain::from_hex(&dna) {
            entity.brain = brain;
        }

        self.entities.push(entity);
    }
}
