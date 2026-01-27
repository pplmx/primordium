use crate::model::state::entity::Entity;
use crate::model::world::World;
use rand::Rng;

impl World {
    /// Spawn an entity migrating from another world
    pub fn import_migrant(
        &mut self,
        dna: String,
        energy: f32,
        generation: u32,
        fingerprint: &str,
        checksum: &str,
    ) -> anyhow::Result<()> {
        // 1. Validate Compatibility
        if fingerprint != self.config.fingerprint() {
            anyhow::bail!("Incompatible world fingerprint: {}", fingerprint);
        }

        // 2. Validate Integrity
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(dna.as_bytes());
        hasher.update(energy.to_be_bytes());
        hasher.update(generation.to_be_bytes());
        let real_checksum = hex::encode(hasher.finalize());
        if checksum != real_checksum {
            anyhow::bail!("Migration checksum mismatch");
        }

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

        let genotype = crate::model::state::entity::Genotype::from_hex(&dna)?;
        entity.intel.genotype = genotype;
        // Sync phenotype
        entity.physics.sensing_range = entity.intel.genotype.sensing_range;
        entity.physics.max_speed = entity.intel.genotype.max_speed;
        entity.metabolism.max_energy = entity.intel.genotype.max_energy;
        entity.metabolism.lineage_id = entity.intel.genotype.lineage_id;

        self.lineage_registry.record_migration_in(
            entity.metabolism.lineage_id,
            entity.metabolism.generation,
            self.tick,
        );

        self.entities.push(entity);
        Ok(())
    }
}
