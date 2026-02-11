use crate::model::environment::Environment;
use crate::model::world::World;
use primordium_data::{Entity, Food, GeneType, Identity, Intel, Metabolism, Physics, Position};
use rand::Rng;

impl World {
    pub fn reincarnate_selected(&mut self, entity_id: uuid::Uuid) {
        let mut query = self
            .ecs
            .query::<(&mut Intel, &mut Physics, &mut Metabolism, &Identity)>();
        for (_handle, (intel, phys, met, identity)) in query.iter() {
            if identity.id == entity_id {
                intel.genotype = std::sync::Arc::new(
                    crate::model::brain::create_genotype_random_with_rng(&mut rand::thread_rng()),
                );
                crate::model::brain::BrainLogic::initialize_node_idx_map(
                    &mut std::sync::Arc::make_mut(&mut intel.genotype).brain,
                );
                phys.sensing_range = intel.genotype.sensing_range;
                phys.max_speed = intel.genotype.max_speed;
                met.max_energy = intel.genotype.max_energy;
                break;
            }
        }
    }

    pub fn apply_genetic_edit(&mut self, entity_id: uuid::Uuid, gene: GeneType, delta: f32) {
        let mut query = self
            .ecs
            .query::<(&Identity, &mut Intel, &mut Metabolism, &mut Physics)>();
        for (_handle, (identity, intel, met, phys)) in query.iter() {
            if identity.id == entity_id {
                let genotype = std::sync::Arc::make_mut(&mut intel.genotype);
                match gene {
                    GeneType::Trophic => {
                        genotype.trophic_potential =
                            (genotype.trophic_potential + delta).clamp(0.0, 1.0);
                        met.trophic_potential = genotype.trophic_potential;
                    }
                    GeneType::Sensing => {
                        genotype.sensing_range =
                            (genotype.sensing_range + delta as f64).clamp(3.0, 30.0);
                        phys.sensing_range = genotype.sensing_range;
                    }
                    GeneType::Speed => {
                        genotype.max_speed = (genotype.max_speed + delta as f64).clamp(0.1, 5.0);
                        phys.max_speed = genotype.max_speed;
                    }
                    GeneType::ReproInvest => {
                        genotype.reproductive_investment =
                            (genotype.reproductive_investment + delta).clamp(0.1, 0.9);
                    }
                    GeneType::Maturity => {
                        genotype.maturity_gene = (genotype.maturity_gene + delta).clamp(0.1, 5.0);
                    }
                    GeneType::MaxEnergy => {
                        genotype.max_energy =
                            (genotype.max_energy + delta as f64).clamp(50.0, 2000.0);
                        met.max_energy = genotype.max_energy;
                    }
                }
                break;
            }
        }
    }

    pub fn apply_relief(&mut self, lineage_id: uuid::Uuid, amount: f32) {
        let mut targets = Vec::new();
        {
            let mut query = self.ecs.query::<&Metabolism>();
            for (h, met) in query.iter() {
                if met.lineage_id == lineage_id {
                    targets.push(h);
                }
            }
        }
        if !targets.is_empty() {
            let per_target = amount as f64 / targets.len() as f64;
            for h in targets {
                if let Ok(mut met) = self.ecs.get::<&mut Metabolism>(h) {
                    met.energy = (met.energy + per_target).min(met.max_energy);
                }
            }
        }
    }

    pub fn spawn_entity(&mut self, entity: Entity) -> hecs::Entity {
        self.ecs.spawn((
            entity.identity,
            entity.position,
            entity.velocity,
            entity.appearance,
            entity.physics,
            entity.metabolism,
            entity.health,
            entity.intel,
        ))
    }

    pub fn apply_trade(
        &mut self,
        env: &mut Environment,
        resource: crate::model::infra::network::TradeResource,
        amount: f32,
        incoming: bool,
    ) {
        use crate::model::infra::network::TradeResource;
        let sign = if incoming { 1.0 } else { -1.0 };
        match resource {
            TradeResource::Energy => {
                let pop = self.get_population_count();
                if pop > 0 {
                    let amount_per = (amount as f64 * sign) / pop as f64;
                    for (_handle, met) in self.ecs.query_mut::<&mut Metabolism>() {
                        met.energy = (met.energy + amount_per).min(met.max_energy);
                    }
                }
            }
            TradeResource::Oxygen => {
                env.oxygen_level = (env.oxygen_level + (amount as f64 * sign)).clamp(0.0, 100.0);
            }
            TradeResource::SoilFertility => {
                std::sync::Arc::make_mut(&mut self.terrain)
                    .add_global_fertility(amount * sign as f32);
            }
            TradeResource::Biomass => {
                if incoming {
                    let mut rng = rand::thread_rng();
                    for _ in 0..(amount as usize).min(100) {
                        let fx = rng.gen_range(1..self.width - 1);
                        let fy = rng.gen_range(1..self.height - 1);
                        let n_type = rng.gen_range(0.0..1.0);
                        self.ecs.spawn((
                            Food::new(fx, fy, n_type),
                            Position {
                                x: fx as f64,
                                y: fy as f64,
                            },
                            primordium_data::MetabolicNiche(n_type),
                        ));
                    }
                    self.food_dirty = true;
                } else {
                    let mut handles = Vec::new();
                    for (h, _) in self.ecs.query::<&Food>().iter() {
                        handles.push(h);
                        if handles.len() >= amount as usize {
                            break;
                        }
                    }
                    for h in handles {
                        let _ = self.ecs.despawn(h);
                    }
                    self.food_dirty = true;
                }
            }
        }
    }

    pub fn clear_research_deltas(&mut self, entity_id: uuid::Uuid) {
        for (_handle, (identity, intel)) in self.ecs.query_mut::<(&Identity, &mut Intel)>() {
            if identity.id == entity_id {
                std::sync::Arc::make_mut(&mut intel.genotype)
                    .brain
                    .weight_deltas
                    .clear();
                break;
            }
        }
    }

    /// Generate a deterministic hash of the entire world state for verification.
    pub fn deterministic_hash(&self, env: &Environment) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();

        // 1. Tick
        hasher.update(self.tick.to_le_bytes());

        // 2. Entities (Sorted by ID)
        let mut entity_data: Vec<_> = self
            .ecs
            .query::<(
                &primordium_data::Identity,
                &primordium_data::Position,
                &primordium_data::Metabolism,
                &primordium_data::Intel,
            )>()
            .iter()
            .map(|(_, (id, pos, met, intel))| {
                (
                    id.id,
                    pos.x,
                    pos.y,
                    met.energy,
                    intel.genotype.lineage_id,
                    intel.genotype.sensing_range,
                    intel.genotype.max_speed,
                    intel.genotype.max_energy,
                )
            })
            .collect();

        entity_data.sort_by_key(|e| e.0);
        for (id, x, y, energy, lineage_id, sensing_range, max_speed, max_energy) in entity_data {
            hasher.update(id.as_bytes());
            // Use bits for float stability in hash
            hasher.update(x.to_bits().to_le_bytes());
            hasher.update(y.to_bits().to_le_bytes());
            hasher.update(energy.to_bits().to_le_bytes());

            hasher.update(lineage_id.as_bytes());
            hasher.update(sensing_range.to_bits().to_le_bytes());
            hasher.update(max_speed.to_bits().to_le_bytes());
            hasher.update(max_energy.to_bits().to_le_bytes());
        }

        // 3. Food (Sorted by position)
        let mut food: Vec<_> = Vec::new();
        for (_, (pos, f)) in self.ecs.query::<(&Position, &Food)>().iter() {
            food.push((pos.x, pos.y, f.nutrient_type));
        }
        food.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        for f in food {
            hasher.update(f.0.to_bits().to_le_bytes());
            hasher.update(f.1.to_bits().to_le_bytes());
            hasher.update(f.2.to_bits().to_le_bytes());
        }

        // 4. Terrain
        for cell in &self.terrain.cells {
            hasher.update((cell.terrain_type as u8).to_le_bytes());
            hasher.update(cell.fertility.to_bits().to_le_bytes());
        }

        // 5. Environment
        hasher.update(env.carbon_level.to_bits().to_le_bytes());
        hasher.update(env.oxygen_level.to_bits().to_le_bytes());

        hex::encode(hasher.finalize())
    }
}
