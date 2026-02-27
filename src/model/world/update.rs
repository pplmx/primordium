use crate::model::environment::Environment;
use crate::model::interaction::InteractionCommand;
use hecs;
use primordium_data::LiveEvent;
use primordium_data::{Entity, Food, Identity, Intel, Metabolism, Physics, Position};
use rand::SeedableRng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::model::world::{systems, EntityComponents, SystemContext, World};
use primordium_core::brain::BrainLogic;
use primordium_core::systems::{action, biological, ecological, environment, social};

impl World {
    /// Advances the simulation by one tick.
    ///
    /// This is the main simulation loop that updates all systems:
    /// - Environment and resource propagation
    /// - Entity perception and neural decision-making
    /// - Interactions (predation, reproduction, social)
    /// - Spatial indexing and statistics
    ///
    /// # Arguments
    /// * `env` - Mutable reference to the environment (climate, oxygen, carbon)
    ///
    /// # Returns
    /// Vector of live events (births, deaths, fossilizations) that occurred this tick
    pub fn update(&mut self, env: &mut Environment) -> anyhow::Result<Vec<LiveEvent>> {
        self.tick += 1;
        let world_seed = self.config.world.seed.unwrap_or(0);

        if self.config.world.deterministic {
            let seed = world_seed.wrapping_add(self.tick).wrapping_add(0x5EED);
            self.rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
            env.tick_deterministic(self.tick);
            self.update_environment_and_resources(env, seed);
        } else {
            self.update_environment_and_resources(env, world_seed);
        }

        let (handles, id_map) = self.build_tick_indices();

        self.pass_social_ranks();
        self.pass_spatial_indexing();
        let (food_handles, food_data) = self.pass_food_indexing();
        self.capture_entity_snapshots_with_handles(&handles);
        self.pass_learning();

        Arc::make_mut(&mut self.influence).update(&self.entity_snapshots);

        let overmind_broadcasts = {
            let mut query = self.ecs.query::<EntityComponents>();
            let mut entity_data: Vec<_> = query.iter().collect();
            entity_data.sort_by_key(|(_h, (i, ..))| i.id);

            let current_biomass_c = entity_data
                .iter()
                .filter_map(|(_, (_, _, _, _, met, _, _))| {
                    if met.trophic_potential > 0.6 {
                        Some(met.energy)
                    } else {
                        None
                    }
                })
                .sum::<f64>();

            let mut interaction_commands_buffer = std::mem::take(&mut self.interaction_buffer);
            let mut decision_buffer = std::mem::take(&mut self.decision_buffer);

            let result = {
                let system_ctx = SystemContext {
                    config: &self.config,
                    ecs: &self.ecs,
                    food_hash: &self.food_hash,
                    spatial_hash: &self.spatial_hash,
                    pheromones: &self.pheromones,
                    sound: &self.sound,
                    pressure: &self.pressure,
                    influence: &self.influence,
                    terrain: &self.terrain,
                    tick: self.tick,
                    registry: &self.lineage_registry,
                    snapshots: &self.entity_snapshots,
                    food_handles: &food_handles,
                    food_data: &food_data,
                    world_seed,
                };

                systems::perceive_and_decide_internal(
                    &system_ctx,
                    env,
                    current_biomass_c,
                    &mut entity_data,
                    &id_map,
                    &mut interaction_commands_buffer,
                    &mut decision_buffer,
                );

                let all_outputs = systems::calculate_actions_parallel(
                    &system_ctx,
                    env,
                    &id_map,
                    &mut entity_data,
                    &mut decision_buffer,
                );

                systems::apply_actions_sequential(
                    all_outputs,
                    Arc::make_mut(&mut self.pheromones),
                    Arc::make_mut(&mut self.sound),
                    Arc::make_mut(&mut self.pressure),
                    env,
                )
            };

            self.decision_buffer = decision_buffer;
            self.interaction_buffer = interaction_commands_buffer;
            result
        };

        for (l_id, amount) in &overmind_broadcasts {
            self.lineage_registry
                .set_memory_value(l_id, "overmind", *amount);
        }

        let (mut events, new_babies) = self.pass_interactions(env, &food_handles, &handles);

        self.finalize_tick(env, &mut events, &handles, new_babies);

        self.update_grids_and_environment(env);

        Ok(events)
    }

    fn build_tick_indices(&mut self) -> (Vec<hecs::Entity>, HashMap<uuid::Uuid, usize>) {
        let mut data: Vec<_> = self
            .ecs
            .query::<&Identity>()
            .iter()
            .map(|(h, i)| (h, i.id))
            .collect();
        data.sort_by_key(|d| d.1);

        let mut handles = Vec::with_capacity(data.len());
        let mut id_to_idx = HashMap::new();

        for (idx, (handle, id)) in data.into_iter().enumerate() {
            id_to_idx.insert(id, idx);
            handles.push(handle);
        }

        (handles, id_to_idx)
    }

    fn pass_social_ranks(&mut self) {
        let tick = self.tick;
        let config = &self.config;
        let mut query = self.ecs.query::<(&Metabolism, &mut Intel, &Identity)>();
        let mut data: Vec<_> = query.iter().collect();
        data.sort_by_key(|(_h, (.., ident))| ident.id);

        data.par_iter_mut().for_each(|(_, (met, intel, _ident))| {
            intel.rank = social::calculate_social_rank_components(met, intel, tick, config);
        });
    }

    fn pass_spatial_indexing(&mut self) {
        let mut query = self.ecs.query::<EntityComponents>();
        let mut spatial_data_with_ids = std::mem::take(&mut self.spatial_sort_buffer);
        spatial_data_with_ids.clear();
        spatial_data_with_ids.extend(
            query
                .iter()
                .map(|(_h, (ident, pos, _, _, met, ..))| (pos.x, pos.y, met.lineage_id, ident.id)),
        );
        spatial_data_with_ids.sort_by_key(|d| d.3);

        let mut spatial_data = std::mem::take(&mut self.spatial_data_buffer);
        spatial_data.clear();
        for (x, y, lid, _) in &spatial_data_with_ids {
            spatial_data.push((*x, *y, *lid));
        }

        self.spatial_hash
            .build_with_lineage(&spatial_data, self.width, self.height);
        self.spatial_data_buffer = spatial_data;
        self.spatial_sort_buffer = spatial_data_with_ids;
    }

    fn pass_food_indexing(&mut self) -> (Vec<hecs::Entity>, Vec<(f64, f64, f32)>) {
        let mut food_data: Vec<_> = self
            .ecs
            .query::<(&Position, &Food)>()
            .iter()
            .map(|(handle, (pos, food))| (pos.x, pos.y, handle, food.nutrient_type))
            .collect();

        food_data.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .then(a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal))
        });

        let len = food_data.len();
        let mut handles = Vec::with_capacity(len);
        let mut positions = Vec::with_capacity(len);
        let mut nutrition_data = Vec::with_capacity(len);
        for (x, y, handle, nutrient_type) in food_data {
            handles.push(handle);
            positions.push((x, y));
            nutrition_data.push((x, y, nutrient_type));
        }

        self.food_hash
            .build_parallel(&positions, self.width, self.height);
        (handles, nutrition_data)
    }

    fn pass_learning(&mut self) {
        let mut query = self.ecs.query::<(&Metabolism, &mut Intel, &Identity)>();
        let mut data: Vec<_> = query.iter().collect();
        data.sort_by_key(|(_h, (.., ident))| ident.id);

        data.par_iter_mut().for_each(|(_, (met, intel, _ident))| {
            let reinforcement = if met.energy > met.prev_energy {
                0.1
            } else if met.energy < met.prev_energy {
                -0.05
            } else {
                0.0
            };
            std::sync::Arc::make_mut(&mut intel.genotype)
                .brain
                .learn(&intel.last_activations, reinforcement as f32);
        });
    }

    fn pass_interactions(
        &mut self,
        env: &mut Environment,
        food_handles: &[hecs::Entity],
        handles: &[hecs::Entity],
    ) -> (Vec<LiveEvent>, Vec<Entity>) {
        let interaction_commands = std::mem::take(&mut self.interaction_buffer);
        let (interaction_events, new_babies) =
            self.execute_interactions(env, interaction_commands, handles, food_handles);

        for ev in &interaction_events {
            let _ = self.logger.log_event(ev.clone());
        }
        (interaction_events, new_babies)
    }

    fn update_grids_and_environment(&mut self, env: &mut Environment) {
        let phero = Arc::make_mut(&mut self.pheromones);
        let snd = Arc::make_mut(&mut self.sound);
        let press = Arc::make_mut(&mut self.pressure);

        rayon::join(
            || phero.update(),
            || {
                rayon::join(|| snd.update(), || press.update());
            },
        );

        if !self.config.world.deterministic {
            env.tick();
        } else {
            env.world_time = 500;
        }

        if self.tick.is_multiple_of(10) {
            self.update_rank_grid();
        }
    }

    fn update_environment_and_resources(&mut self, env: &mut Environment, world_seed: u64) {
        action::handle_game_modes_ecs(
            &mut self.ecs,
            &self.config,
            self.tick,
            self.width,
            self.height,
        );

        if self.tick.is_multiple_of(50) {
            for val in Arc::make_mut(&mut self.social_grid) {
                *val = 0;
            }
        }

        self.lineage_registry.decay_memory(0.99);

        let pop_count = self.get_population_count();
        environment::handle_disasters(
            env,
            pop_count,
            Arc::make_mut(&mut self.terrain),
            &mut self.rng,
            &self.config,
        );

        let (_total_plant_biomass, total_sequestration) = Arc::make_mut(&mut self.terrain).update(
            self.pop_stats.biomass_h,
            self.tick,
            world_seed,
        );

        let total_owned_forests = self
            .terrain
            .cells
            .iter()
            .filter(|c| {
                c.terrain_type == primordium_data::TerrainType::Forest && c.owner_id.is_some()
            })
            .count();

        let sequestration_bonus =
            total_owned_forests as f64 * self.config.ecosystem.sequestration_rate * 0.1;
        env.sequestrate_carbon(
            total_sequestration * self.config.ecosystem.sequestration_rate + sequestration_bonus,
        );
        env.add_carbon(pop_count as f64 * self.config.ecosystem.carbon_emission_rate);
        env.consume_oxygen(pop_count as f64 * self.config.metabolism.oxygen_consumption_rate);

        // Phase 67 Task C: Update DDA based on average fitness
        let avg_fitness = self.pop_stats.avg_fitness;
        let target_fitness = 500.0; // Target fitness threshold for DDA
        env.update_dda(avg_fitness, target_fitness, pop_count);

        // Apply DDA multiplier to solar energy injection
        let effective_solar_rate =
            self.config.ecosystem.solar_energy_rate * env.dda_solar_multiplier;
        env.available_energy += effective_solar_rate;

        env.tick();

        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut self.rng);

        let mut spawn_ctx = ecological::SpawnFoodContext {
            world: &mut self.ecs,
            env,
            terrain: &self.terrain,
            config: &self.config,
            width: self.width,
            height: self.height,
            food_count_ptr: &self.food_count,
        };
        ecological::spawn_food_ecs(&mut spawn_ctx, &mut self.rng);

        if self.food_dirty {
            let mut food_positions = std::mem::take(&mut self.food_positions_buffer);
            food_positions.clear();
            for (_handle, (pos, _)) in self
                .ecs
                .query::<(&Position, &primordium_data::Food)>()
                .iter()
            {
                food_positions.push((pos.x, pos.y));
            }
            self.food_hash
                .build_parallel(&food_positions, self.width, self.height);
            self.food_dirty = false;
            self.food_positions_buffer = food_positions;
        }
    }

    fn execute_interactions(
        &mut self,
        env: &mut Environment,
        interaction_commands: Vec<InteractionCommand>,
        entity_handles: &[hecs::Entity],
        food_handles: &[hecs::Entity],
    ) -> (Vec<LiveEvent>, Vec<Entity>) {
        let (state_cmds, struct_cmds): (Vec<_>, Vec<_>) =
            interaction_commands.into_iter().partition(|cmd| {
                matches!(
                    cmd,
                    InteractionCommand::TransferEnergy { .. }
                        | InteractionCommand::UpdateReputation { .. }
                        | InteractionCommand::Fertilize { .. }
                )
            });

        let mut interaction_ctx = primordium_core::systems::interaction::InteractionContext {
            terrain: Arc::make_mut(&mut self.terrain),
            env,
            pop_stats: Arc::make_mut(&mut self.pop_stats),
            lineage_registry: &mut self.lineage_registry,
            fossil_registry: &mut self.fossil_registry,
            config: &self.config,
            tick: self.tick,
            width: self.width,
            height: self.height,
            social_grid: Arc::make_mut(&mut self.social_grid).as_mut_slice(),
            lineage_consumption: &mut self.lineage_consumption,
            food_handles,
            spatial_hash: &self.spatial_hash,
            rng: &mut self.rng,
            food_count: &self.food_count,
            world_seed: self.config.world.seed.unwrap_or(0),
        };

        let result1 = primordium_core::systems::interaction::process_interaction_commands_ecs(
            &mut self.ecs,
            entity_handles,
            state_cmds,
            &mut interaction_ctx,
        );

        let interaction_result =
            primordium_core::systems::interaction::process_interaction_commands_ecs(
                &mut self.ecs,
                entity_handles,
                struct_cmds,
                &mut interaction_ctx,
            );

        let mut all_events = result1.events;
        all_events.extend(interaction_result.events);

        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();

        self.killed_ids = interaction_result.killed_ids;
        self.eaten_food_indices = interaction_result.eaten_food_indices;

        (all_events, interaction_result.new_babies)
    }

    pub fn update_rank_grid(&mut self) {
        let width = self.width as usize;
        let height = self.height as usize;

        let deposits: Vec<(usize, f32)> = self
            .ecs
            .query::<(&Physics, &Intel)>()
            .iter()
            .par_bridge()
            .filter_map(|(_handle, (phys, intel))| {
                if !phys.x.is_finite() || !phys.y.is_finite() {
                    return None;
                }
                let mut local_deps = Vec::new();
                let ex = phys.x as i32;
                let ey = phys.y as i32;
                let r = 3;
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = ex + dx;
                        let ny = ey + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let idx = (ny as usize * width) + nx as usize;
                            let dist_sq = dx * dx + dy * dy;
                            let weight = (1.0 - (dist_sq as f32 / (r * r) as f32).sqrt()).max(0.0);
                            if weight > 0.0 {
                                local_deps.push((idx, intel.rank * weight));
                            }
                        }
                    }
                }
                Some(local_deps)
            })
            .flatten()
            .collect();

        let mut rank_grid = vec![0.0f32; width * height];
        for (idx, val) in deposits {
            rank_grid[idx] += val;
        }
        self.cached_rank_grid = Arc::new(rank_grid);
    }
}
