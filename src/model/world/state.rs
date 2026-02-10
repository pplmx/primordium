use crate::model::lifecycle;
use crate::model::snapshot::{EntitySnapshot, WorldSnapshot};
use crate::model::world::World;
use primordium_data::{Food, Physics};
use std::collections::HashMap;
use std::sync::Arc;

pub use crate::model::snapshot::InternalEntitySnapshot;

pub type EntityComponents<'a> = (
    &'a primordium_data::Identity,
    &'a mut primordium_data::Position,
    &'a mut primordium_data::Velocity,
    &'a primordium_data::Physics,
    &'a mut primordium_data::Metabolism,
    &'a mut primordium_data::Intel,
    &'a mut primordium_data::Health,
);

pub struct SpatialHashResult {
    pub entity_id_map: HashMap<uuid::Uuid, usize>,
    pub entity_handles: Vec<hecs::Entity>,
    pub food_handles: Vec<hecs::Entity>,
    pub food_data: Vec<(f64, f64, f32)>,
}

#[derive(Clone, Default)]
pub struct EntityDecision {
    pub outputs: [f32; 12],
    pub nearby_count: usize,
    pub grn_speed_mod: f64,
    pub grn_sensing_mod: f64,
    pub grn_repro_mod: f32,
    pub sensed_food: Option<(usize, f64, f64, f32)>, // index, dx, dy, type
}

impl World {
    pub fn capture_entity_snapshots_with_handles(&mut self, handles: &[hecs::Entity]) {
        self.entity_snapshots.clear();
        for &handle in handles {
            if let Ok(mut query) = self.ecs.query_one::<EntityComponents>(handle) {
                if let Some((identity, position, _velocity, physics, metabolism, intel, health)) =
                    query.get()
                {
                    self.entity_snapshots.push(InternalEntitySnapshot {
                        id: identity.id,
                        lineage_id: metabolism.lineage_id,
                        x: position.x,
                        y: position.y,
                        energy: metabolism.energy,
                        birth_tick: metabolism.birth_tick,
                        offspring_count: metabolism.offspring_count,
                        generation: metabolism.generation,
                        max_energy: metabolism.max_energy,
                        r: physics.r,
                        g: physics.g,
                        b: physics.b,
                        rank: intel.rank,
                        status: lifecycle::calculate_status(
                            metabolism,
                            health,
                            intel,
                            self.config.brain.activation_threshold,
                            self.tick,
                            self.config.metabolism.maturity_age,
                        ),
                        genotype: Some(Arc::clone(&intel.genotype)),
                    });
                }
            }
        }
    }

    pub fn capture_entity_snapshots(&mut self) {
        let sorted_handles = self.get_sorted_handles();
        self.capture_entity_snapshots_with_handles(&sorted_handles);
    }
}

impl World {
    /// Returns the current number of entities in the simulation.
    pub fn get_population_count(&self) -> usize {
        self.ecs
            .query::<&primordium_data::Identity>()
            .iter()
            .count()
    }

    /// Returns the current number of food items in the simulation.
    pub fn get_food_count(&self) -> usize {
        self.ecs.query::<&Food>().iter().count()
    }

    pub fn get_sorted_handles(&self) -> Vec<hecs::Entity> {
        let mut data: Vec<_> = self
            .ecs
            .query::<EntityComponents>()
            .iter()
            .map(|(h, (i, ..))| (h, i.id))
            .collect();
        data.sort_by_key(|d| d.1);
        data.into_iter().map(|d| d.0).collect()
    }

    pub fn get_all_entities(&self) -> Vec<primordium_data::Entity> {
        let mut entities = Vec::new();
        let sorted_handles = self.get_sorted_handles();
        for handle in sorted_handles {
            if let Ok(mut query) = self.ecs.query_one::<(
                &primordium_data::Identity,
                &primordium_data::Position,
                &primordium_data::Velocity,
                &primordium_data::Appearance,
                &primordium_data::Physics,
                &primordium_data::Metabolism,
                &primordium_data::Health,
                &primordium_data::Intel,
            )>(handle)
            {
                if let Some((
                    identity,
                    position,
                    velocity,
                    appearance,
                    physics,
                    metabolism,
                    health,
                    intel,
                )) = query.get()
                {
                    entities.push(primordium_data::Entity {
                        identity: identity.clone(),
                        position: *position,
                        velocity: velocity.clone(),
                        appearance: appearance.clone(),
                        physics: physics.clone(),
                        metabolism: metabolism.clone(),
                        health: health.clone(),
                        intel: intel.clone(),
                    });
                }
            }
        }
        entities
    }

    pub fn prepare_for_save(&mut self) {
        self.food_persist.clear();
        for (_handle, f) in self.ecs.query::<&Food>().iter() {
            self.food_persist.push(f.clone());
        }
        self.food_persist.sort_by_key(|f| (f.x, f.y));
    }

    pub fn create_snapshot(&self, selected_id: Option<uuid::Uuid>) -> WorldSnapshot {
        let mut entities = Vec::new();

        for (_handle, (identity, position, _velocity, physics, metabolism, intel, health)) in
            self.ecs.query::<EntityComponents>().iter()
        {
            entities.push(EntitySnapshot {
                id: identity.id,
                name: lifecycle::get_name_components(&identity.id, metabolism),
                x: position.x,
                y: position.y,
                r: physics.r,
                g: physics.g,
                b: physics.b,
                energy: metabolism.energy,
                max_energy: metabolism.max_energy,
                generation: metabolism.generation,
                age: self.tick - metabolism.birth_tick,
                offspring: metabolism.offspring_count,
                lineage_id: metabolism.lineage_id,
                rank: intel.rank,
                status: lifecycle::calculate_status(
                    metabolism,
                    health,
                    intel,
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
                last_vocalization: intel.last_vocalization,
                bonded_to: intel.bonded_to,
                trophic_potential: metabolism.trophic_potential,
                last_activations: if Some(identity.id) == selected_id {
                    intel
                        .last_activations
                        .0
                        .iter()
                        .enumerate()
                        .filter(|(_, v)| v.abs() > 0.001)
                        .map(|(k, v)| (k as i32, *v))
                        .collect()
                } else {
                    HashMap::new()
                },
                weight_deltas: if Some(identity.id) == selected_id {
                    intel.genotype.brain.weight_deltas.clone()
                } else {
                    HashMap::new()
                },
                genotype_hex: if Some(identity.id) == selected_id {
                    Some(intel.genotype.to_hex())
                } else {
                    None
                },
                specialization: intel.specialization,
                is_larva: !metabolism.has_metamorphosed,
            });
        }

        entities.sort_by_key(|e| e.id);

        let mut food = Vec::new();
        for (_handle, f) in self.ecs.query::<&Food>().iter() {
            food.push(f.clone());
        }
        food.sort_by_key(|f| (f.x, f.y));

        WorldSnapshot {
            tick: self.tick,
            entities,
            food,
            stats: self.pop_stats.clone(),
            hall_of_fame: self.hall_of_fame.clone(),
            terrain: self.cached_terrain.clone(),
            pheromones: self.cached_pheromones.clone(),
            sound: self.cached_sound.clone(),
            pressure: self.cached_pressure.clone(),
            influence: self.cached_influence.clone(),
            social_grid: self.cached_social_grid.clone(),
            rank_grid: self.cached_rank_grid.clone(),
            width: self.width,
            height: self.height,
        }
    }

    pub fn prepare_spatial_hash(&mut self) -> SpatialHashResult {
        let mut query = self.ecs.query::<(
            &primordium_data::Identity,
            &primordium_data::Position,
            &primordium_data::Metabolism,
        )>();
        let mut entity_data: Vec<_> = query
            .iter()
            .map(|(h, (i, p, m))| (i.id, h, p.x, p.y, m.lineage_id))
            .collect();
        entity_data.sort_by_key(|d| d.0);

        let mut spatial_data = std::mem::take(&mut self.spatial_data_buffer);
        spatial_data.clear();

        let mut entity_handles = Vec::new();
        let mut entity_id_map = HashMap::new();

        for (id, handle, x, y, lid) in entity_data {
            let idx = entity_handles.len();
            entity_id_map.insert(id, idx);
            entity_handles.push(handle);
            spatial_data.push((x, y, lid));
        }

        self.spatial_hash
            .build_with_lineage(&spatial_data, self.width, self.height);

        let mut food_handles = Vec::new();
        let mut food_positions = Vec::new();
        let mut food_data = Vec::new();
        for (handle, (pos, food)) in self.ecs.query::<(&Physics, &Food)>().iter() {
            food_handles.push(handle);
            food_positions.push((pos.x, pos.y));
            food_data.push((pos.x, pos.y, food.nutrient_type));
        }

        self.food_hash
            .build_parallel(&food_positions, self.width, self.height);

        SpatialHashResult {
            entity_id_map,
            entity_handles,
            food_handles,
            food_data,
        }
    }
}
