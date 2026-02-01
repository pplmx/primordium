use crate::model::environment::Environment;
use crate::model::world::World;
use primordium_data::{
    Entity, Food, GeneType, Identity, Intel, MetabolicNiche, Metabolism, Physics, Position,
};
use rand::Rng;

impl World {
    pub fn spawn_entity(&mut self, e: Entity) -> hecs::Entity {
        self.ecs.spawn((
            e.identity,
            e.position,
            e.velocity,
            e.appearance,
            e.physics,
            e.metabolism,
            e.health,
            e.intel,
        ))
    }

    pub fn apply_genetic_edit(&mut self, entity_id: uuid::Uuid, gene: GeneType, delta: f32) {
        let mut query = self
            .ecs
            .query::<(&Identity, &mut Intel, &mut Metabolism, &mut Physics)>();
        for (_handle, (identity, intel, met, phys)) in query.iter() {
            if identity.id == entity_id {
                match gene {
                    GeneType::Trophic => {
                        intel.genotype.trophic_potential =
                            (intel.genotype.trophic_potential + delta).clamp(0.0, 1.0);
                        met.trophic_potential = intel.genotype.trophic_potential;
                    }
                    GeneType::Sensing => {
                        intel.genotype.sensing_range =
                            (intel.genotype.sensing_range + delta as f64).clamp(3.0, 30.0);
                        phys.sensing_range = intel.genotype.sensing_range;
                    }
                    GeneType::Speed => {
                        intel.genotype.max_speed =
                            (intel.genotype.max_speed + delta as f64).clamp(0.1, 5.0);
                        phys.max_speed = intel.genotype.max_speed;
                    }
                    GeneType::ReproInvest => {
                        intel.genotype.reproductive_investment =
                            (intel.genotype.reproductive_investment + delta).clamp(0.1, 0.9);
                    }
                    GeneType::Maturity => {
                        intel.genotype.maturity_gene =
                            (intel.genotype.maturity_gene + delta).clamp(0.1, 5.0);
                    }
                    GeneType::MaxEnergy => {
                        intel.genotype.max_energy =
                            (intel.genotype.max_energy + delta as f64).clamp(50.0, 2000.0);
                        met.max_energy = intel.genotype.max_energy;
                    }
                }
                break;
            }
        }
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
                let query = self.ecs.query_mut::<&mut Metabolism>();
                let mut components: Vec<_> = query.into_iter().collect();
                let count = (components.len() / 10).max(1);
                let amount_per = (amount * sign) / count as f32;
                for (_handle, met) in components.iter_mut().take(count) {
                    met.energy = (met.energy + amount_per as f64).clamp(0.0, met.max_energy);
                }
            }
            TradeResource::Oxygen => {
                env.oxygen_level = (env.oxygen_level + (amount * sign) as f64).clamp(0.0, 50.0);
            }
            TradeResource::SoilFertility => {
                self.terrain.add_global_fertility(amount * sign);
            }
            TradeResource::Biomass => {
                if incoming {
                    let mut rng = rand::thread_rng();
                    for _ in 0..(amount as usize) {
                        let fx = rng.gen_range(1..self.width - 1);
                        let fy = rng.gen_range(1..self.height - 1);
                        let n_type = rng.gen_range(0.0..1.0);
                        self.ecs.spawn((
                            Position {
                                x: fx as f64,
                                y: fy as f64,
                            },
                            MetabolicNiche(n_type),
                            Food::new(fx, fy, n_type),
                        ));
                    }
                } else {
                    let mut food_entities = Vec::new();
                    for (handle, _) in self.ecs.query::<&Food>().iter() {
                        food_entities.push(handle);
                    }
                    for &handle in food_entities.iter().take(amount as usize) {
                        let _ = self.ecs.despawn(handle);
                    }
                }
                self.food_dirty = true;
            }
        }
    }

    pub fn apply_relief(&mut self, lineage_id: uuid::Uuid, amount: f32) {
        let mut members = Vec::new();
        for (handle, met) in self.ecs.query::<&Metabolism>().iter() {
            if met.lineage_id == lineage_id {
                members.push(handle);
            }
        }

        if !members.is_empty() {
            let amount_per = amount as f64 / members.len() as f64;
            for handle in members {
                if let Ok(mut met) = self.ecs.get::<&mut Metabolism>(handle) {
                    met.energy = (met.energy + amount_per).min(met.max_energy);
                }
            }
        }
    }

    pub fn clear_research_deltas(&mut self, entity_id: uuid::Uuid) {
        for (_handle, (identity, intel)) in self.ecs.query_mut::<(&Identity, &mut Intel)>() {
            if identity.id == entity_id {
                intel.genotype.brain.weight_deltas.clear();
                break;
            }
        }
    }
}
