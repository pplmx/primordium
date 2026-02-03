use crate::model::environment::Environment;
use crate::model::history::LiveEvent;
use crate::model::interaction::InteractionCommand;
use chrono::Utc;
use hecs;
use primordium_data::{Entity, Food, Health, Identity, Intel, Metabolism, Physics, Position};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::model::brain::BrainLogic;
use crate::model::world::{systems, EntityComponents, SystemContext, World};
use primordium_core::systems::{
    action, biological, civilization, ecological, environment, history, interaction, social, stats,
};

impl World {
    pub fn update(&mut self, env: &mut Environment) -> anyhow::Result<Vec<LiveEvent>> {
        self.tick += 1;
        let world_seed = self.config.world.seed.unwrap_or(0);

        self.update_environment_and_resources(env, world_seed);

        self.pass_social_ranks();
        self.pass_spatial_indexing();
        let food_handles = self.pass_food_indexing();
        self.capture_entity_snapshots();
        self.pass_learning();

        self.influence.update(&self.entity_snapshots);

        let overmind_broadcasts = self.pass_perception_and_action(env, &food_handles, world_seed);

        for (l_id, amount) in &overmind_broadcasts {
            self.lineage_registry
                .set_memory_value(l_id, "overmind", *amount);
        }

        let (mut events, new_babies) = self.pass_interactions(env, &food_handles);

        let handles = self.get_sorted_handles();
        self.finalize_tick(env, &mut events, &handles, new_babies);

        self.update_grids_and_environment(env);

        Ok(events)
    }

    fn pass_social_ranks(&mut self) {
        let tick = self.tick;
        let config = &self.config;
        let mut query = self.ecs.query::<(&Metabolism, &mut Intel)>();
        let mut data: Vec<_> = query.iter().collect();
        data.par_iter_mut().for_each(|(_, (met, intel))| {
            intel.rank = social::calculate_social_rank_components(met, intel, tick, config);
        });
    }

    fn pass_spatial_indexing(&mut self) {
        let mut query = self.ecs.query::<(&Position, &Metabolism)>();
        let mut spatial_data = std::mem::take(&mut self.spatial_data_buffer);
        spatial_data.clear();
        for (_h, (pos, met)) in query.iter() {
            spatial_data.push((pos.x, pos.y, met.lineage_id));
        }
        self.spatial_hash
            .build_with_lineage(&spatial_data, self.width, self.height);
        self.spatial_data_buffer = spatial_data;
    }

    fn pass_food_indexing(&mut self) -> Vec<hecs::Entity> {
        let mut handles = Vec::new();
        let mut positions = Vec::new();
        for (handle, (pos, _)) in self.ecs.query::<(&Position, &Food)>().iter() {
            handles.push(handle);
            positions.push((pos.x, pos.y));
        }
        self.food_hash
            .build_parallel(&positions, self.width, self.height);
        handles
    }

    fn pass_learning(&mut self) {
        let mut query = self.ecs.query::<(&Metabolism, &mut Intel)>();
        let mut data: Vec<_> = query.iter().collect();
        data.par_iter_mut().for_each(|(_, (met, intel))| {
            let reinforcement = if met.energy > met.prev_energy {
                0.1
            } else {
                -0.05
            };
            intel
                .genotype
                .brain
                .learn(&intel.last_activations, reinforcement as f32);
        });
    }

    fn pass_perception_and_action(
        &mut self,
        env: &mut Environment,
        food_handles: &[hecs::Entity],
        world_seed: u64,
    ) -> Vec<(uuid::Uuid, f32)> {
        let mut query = self.ecs.query::<EntityComponents>();
        let mut entity_data: Vec<_> = query.iter().collect();
        entity_data.par_sort_by_key(|(_h, (i, ..))| i.id);

        let id_map: HashMap<uuid::Uuid, usize> = entity_data
            .iter()
            .enumerate()
            .map(|(idx, (_, (i, ..)))| (i.id, idx))
            .collect();

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
            food_handles,
            world_seed,
        };

        let mut interaction_commands_buffer = std::mem::take(&mut self.interaction_buffer);
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);

        systems::perceive_and_decide_internal(
            &system_ctx,
            env,
            self.pop_stats.biomass_c,
            &mut entity_data,
            &id_map,
            &mut interaction_commands_buffer,
            &mut decision_buffer,
        );

        let overmind_broadcasts = systems::execute_actions_internal(
            &system_ctx,
            env,
            &id_map,
            &mut entity_data,
            &mut decision_buffer,
        );

        self.decision_buffer = decision_buffer;
        self.interaction_buffer = interaction_commands_buffer;
        overmind_broadcasts
    }

    fn pass_interactions(
        &mut self,
        env: &mut Environment,
        food_handles: &[hecs::Entity],
    ) -> (Vec<LiveEvent>, Vec<Entity>) {
        let interaction_commands = std::mem::take(&mut self.interaction_buffer);
        let handles = self.get_sorted_handles();
        let (interaction_events, new_babies) =
            self.execute_interactions(env, interaction_commands, &handles, food_handles);

        for ev in &interaction_events {
            let _ = self.logger.log_event(ev.clone());
        }
        (interaction_events, new_babies)
    }

    fn update_grids_and_environment(&mut self, env: &mut Environment) {
        rayon::join(
            || self.pheromones.update(),
            || {
                rayon::join(|| self.sound.update(), || self.pressure.update());
            },
        );

        if self.config.world.deterministic {
            env.tick_deterministic(self.tick);
        } else {
            env.tick();
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
            for val in &mut self.social_grid {
                *val = 0;
            }
        }

        self.lineage_registry.decay_memory(0.99);

        let pop_count = self.get_population_count();
        environment::handle_disasters(
            env,
            pop_count,
            &mut self.terrain,
            &mut self.rng,
            &self.config,
        );

        let (_total_plant_biomass, total_sequestration) =
            self.terrain
                .update(self.pop_stats.biomass_h, self.tick, world_seed);

        let total_owned_forests = self
            .terrain
            .cells
            .iter()
            .filter(|c| {
                c.terrain_type == crate::model::terrain::TerrainType::Forest && c.owner_id.is_some()
            })
            .count();

        let sequestration_bonus =
            total_owned_forests as f64 * self.config.ecosystem.sequestration_rate * 0.1;
        env.sequestrate_carbon(
            total_sequestration * self.config.ecosystem.sequestration_rate + sequestration_bonus,
        );
        env.add_carbon(pop_count as f64 * self.config.ecosystem.carbon_emission_rate);
        env.consume_oxygen(pop_count as f64 * self.config.metabolism.oxygen_consumption_rate);
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
                .query::<(&Position, &crate::model::food::Food)>()
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

        let mut interaction_ctx = interaction::InteractionContext {
            terrain: &mut self.terrain,
            env,
            pop_stats: &mut self.pop_stats,
            lineage_registry: &mut self.lineage_registry,
            fossil_registry: &mut self.fossil_registry,
            config: &self.config,
            tick: self.tick,
            width: self.width,
            height: self.height,
            social_grid: &mut self.social_grid,
            lineage_consumption: &mut self.lineage_consumption,
            food_handles,
            spatial_hash: &self.spatial_hash,
            rng: &mut self.rng,
            food_count: &self.food_count,
        };

        let result1 = interaction::process_interaction_commands_ecs(
            &mut self.ecs,
            entity_handles,
            state_cmds,
            &mut interaction_ctx,
        );

        let interaction_result = interaction::process_interaction_commands_ecs(
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

    fn update_rank_grid(&mut self) {
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

    fn finalize_tick(
        &mut self,
        env: &mut Environment,
        events: &mut Vec<LiveEvent>,
        entity_handles: &[hecs::Entity],
        new_babies: Vec<Entity>,
    ) {
        let config = &self.config;
        let tick = self.tick;
        let active_pathogens = &self.active_pathogens;
        let spatial_hash = &self.spatial_hash;
        let killed_ids = &self.killed_ids;

        // 1. Parallel pass: Update systems and collect proposals
        let proposals: Vec<_> = {
            let mut query = self.ecs.query::<(
                &Identity,
                &mut Metabolism,
                &mut Intel,
                &mut Health,
                &Physics,
            )>();
            let mut components: Vec<_> = query.iter().collect();
            let population_count = components.len();

            components
                .par_iter_mut()
                .map(|(handle, (identity, met, intel, health, phys))| {
                    let mut rng =
                        rand_chacha::ChaCha8Rng::seed_from_u64(tick ^ identity.id.as_u128() as u64);

                    // a. Basic biological update
                    biological::biological_system_components(
                        met,
                        intel,
                        health,
                        phys,
                        population_count,
                        config,
                        &mut rng,
                    );

                    // b. Infection check (Incoming from environment)
                    for p in active_pathogens {
                        if rng.gen_bool(0.005) {
                            biological::try_infect_components(health, p, &mut rng);
                        }
                    }

                    // c. Infection Spread proposal (Outgoing to neighbors)
                    let mut infections = Vec::new();
                    if let Some(p) = &health.pathogen {
                        spatial_hash.query_callback(phys.x, phys.y, 2.0, |n_idx| {
                            let n_handle = entity_handles[n_idx];
                            if n_handle != *handle {
                                infections.push((n_handle, p.clone()));
                            }
                        });
                    }

                    // d. Death check
                    let is_dead = killed_ids.contains(&identity.id) || met.energy <= 0.0;

                    (*handle, infections, is_dead)
                })
                .collect()
        };

        // 2. Sequential pass: Apply proposals
        for (_handle, infections, _) in &proposals {
            for (n_handle, pathogen) in infections {
                if let Ok(mut n_health) = self.ecs.get::<&mut Health>(*n_handle) {
                    biological::try_infect_components(&mut n_health, pathogen, &mut self.rng);
                }
            }
        }

        let mut dead_handles = Vec::new();
        for (handle, _, is_dead) in proposals {
            if is_dead {
                dead_handles.push(handle);
            }
        }

        for handle in dead_handles {
            let (dead_info, extra) = {
                if let (Ok(met), Ok(identity)) = (
                    self.ecs.get::<&Metabolism>(handle),
                    self.ecs.get::<&Identity>(handle),
                ) {
                    let info = ((*met).clone(), (*identity).clone());
                    let extra = if let (Ok(phys), Ok(intel)) = (
                        self.ecs.get::<&Physics>(handle),
                        self.ecs.get::<&Intel>(handle),
                    ) {
                        Some(((*phys).clone(), (*intel).clone()))
                    } else {
                        None
                    };
                    (Some(info), extra)
                } else {
                    (None, None)
                }
            };

            if let Some((met, identity)) = dead_info {
                self.lineage_registry.record_death(met.lineage_id);

                if let Some((phys, intel)) = extra {
                    if let Some(legend) =
                        social::archive_if_legend_components(&identity, &met, &intel, &phys, tick)
                    {
                        let _ = self.logger.archive_legend(legend.clone());
                        history::update_best_legend(
                            &mut self.lineage_registry,
                            &mut self.best_legends,
                            legend,
                        );
                    }
                    let fertilize_amount = (met.max_energy
                        * self.config.ecosystem.corpse_fertility_mult as f64)
                        as f32
                        / 100.0;
                    self.terrain.fertilize(phys.x, phys.y, fertilize_amount);
                    self.terrain
                        .add_biomass(phys.x, phys.y, fertilize_amount * 10.0);
                }
                let _ = self.ecs.despawn(handle);
            }
        }

        self.ecs.spawn_batch(new_babies.into_iter().map(|baby| {
            (
                baby.identity,
                baby.position,
                baby.velocity,
                baby.appearance,
                baby.physics,
                baby.metabolism,
                baby.health,
                baby.intel,
            )
        }));

        if !self.eaten_food_indices.is_empty() {
            self.food_dirty = true;
        }

        self.killed_ids.clear();
        self.eaten_food_indices.clear();

        if self.tick.is_multiple_of(self.config.world.fossil_interval) {
            let outpost_counts = civilization::count_outposts_by_lineage(&self.terrain);
            self.lineage_registry.check_goals(
                self.tick,
                &self.social_grid,
                self.width,
                self.height,
                &outpost_counts,
            );
            self.lineage_registry.prune();
            let _ = self.logger.save_lineages_async(
                self.lineage_registry.clone(),
                format!("{}/lineages.json", self.log_dir),
            );
            let _ = self.logger.save_fossils_async(
                self.fossil_registry.clone(),
                format!("{}/fossils.json.gz", self.log_dir),
            );
            let _ = self
                .logger
                .sync_to_storage_async(self.lineage_registry.clone(), self.fossil_registry.clone());
            let snap_ev = LiveEvent::Snapshot {
                tick: self.tick,
                stats: self.pop_stats.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            if let Some(ref storage) = self.logger.storage {
                let snapshot = self.create_snapshot(None);
                let world_data =
                    rkyv::to_bytes::<crate::model::snapshot::WorldSnapshot, 4096>(&snapshot)
                        .map(|v| v.to_vec())
                        .unwrap_or_default();
                storage.save_snapshot(
                    self.tick,
                    self.pop_stats.population as u32,
                    env.carbon_level,
                    self.pop_stats.biomass_h + self.pop_stats.biomass_c,
                    world_data,
                );
            }

            let _ = self.logger.log_event(snap_ev.clone());
            events.push(snap_ev);
            history::handle_fossilization(
                &self.lineage_registry,
                &mut self.fossil_registry,
                &mut self.best_legends,
                self.tick,
            );
            self.lineage_registry.prune();
        }

        civilization::handle_outposts_ecs(
            &mut self.terrain,
            &mut self.ecs,
            entity_handles,
            &self.spatial_hash,
            &self.entity_snapshots,
            self.width,
            self.config.social.silo_energy_capacity,
            self.config.social.outpost_energy_capacity,
        );

        civilization::resolve_contested_ownership(
            &mut self.terrain,
            self.width,
            self.height,
            &self.spatial_hash,
            &self.entity_snapshots,
            &self.lineage_registry,
        );
        civilization::resolve_outpost_upgrades(
            &mut self.terrain,
            self.width,
            self.height,
            &self.spatial_hash,
            &self.entity_snapshots,
            &self.lineage_registry,
        );

        if self
            .tick
            .is_multiple_of(self.config.world.power_grid_interval)
        {
            civilization::resolve_power_grid(
                &mut self.terrain,
                self.width,
                self.height,
                &self.lineage_registry,
            );
        }

        let snapshot = self.create_snapshot(None);
        self.cached_terrain = snapshot.terrain;
        self.cached_pheromones = snapshot.pheromones;
        self.cached_sound = snapshot.sound;
        self.cached_pressure = snapshot.pressure;
        self.cached_influence = snapshot.influence;
        self.cached_social_grid = snapshot.social_grid;
        self.cached_rank_grid = snapshot.rank_grid;

        stats::update_stats(
            tick,
            &snapshot.entities,
            snapshot.food.len(),
            env.carbon_level,
            1.0,
            &mut self.pop_stats,
            &mut self.hall_of_fame,
            &self.terrain,
        );

        history::handle_fossilization(
            &self.lineage_registry,
            &mut self.fossil_registry,
            &mut self.best_legends,
            tick,
        );

        if tick.is_multiple_of(10) {
            self.update_rank_grid();
        }
    }
}
