use crate::model::environment::Environment;
use crate::model::world::World;
use chrono::Utc;
use primordium_core::systems::{biological, civilization, history, social, stats};
use primordium_data::LiveEvent;
use primordium_data::{Entity, Health, Identity, Intel, Metabolism, Pathogen, Physics};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::sync::Arc;

type ProposalResult = (hecs::Entity, Vec<(hecs::Entity, Pathogen)>, bool, f64);

impl World {
    pub fn finalize_tick(
        &mut self,
        env: &mut Environment,
        events: &mut Vec<LiveEvent>,
        entity_handles: &[hecs::Entity],
        new_babies: Vec<Entity>,
    ) {
        let tick = self.tick;
        self.capture_entity_snapshots();

        let active_pathogens = &self.active_pathogens;
        let spatial_hash = &self.spatial_hash;
        let killed_ids = &self.killed_ids;
        let config = &self.config;

        let proposals: Vec<ProposalResult> = {
            let mut query = self.ecs.query::<(
                &Identity,
                &mut Metabolism,
                &mut Intel,
                &mut Health,
                &Physics,
            )>();
            let mut components: Vec<_> = query.iter().collect();
            components.sort_by_key(|(_h, (ident, ..))| ident.id);
            let population_count = components.len();

            components
                .par_iter_mut()
                .map(|(handle, (identity, met, intel, health, phys))| {
                    let u = identity.id.as_u128();
                    let mut seed = tick
                        .wrapping_add(config.world.seed.unwrap_or(0))
                        .wrapping_mul(0x517CC1B727220A95);
                    seed ^= (u >> 64) as u64;
                    seed = seed.wrapping_mul(0x517CC1B727220A95);
                    seed ^= u as u64;

                    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

                    let mut context = biological::BiologicalContext::new(
                        population_count,
                        config,
                        tick,
                        &mut rng,
                    );

                    let metabolic_consumption = biological::biological_system_components(
                        met,
                        intel,
                        health,
                        phys,
                        &mut context,
                    );

                    for p in active_pathogens {
                        if rng.gen_bool(0.005) {
                            biological::try_infect_components(health, p, &mut rng);
                        }
                    }

                    let mut infections = Vec::with_capacity(self.active_pathogens.len() * 5);
                    if let Some(p) = &health.pathogen {
                        spatial_hash.query_callback(phys.x, phys.y, 2.0, |n_idx| {
                            let n_handle = entity_handles[n_idx];
                            if n_handle != *handle {
                                infections.push((n_handle, p.clone()));
                            }
                        });
                    }

                    let is_dead = killed_ids.contains(&identity.id) || met.energy <= 0.0;

                    (*handle, infections, is_dead, metabolic_consumption)
                })
                .collect()
        };

        for (_handle, infections, _, _) in &proposals {
            for (n_handle, pathogen) in infections {
                if let Ok(mut n_health) = self.ecs.get::<&mut Health>(*n_handle) {
                    biological::try_infect_components(&mut n_health, pathogen, &mut self.rng);
                }
            }
        }

        self.process_deaths(&proposals, tick, env, events);

        // Phase 67 Task B: Aggregate metabolic consumption (heat loss)
        let total_metabolic_consumption: f64 = proposals
            .iter()
            .map(|(_, _, _, consumption)| consumption)
            .sum();

        // Phase 67 Task B: Heat loss from metabolic consumption
        // This is thermodynamically accounted as energy dissipated from the system
        env.available_energy -= total_metabolic_consumption;

        self.process_births(new_babies);
        self.finalize_snapshots(env, events);
        self.finalize_civilization(entity_handles);
        self.finalize_stats(env, tick);
    }

    pub fn process_deaths(
        &mut self,
        proposals: &[ProposalResult],
        tick: u64,
        env: &mut Environment,
        events: &mut Vec<LiveEvent>,
    ) {
        let mut dead_handles = Vec::with_capacity(proposals.len());
        for (handle, _, is_dead, _) in proposals {
            if *is_dead {
                dead_handles.push(*handle);
            }
        }

        for handle in dead_handles {
            if let Ok((met, identity, phys, intel)) = self
                .ecs
                .remove::<(Metabolism, Identity, Physics, Intel)>(handle)
            {
                self.lineage_registry.record_death(met.lineage_id);

                // Create Death event for starvation deaths
                let ev = LiveEvent::Death {
                    id: identity.id,
                    age: tick - met.birth_tick,
                    offspring: met.offspring_count,
                    tick,
                    timestamp: Utc::now().to_rfc3339(),
                    cause: "Starvation".to_string(),
                    x: Some(phys.x),
                    y: Some(phys.y),
                };
                events.push(ev);

                if let Some(legend) =
                    social::archive_if_legend_components(&identity, &met, &intel, &phys, tick)
                {
                    let _ = self.world_logger_archive_legend(legend.clone());
                    history::update_best_legend(
                        &mut self.lineage_registry,
                        &mut self.best_legends,
                        legend,
                    );
                }
                let fertilize_amount =
                    (met.max_energy * self.config.ecosystem.corpse_fertility_mult as f64) as f32
                        / 100.0;

                // Return energy to global pool (Energy recycling)
                // Remaining energy + 50% of body mass (max_energy)
                let recycled_energy = met.energy + met.max_energy * 0.5;
                env.available_energy += recycled_energy;

                let terrain = Arc::make_mut(&mut self.terrain);
                terrain.fertilize(phys.x, phys.y, fertilize_amount);
                terrain.add_biomass(phys.x, phys.y, fertilize_amount * 10.0);

                let _ = self.ecs.despawn(handle);
            }
        }
    }

    fn world_logger_archive_legend(&self, legend: primordium_data::Legend) -> anyhow::Result<()> {
        self.logger.archive_legend(legend)
    }

    pub fn process_births(&mut self, new_babies: Vec<Entity>) {
        let babies_to_spawn: Vec<Entity> = if self.config.ecosystem.spawn_rate_limit_enabled {
            let limit = self.config.ecosystem.max_entities_per_tick;

            new_babies.into_iter().take(limit).collect::<Vec<Entity>>()
        } else {
            new_babies
        };

        self.ecs
            .spawn_batch(babies_to_spawn.into_iter().map(|baby| {
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
    }

    pub fn finalize_snapshots(&mut self, env: &mut Environment, events: &mut Vec<LiveEvent>) {
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
            let reg_clone = self.lineage_registry.clone();
            let fossil_clone = self.fossil_registry.clone();

            let _ = self
                .logger
                .save_lineages_async(reg_clone.clone(), format!("{}/lineages.json", self.log_dir));
            let _ = self.logger.save_fossils_async(
                fossil_clone.clone(),
                format!("{}/fossils.json.gz", self.log_dir),
            );
            let _ = self.logger.sync_to_storage_async(reg_clone, fossil_clone);
            let snap_ev = LiveEvent::Snapshot {
                tick: self.tick,
                stats: (*self.pop_stats).clone(),
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

        if self
            .tick
            .is_multiple_of(self.config.world.lineage_prune_interval)
        {
            self.lineage_registry.prune_extinct_old(
                self.tick,
                self.config.world.lineage_extinction_age_threshold,
            );
            self.lineage_registry
                .prune_by_count(self.config.world.max_lineages);
        }
    }

    pub fn finalize_civilization(&mut self, entity_handles: &[hecs::Entity]) {
        civilization::handle_outposts_ecs(
            Arc::make_mut(&mut self.terrain),
            &mut self.ecs,
            &civilization::OutpostContext {
                entity_handles,
                spatial_hash: &self.spatial_hash,
                snapshots: &self.entity_snapshots,
                width: self.width,
                silo_cap: self.config.social.silo_energy_capacity,
                outpost_cap: self.config.social.outpost_energy_capacity,
            },
        );

        civilization::resolve_contested_ownership(
            Arc::make_mut(&mut self.terrain),
            self.width,
            self.height,
            &self.spatial_hash,
            &self.entity_snapshots,
            &self.lineage_registry,
        );
        civilization::resolve_outpost_upgrades(
            Arc::make_mut(&mut self.terrain),
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
                Arc::make_mut(&mut self.terrain),
                self.width,
                self.height,
                &self.lineage_registry,
            );
        }
    }

    pub fn finalize_stats(&mut self, env: &mut Environment, tick: u64) {
        // Optimization: update_stats only needs a slice of entity snapshots which we already have
        let food_count = self.ecs.query::<&primordium_data::Food>().iter().count();

        stats::update_stats(
            &stats::StatsInput {
                tick,
                entities: &self.entity_snapshots,
                food_count,
                carbon_level: env.carbon_level,
                mutation_scale: 1.0,
                terrain: &self.terrain,
            },
            Arc::make_mut(&mut self.pop_stats),
            Arc::make_mut(&mut self.hall_of_fame),
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
