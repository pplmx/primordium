use crate::model::config::AppConfig;
use crate::model::history::{
    FossilRegistry, HallOfFame, HistoryLogger, LiveEvent, PopulationStats,
};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::lineage_registry::LineageRegistry;
use crate::model::state::pheromone::{PheromoneGrid, PheromoneType};
use crate::model::state::terrain::TerrainGrid;
use crate::model::systems::{action, biological, ecological, environment, intel, social, stats};
use chrono::Utc;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A lightweight snapshot of an entity's state for read-only access during update cycles.
#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub id: uuid::Uuid,
    pub lineage_id: uuid::Uuid,
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub birth_tick: u64,
    pub offspring_count: u32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub rank: f32,
    pub status: crate::model::state::entity::EntityStatus,
}

/// NEW: Phase 41 - Commands generated during parallel interaction pass
#[derive(Debug)]
pub enum InteractionCommand {
    Kill {
        target_idx: usize,
        attacker_lineage: uuid::Uuid,
        energy_gain: f64,
        cause: String,
    },
    TransferEnergy {
        target_idx: usize,
        amount: f64,
    },
    Birth {
        parent_idx: usize,
        baby: Box<Entity>,
        genetic_distance: f32,
    },
    EatFood {
        food_index: usize,
        attacker_idx: usize,
        energy_gain: f64,
    },
    Infect {
        target_idx: usize,
        pathogen: crate::model::state::pathogen::Pathogen,
    },
    Fertilize {
        x: f64,
        y: f64,
        amount: f32,
    },
    UpdateReputation {
        target_idx: usize,
        delta: f32,
    },
    TribalSplit {
        target_idx: usize,
        new_color: (u8, u8, u8),
    },
    TribalTerritory {
        x: f64,
        y: f64,
        is_war: bool,
    },
    Bond {
        target_idx: usize,
        partner_id: uuid::Uuid,
    },
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub width: u16,
    pub height: u16,
    pub entities: Vec<Entity>,
    pub food: Vec<Food>,
    pub tick: u64,
    #[serde(skip, default = "HistoryLogger::new_dummy")]
    pub logger: HistoryLogger,
    pub config: AppConfig,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub spatial_hash: SpatialHash,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub food_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid,
    pub social_grid: Vec<Vec<u8>>, // NEW: 0: Normal, 1: Peace, 2: War
    pub lineage_registry: LineageRegistry,
    pub fossil_registry: FossilRegistry,
    pub log_dir: String,
    pub active_pathogens: Vec<crate::model::state::pathogen::Pathogen>,

    #[serde(skip, default)]
    pub best_legends: std::collections::HashMap<uuid::Uuid, crate::model::history::Legend>,

    #[serde(skip, default)]
    killed_ids: HashSet<uuid::Uuid>,
    #[serde(skip, default)]
    eaten_food_indices: HashSet<usize>,
    #[serde(skip, default)]
    new_babies: Vec<Entity>,
    #[serde(skip, default)]
    alive_entities: Vec<Entity>,
    #[serde(skip, default)]
    perception_buffer: Vec<[f32; 16]>,
    #[serde(skip, default)]
    decision_buffer: Vec<([f32; 9], [f32; 6])>,
    #[serde(skip, default)]
    energy_transfers: Vec<(usize, f64)>,
    #[serde(skip, default)]
    lineage_consumption: Vec<(uuid::Uuid, f64)>,
}

impl World {
    pub fn new(initial_population: usize, config: AppConfig) -> anyhow::Result<Self> {
        Self::new_at(initial_population, config, "logs")
    }

    pub fn new_at(
        initial_population: usize,
        config: AppConfig,
        log_dir: &str,
    ) -> anyhow::Result<Self> {
        let mut rng = rand::thread_rng();
        let mut entities = Vec::with_capacity(initial_population);
        let logger = HistoryLogger::new_at(log_dir)?;
        let mut lineage_registry = LineageRegistry::new();
        for _ in 0..initial_population {
            let e = Entity::new(
                rng.gen_range(1.0..config.world.width as f64 - 1.0),
                rng.gen_range(1.0..config.world.height as f64 - 1.0),
                0,
            );
            lineage_registry.record_birth(e.metabolism.lineage_id, 1, 0);
            entities.push(e);
        }

        let mut food = Vec::new();
        for _ in 0..config.world.initial_food {
            let x = rng.gen_range(1..config.world.width - 1);
            let y = rng.gen_range(1..config.world.height - 1);
            let n_type = rng.gen_range(0.0..1.0);
            food.push(Food::new(x, y, n_type));
        }
        let terrain = TerrainGrid::generate(
            config.world.width,
            config.world.height,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(42),
        );
        let pheromones = PheromoneGrid::new(config.world.width, config.world.height);

        Ok(Self {
            width: config.world.width,
            height: config.world.height,
            entities,
            food,
            tick: 0,
            logger,
            spatial_hash: SpatialHash::new(5.0),
            food_hash: SpatialHash::new(5.0),
            pop_stats: PopulationStats::new(),
            hall_of_fame: HallOfFame::new(),
            terrain,
            pheromones,
            social_grid: vec![vec![0; config.world.width as usize]; config.world.height as usize],
            lineage_registry,
            config,
            fossil_registry: FossilRegistry::default(),

            log_dir: log_dir.to_string(),
            active_pathogens: Vec::new(),
            best_legends: std::collections::HashMap::new(),
            killed_ids: HashSet::new(),
            eaten_food_indices: HashSet::new(),
            new_babies: Vec::new(),
            alive_entities: Vec::new(),
            perception_buffer: Vec::new(),
            decision_buffer: Vec::new(),
            energy_transfers: Vec::new(),
            lineage_consumption: Vec::new(),
        })
    }

    pub fn load_persistent(&mut self) -> anyhow::Result<()> {
        self.lineage_registry =
            LineageRegistry::load(format!("{}/lineages.json", self.log_dir)).unwrap_or_default();
        self.fossil_registry =
            FossilRegistry::load(&format!("{}/fossils.json", self.log_dir)).unwrap_or_default();
        Ok(())
    }

    fn update_best_legend(&mut self, legend: crate::model::history::Legend) {
        let entry = self
            .best_legends
            .entry(legend.lineage_id)
            .or_insert_with(|| legend.clone());

        let current_score = entry.lifespan as f64 * 0.5 + entry.offspring_count as f64 * 10.0;
        let new_score = legend.lifespan as f64 * 0.5 + legend.offspring_count as f64 * 10.0;

        if new_score > current_score {
            *entry = legend;
        }
    }

    fn handle_fossilization(&mut self) {
        let extinct = self.lineage_registry.get_extinct_lineages();
        for l_id in extinct {
            if let Some(legend) = self.best_legends.remove(&l_id) {
                if !self
                    .fossil_registry
                    .fossils
                    .iter()
                    .any(|f| f.lineage_id == l_id)
                {
                    if let Some(record) = self.lineage_registry.lineages.get(&l_id) {
                        let fossil = crate::model::history::Fossil {
                            lineage_id: l_id,
                            name: record.name.clone(),
                            color_rgb: legend.color_rgb,
                            avg_lifespan: legend.lifespan as f64,
                            max_generation: record.max_generation,
                            total_offspring: record.total_entities_produced as u32,
                            extinct_tick: self.tick,
                            peak_population: record.peak_population,
                            genotype: legend.genotype.clone(),
                        };
                        self.fossil_registry.add_fossil(fossil);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, env: &mut Environment) -> anyhow::Result<Vec<LiveEvent>> {
        let mut events = Vec::new();
        self.tick += 1;
        let mut rng = rand::thread_rng();

        action::handle_game_modes(
            &mut self.entities,
            &self.config,
            self.tick,
            self.width,
            self.height,
        );
        self.pheromones.decay();

        environment::handle_disasters(
            env,
            self.entities.len(),
            &mut self.terrain,
            &mut rng,
            &self.config,
        );

        // Slow decay of social grid overrides (revert to 0: Normal)
        if self.tick.is_multiple_of(50) {
            for row in &mut self.social_grid {
                for cell in row {
                    if *cell > 0 && rng.gen_bool(0.1) {
                        *cell = 0;
                    }
                }
            }
        }

        let total_plant_biomass = self.terrain.update(self.pop_stats.biomass_h);

        env.sequestrate_carbon(total_plant_biomass * 0.00001);
        let animal_emission = (self.entities.len() as f64 * 0.01) * env.metabolism_multiplier();
        env.add_carbon(animal_emission);
        env.tick();

        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut rng);
        ecological::spawn_food(
            &mut self.food,
            env,
            &self.terrain,
            self.config.world.max_food,
            self.width,
            self.height,
            &mut rng,
        );

        self.food_hash.clear();
        for (i, f) in self.food.iter().enumerate() {
            self.food_hash.insert(f64::from(f.x), f64::from(f.y), i);
        }

        let mut current_entities = std::mem::take(&mut self.entities);
        let positions: Vec<(f64, f64)> = current_entities
            .iter()
            .map(|e| (e.physics.x, e.physics.y))
            .collect();
        self.spatial_hash.build_parallel(&positions);

        let entity_snapshots: Vec<EntitySnapshot> = current_entities
            .iter()
            .map(|e| EntitySnapshot {
                id: e.id,
                lineage_id: e.metabolism.lineage_id,
                x: e.physics.x,
                y: e.physics.y,
                energy: e.metabolism.energy,
                birth_tick: e.metabolism.birth_tick,
                offspring_count: e.metabolism.offspring_count,
                r: e.physics.r,
                g: e.physics.g,

                b: e.physics.b,
                rank: e.intel.rank,
                status: e.status(
                    e.intel.genotype.max_energy * 0.5,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
            })
            .collect();

        let mut killed_ids = std::mem::take(&mut self.killed_ids);
        let mut eaten_food_indices = std::mem::take(&mut self.eaten_food_indices);
        let mut new_babies = std::mem::take(&mut self.new_babies);
        let mut alive_entities = std::mem::take(&mut self.alive_entities);
        let mut perception_buffer = std::mem::take(&mut self.perception_buffer);
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);
        let mut energy_transfers = std::mem::take(&mut self.energy_transfers);

        killed_ids.clear();
        eaten_food_indices.clear();
        new_babies.clear();
        alive_entities.clear();
        energy_transfers.clear();

        // Pass 0: Learning (Hebbian Plasticity)
        // Adjust weights based on Reinforcement (Energy Delta) from previous tick
        current_entities.par_iter_mut().for_each(|e| {
            // Reinforcement Signal: Normalized Energy Gain/Loss
            let energy_delta = e.metabolism.energy - e.metabolism.prev_energy;

            // SOCIAL REINFORCEMENT: Alpha proximity reward
            // If near Alpha of same tribe, and energy is stable, reinforce
            // (Implemented during perception pass via herding bonus,
            // but here we reinforce neural connections)

            let reinforcement = (energy_delta / e.metabolism.max_energy.max(1.0)) as f32;

            // Apply learning if non-zero
            e.intel.genotype.brain.learn(
                e.intel.last_inputs,
                e.intel.last_hidden,
                reinforcement * 10.0,
            );

            // Reset prev_energy for next delta
            e.metabolism.prev_energy = e.metabolism.energy;

            // Update Social Rank
            e.intel.rank = social::calculate_social_rank(e, self.tick);

            // Phase 51: Bond Persistence Check
            if let Some(p_id) = e.intel.bonded_to {
                if !entity_snapshots.iter().any(|s| s.id == p_id) {
                    e.intel.bonded_to = None;
                }
            }
        });

        current_entities
            .par_iter()
            .enumerate()
            .map(|(i, e)| {
                let (dx_f, dy_f, f_type) =
                    ecological::sense_nearest_food(e, &self.food, &self.food_hash);
                let sensing_radius = e.physics.sensing_range;
                let nearby_indices =
                    self.spatial_hash
                        .query(e.physics.x, e.physics.y, sensing_radius);
                let (pheromone_food, _, pheromone_a, pheromone_b) =
                    self.pheromones
                        .sense_all(e.physics.x, e.physics.y, sensing_radius / 2.0);
                let tribe_count = nearby_indices
                    .iter()
                    .filter(|&&n_idx| {
                        n_idx != i && social::are_same_tribe(e, &current_entities[n_idx])
                    })
                    .count();

                // LEADERSHIP VECTOR (Alpha Influence)
                // Find highest ranking tribe member nearby
                let mut alpha_dx = 0.0;
                let mut alpha_dy = 0.0;
                let mut highest_rank = e.intel.rank;

                for &n_idx in &nearby_indices {
                    if n_idx != i && social::are_same_tribe(e, &current_entities[n_idx]) {
                        let n_rank = current_entities[n_idx].intel.rank;
                        if n_rank > highest_rank {
                            highest_rank = n_rank;
                            alpha_dx = current_entities[n_idx].physics.x - e.physics.x;
                            alpha_dy = current_entities[n_idx].physics.y - e.physics.y;
                        }
                    }
                }

                // Normalize vector
                let dist = (alpha_dx * alpha_dx + alpha_dy * alpha_dy)
                    .sqrt()
                    .max(0.001);
                let kin_vec_x = if highest_rank > e.intel.rank {
                    alpha_dx / dist
                } else {
                    0.0
                };
                let kin_vec_y = if highest_rank > e.intel.rank {
                    alpha_dy / dist
                } else {
                    0.0
                };
                let mut wall_proximity = 0.0;
                if e.physics.x < 5.0
                    || e.physics.x > (self.width - 5) as f64
                    || e.physics.y < 5.0
                    || e.physics.y > (self.height - 5) as f64
                {
                    wall_proximity = 1.0;
                }

                // Hearing: Average vocalization of nearby entities
                let mut heard_signal = 0.0;
                let mut heard_count = 0;
                for &n_idx in &nearby_indices {
                    if n_idx != i {
                        // We hear what they said LAST tick
                        heard_signal += current_entities[n_idx].intel.last_vocalization;
                        heard_count += 1;
                    }
                }
                let avg_hearing = if heard_count > 0 {
                    heard_signal / heard_count as f32
                } else {
                    0.0
                };

                // SYMBIOSIS: Sense partner energy if bonded
                let mut partner_energy = 0.0;
                if let Some(p_id) = e.intel.bonded_to {
                    if let Some(p_idx) = entity_snapshots.iter().position(|s| s.id == p_id) {
                        partner_energy = (entity_snapshots[p_idx].energy
                            / e.metabolism.max_energy.max(1.0))
                            as f32;
                    }
                }

                [
                    (dx_f / (sensing_radius * 4.0)).clamp(-1.0, 1.0) as f32,
                    (dy_f / (sensing_radius * 4.0)).clamp(-1.0, 1.0) as f32,
                    (e.metabolism.energy / e.metabolism.max_energy.max(1.0)) as f32,
                    (nearby_indices.len().saturating_sub(1) as f32 / 10.0).min(1.0),
                    pheromone_food,
                    (tribe_count as f32 / 5.0).min(1.0),
                    kin_vec_x as f32,
                    kin_vec_y as f32,
                    pheromone_a,
                    pheromone_b,
                    wall_proximity as f32,
                    (e.metabolism.birth_tick as f32 / self.tick.max(1) as f32).min(1.0),
                    f_type,
                    e.metabolism.trophic_potential,
                    avg_hearing,    // Input 14
                    partner_energy, // Input 15 (Index 21 in brain)
                ]
            })
            .collect_into_vec(&mut perception_buffer);

        current_entities
            .par_iter()
            .zip(perception_buffer.par_iter())
            .map(|(e, inputs)| {
                intel::brain_forward(&e.intel.genotype.brain, *inputs, e.intel.last_hidden)
            })
            .collect_into_vec(&mut decision_buffer);

        let pheromone_proposals: Vec<Vec<(f64, f64, PheromoneType, f32)>> = current_entities
            .par_iter_mut()
            .enumerate()
            .map(|(i, e)| {
                // Phase 47: Inputs persistence for learning
                e.intel.last_inputs = perception_buffer[i];

                let (outputs, next_hidden) = decision_buffer[i];
                e.intel.last_hidden = next_hidden;
                let mut local_deposits = Vec::new();
                let speed_cap = e.physics.max_speed;
                let sensing_radius = e.physics.sensing_range;
                let stomach_penalty = (e.metabolism.max_energy - 200.0).max(0.0) / 1000.0;
                let speed_mult = (1.0 + (outputs[2] as f64 + 1.0) / 2.0) * speed_cap;
                let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
                e.intel.last_aggression = (outputs[3] + 1.0) / 2.0;
                e.intel.last_share_intent = (outputs[4] + 1.0) / 2.0;
                e.intel.last_signal = outputs[5];
                e.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0; // P50: Mean of signal channels as volume

                let inertia = (0.8 + stomach_penalty).clamp(0.4, 0.95);
                e.physics.vx = e.physics.vx * inertia + (outputs[0] as f64) * (1.0 - inertia);
                e.physics.vy = e.physics.vy * inertia + (outputs[1] as f64) * (1.0 - inertia);

                let metabolism_mult = env.metabolism_multiplier();
                let mut move_cost =
                    self.config.metabolism.base_move_cost * metabolism_mult * speed_mult;
                if predation_mode {
                    move_cost *= 2.0;
                }
                let signal_cost = outputs[5].abs() as f64 * 0.1;
                let hidden_node_count = e
                    .intel
                    .genotype
                    .brain
                    .nodes
                    .iter()
                    .filter(|n| matches!(n.node_type, crate::model::brain::NodeType::Hidden))
                    .count();
                let enabled_conn_count = e
                    .intel
                    .genotype
                    .brain
                    .connections
                    .iter()
                    .filter(|c| c.enabled)
                    .count();
                let brain_maintenance =
                    (hidden_node_count as f64 * 0.02) + (enabled_conn_count as f64 * 0.005);
                let sensing_cost_mod = 1.0 + (sensing_radius - 5.0).max(0.0) * 0.1;
                let idle_cost = (self.config.metabolism.base_idle_cost + brain_maintenance)
                    * metabolism_mult
                    * sensing_cost_mod;
                e.metabolism.energy -= move_cost + idle_cost + signal_cost;

                if outputs[6] > 0.5 {
                    local_deposits.push((e.physics.x, e.physics.y, PheromoneType::SignalA, 0.5));
                }
                if outputs[7] > 0.5 {
                    local_deposits.push((e.physics.x, e.physics.y, PheromoneType::SignalB, 0.5));
                }

                action::handle_movement(e, speed_mult, &self.terrain, self.width, self.height);
                biological::biological_system(e);
                local_deposits
            })
            .collect();

        for row in pheromone_proposals {
            for (px, py, pt, pa) in row {
                self.pheromones.deposit(px, py, pt, pa);
            }
        }

        // Pass 2: Interaction Proposals
        let interaction_commands: Vec<InteractionCommand> = current_entities
            .par_iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if killed_ids.contains(&e.id) {
                    return None;
                }
                let mut local_cmds = Vec::new();
                let mut local_rng = rand::thread_rng();

                let kin_vx = perception_buffer[i][6] as f64;
                let kin_vy = perception_buffer[i][7] as f64;
                if (kin_vx != 0.0 || kin_vy != 0.0)
                    && (e.physics.vx * kin_vx + e.physics.vy * kin_vy) > 0.5
                {
                    local_cmds.push(InteractionCommand::TransferEnergy {
                        target_idx: i,
                        amount: 0.05,
                    });
                }

                // P44: Metabolic Feedback (Excretion)
                if e.metabolism.energy > e.metabolism.max_energy * 0.7 && local_rng.gen_bool(0.1) {
                    local_cmds.push(InteractionCommand::Fertilize {
                        x: e.physics.x,
                        y: e.physics.y,
                        amount: 0.01,
                    });
                }

                // DYNAMIC SOCIAL GRID (Territoriality)
                // If Alpha (Rank > 0.9), they "claim" territory
                if e.intel.rank > 0.9 && local_rng.gen_bool(0.2) {
                    local_cmds.push(InteractionCommand::TribalTerritory {
                        x: e.physics.x,
                        y: e.physics.y,
                        is_war: e.intel.last_aggression > 0.6,
                    });
                }

                // Tribal Split Check
                let crowding = perception_buffer[i][3];
                if let Some(new_color) = social::start_tribal_split(e, crowding) {
                    local_cmds.push(InteractionCommand::TribalSplit {
                        target_idx: i,
                        new_color,
                    });
                }

                let outputs = decision_buffer[i].0;

                // Phase 51: SYMBIOSIS - Attempt to Bond
                if e.intel.bonded_to.is_none() {
                    if let Some(p_id) =
                        social::handle_symbiosis(i, &current_entities, outputs, &self.spatial_hash)
                    {
                        local_cmds.push(InteractionCommand::Bond {
                            target_idx: i,
                            partner_id: p_id,
                        });
                    }
                }

                // Phase 51: Mutualistic Energy Transfer
                if let Some(p_id) = e.intel.bonded_to {
                    if let Some(p_idx) = entity_snapshots.iter().position(|s| s.id == p_id) {
                        let p_snap = &entity_snapshots[p_idx];
                        if p_snap.energy < 50.0 && e.metabolism.energy > 100.0 {
                            let amount = e.metabolism.energy * 0.1;
                            local_cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: p_idx,
                                amount,
                            });
                            local_cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: i,
                                amount: -amount,
                            });
                        }
                    }
                }

                let predation_mode = (outputs[3] as f64 + 1.0) / 2.0 > 0.5;
                if predation_mode {
                    let territorial_bonus = social::get_territorial_aggression(e);
                    let targets = self.spatial_hash.query(e.physics.x, e.physics.y, 1.5);
                    for t_idx in targets {
                        let v_snap = &entity_snapshots[t_idx];
                        let r = e
                            .intel
                            .genotype
                            .relatedness(&current_entities[t_idx].intel.genotype);

                        let sx = (e.physics.x as usize).min(self.width as usize - 1);
                        let sy = (e.physics.y as usize).min(self.height as usize - 1);
                        let social_mode = self.social_grid[sy][sx];

                        // Social Grid Effects
                        let is_peace_zone = social_mode == 1;
                        let is_war_zone = social_mode == 2;

                        // Punishment: Can attack same tribe if their reputation is low (< 0.5) or in War Zone
                        let can_predate = (!social::are_same_tribe(e, &current_entities[t_idx])
                            || current_entities[t_idx].intel.reputation < 0.5
                            || is_war_zone)
                            && !is_peace_zone;

                        if v_snap.id != e.id && can_predate {
                            let allies = self.spatial_hash.query(v_snap.x, v_snap.y, 3.0);
                            let sum_relatedness: f32 = allies
                                .iter()
                                .filter(|&&a_idx| entity_snapshots[a_idx].id != v_snap.id)
                                .map(|&a_idx| {
                                    current_entities[a_idx]
                                        .intel
                                        .genotype
                                        .relatedness(&current_entities[t_idx].intel.genotype)
                                })
                                .sum();

                            let mut defense_mult = (1.0 - (sum_relatedness as f64 * 0.15)).max(0.4);
                            if is_war_zone {
                                defense_mult *= 1.5; // Harder to defend in war zone? No, easier to kill.
                                                     // Actually, let's just boost attacker power.
                            }

                            let mut attacker_power = e.metabolism.energy * territorial_bonus;

                            // Soldier Bonus
                            if e.status(
                                self.config.metabolism.reproduction_threshold,
                                self.tick,
                                self.config.metabolism.maturity_age,
                            ) == crate::model::state::entity::EntityStatus::Soldier
                            {
                                attacker_power *= 1.5;
                            }

                            if is_war_zone {
                                attacker_power *= 2.0;
                            }

                            if attacker_power > (v_snap.energy / defense_mult) {
                                let gain = v_snap.energy
                                    * (e.metabolism.trophic_potential as f64)
                                    * (1.0 - (self.pop_stats.biomass_c / 10000.0)).max(0.5);

                                // Betrayal Penalty
                                if r > 0.5 {
                                    local_cmds.push(InteractionCommand::UpdateReputation {
                                        target_idx: i,
                                        delta: -0.3,
                                    });
                                }

                                local_cmds.push(InteractionCommand::Kill {
                                    target_idx: t_idx,
                                    attacker_lineage: e.metabolism.lineage_id,
                                    energy_gain: gain,
                                    cause: "predation".to_string(),
                                });
                                local_cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: gain - 10.0,
                                });
                                break;
                            }
                        }
                    }
                }

                if social::can_share(e) && e.intel.last_share_intent >= 0.5 {
                    let targets = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                    for t_idx in targets {
                        let t_snap = &entity_snapshots[t_idx];
                        let r = e
                            .intel
                            .genotype
                            .relatedness(&current_entities[t_idx].intel.genotype);
                        if t_snap.id != e.id && r > 0.25 && t_snap.energy < e.metabolism.energy {
                            let amount = e.metabolism.energy * 0.05 * r as f64;
                            local_cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: t_idx,
                                amount,
                            });
                            local_cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: i,
                                amount: -amount,
                            });
                            // Reputation Reward
                            local_cmds.push(InteractionCommand::UpdateReputation {
                                target_idx: i,
                                delta: 0.1 * r,
                            });
                        }
                    }
                }

                if e.is_mature(self.tick, self.config.metabolism.maturity_age)
                    && e.metabolism.energy > self.config.metabolism.reproduction_threshold
                {
                    let mate_indices = self.spatial_hash.query(e.physics.x, e.physics.y, 3.0);
                    let mut mate_idx = None;
                    for m_idx in mate_indices {
                        if m_idx != i
                            && entity_snapshots[m_idx].energy > 100.0
                            && (1.0
                                - (current_entities[m_idx].metabolism.trophic_potential
                                    - e.intel.genotype.mate_preference)
                                    .abs())
                                > 0.8
                        {
                            mate_idx = Some(m_idx);
                            break;
                        }
                    }
                    if let Some(m_idx) = mate_idx {
                        let mut child_genotype = intel::crossover_genotypes(
                            &e.intel.genotype,
                            &current_entities[m_idx].intel.genotype,
                        );
                        intel::mutate_genotype(
                            &mut child_genotype,
                            &self.config.evolution,
                            current_entities.len(),
                        );

                        let dist = e.intel.genotype.distance(&child_genotype);
                        // SPECIATION: Check if child has drifted too far from parent
                        if dist > self.config.evolution.speciation_threshold {
                            child_genotype.lineage_id = uuid::Uuid::new_v4();
                        }

                        local_cmds.push(InteractionCommand::Birth {
                            parent_idx: i,
                            baby: Box::new(social::reproduce_with_mate_parallel(
                                e,
                                self.tick,
                                child_genotype,
                            )),
                            genetic_distance: dist,
                        });
                        local_cmds.push(InteractionCommand::TransferEnergy {
                            target_idx: i,
                            amount: -50.0,
                        });
                    } else {
                        let (baby, dist) = social::reproduce_asexual_parallel(
                            e,
                            self.tick,
                            &self.config.evolution,
                            current_entities.len(),
                        );
                        local_cmds.push(InteractionCommand::Birth {
                            parent_idx: i,
                            baby: Box::new(baby),
                            genetic_distance: dist,
                        });
                    }
                }

                let (dx_f, dy_f, _) =
                    ecological::sense_nearest_food(e, &self.food, &self.food_hash);
                if (dx_f * dx_f + dy_f * dy_f) < 4.0 {
                    let candidates = self.food_hash.query(e.physics.x, e.physics.y, 2.0);
                    if let Some(&f_idx) = candidates.first() {
                        let niche_eff = 1.0
                            - (e.intel.genotype.metabolic_niche - self.food[f_idx].nutrient_type)
                                .abs();
                        let gain = self.config.metabolism.food_value
                            * niche_eff as f64
                            * (1.0 - e.metabolism.trophic_potential) as f64;
                        if gain > 0.1 {
                            local_cmds.push(InteractionCommand::EatFood {
                                food_index: f_idx,
                                attacker_idx: i,
                                energy_gain: gain,
                            });
                        }
                    }
                }

                if let Some(ref path) = e.health.pathogen {
                    let targets = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                    for t_idx in targets {
                        if entity_snapshots[t_idx].id != e.id
                            && local_rng.gen::<f32>() < path.transmission
                        {
                            local_cmds.push(InteractionCommand::Infect {
                                target_idx: t_idx,
                                pathogen: path.clone(),
                            });
                        }
                    }
                } else {
                    for path in &self.active_pathogens {
                        if local_rng.gen::<f32>() < path.transmission * 0.01 {
                            local_cmds.push(InteractionCommand::Infect {
                                target_idx: i,
                                pathogen: path.clone(),
                            });
                        }
                    }
                }

                if local_cmds.is_empty() {
                    None
                } else {
                    Some(local_cmds)
                }
            })
            .flatten()
            .collect();

        // Pass 3: Apply Commands
        for cmd in interaction_commands {
            match cmd {
                InteractionCommand::Kill {
                    target_idx,
                    attacker_lineage,
                    energy_gain,
                    cause,
                } => {
                    let target_id = current_entities[target_idx].id;
                    if !killed_ids.contains(&target_id) {
                        killed_ids.insert(target_id);
                        self.pop_stats.record_death(
                            self.tick - current_entities[target_idx].metabolism.birth_tick,
                        );
                        let ev = LiveEvent::Death {
                            id: target_id,
                            age: self.tick - current_entities[target_idx].metabolism.birth_tick,
                            offspring: current_entities[target_idx].metabolism.offspring_count,
                            tick: self.tick,
                            timestamp: Utc::now().to_rfc3339(),
                            cause,
                        };
                        let _ = self.logger.log_event(ev.clone());
                        events.push(ev);
                        self.lineage_consumption
                            .push((attacker_lineage, energy_gain));
                        self.pheromones.deposit(
                            current_entities[target_idx].physics.x,
                            current_entities[target_idx].physics.y,
                            PheromoneType::Danger,
                            0.5,
                        );
                    }
                }
                InteractionCommand::TransferEnergy { target_idx, amount } => {
                    let target_id = current_entities[target_idx].id;
                    // Skip dead entities (energy <= 0) and already killed entities
                    if !killed_ids.contains(&target_id)
                        && current_entities[target_idx].metabolism.energy > 0.0
                    {
                        current_entities[target_idx].metabolism.energy =
                            (current_entities[target_idx].metabolism.energy + amount)
                                .clamp(0.0, current_entities[target_idx].metabolism.max_energy);
                    }
                }
                InteractionCommand::Birth {
                    parent_idx: _,
                    baby,
                    genetic_distance,
                } => {
                    self.pop_stats.record_birth_distance(genetic_distance);
                    self.lineage_registry.record_birth(
                        baby.metabolism.lineage_id,
                        baby.metabolism.generation,
                        self.tick,
                    );
                    let ev = LiveEvent::Birth {
                        id: baby.id,
                        parent_id: baby.parent_id,
                        gen: baby.metabolism.generation,
                        tick: self.tick,
                        timestamp: Utc::now().to_rfc3339(),
                    };
                    let _ = self.logger.log_event(ev.clone());
                    events.push(ev);
                    new_babies.push(*baby);
                }
                InteractionCommand::EatFood {
                    food_index,
                    attacker_idx,
                    energy_gain,
                } => {
                    if !eaten_food_indices.contains(&food_index) {
                        let attacker_id = current_entities[attacker_idx].id;
                        if !killed_ids.contains(&attacker_id)
                            && current_entities[attacker_idx].metabolism.energy > 0.0
                        {
                            eaten_food_indices.insert(food_index);
                            current_entities[attacker_idx].metabolism.energy =
                                (current_entities[attacker_idx].metabolism.energy + energy_gain)
                                    .min(current_entities[attacker_idx].metabolism.max_energy);
                            self.lineage_consumption.push((
                                current_entities[attacker_idx].metabolism.lineage_id,
                                energy_gain,
                            ));
                            self.terrain.deplete(
                                current_entities[attacker_idx].physics.x,
                                current_entities[attacker_idx].physics.y,
                                0.01,
                            );
                        }
                    }
                }
                InteractionCommand::Infect {
                    target_idx,
                    pathogen,
                } => {
                    let target_id = current_entities[target_idx].id;
                    if !killed_ids.contains(&target_id)
                        && current_entities[target_idx].health.pathogen.is_none()
                        && rand::thread_rng().gen::<f32>()
                            > current_entities[target_idx].health.immunity
                    {
                        current_entities[target_idx].health.pathogen = Some(pathogen);
                        current_entities[target_idx].health.infection_timer = 0;
                    }
                }
                InteractionCommand::Fertilize { x, y, amount } => {
                    self.terrain.fertilize(x, y, amount);
                }
                InteractionCommand::UpdateReputation { target_idx, delta } => {
                    current_entities[target_idx].intel.reputation =
                        (current_entities[target_idx].intel.reputation + delta).clamp(0.0, 1.0);
                }
                InteractionCommand::TribalSplit {
                    target_idx,
                    new_color,
                } => {
                    current_entities[target_idx].physics.r = new_color.0;
                    current_entities[target_idx].physics.g = new_color.1;
                    current_entities[target_idx].physics.b = new_color.2;
                    current_entities[target_idx].intel.rank = 0.5;
                }
                InteractionCommand::TribalTerritory { x, y, is_war } => {
                    let ix = (x as usize).min(self.width as usize - 1);
                    let iy = (y as usize).min(self.height as usize - 1);
                    // 1: Peace, 2: War
                    self.social_grid[iy][ix] = if is_war { 2 } else { 1 };
                }
                InteractionCommand::Bond {
                    target_idx,
                    partner_id,
                } => {
                    current_entities[target_idx].intel.bonded_to = Some(partner_id);
                }
            }
        }
        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();

        for e in current_entities {
            if killed_ids.contains(&e.id) {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
                if let Some(legend) = social::archive_if_legend(&e, self.tick, &self.logger) {
                    self.update_best_legend(legend);
                }

                // P44: Corpse Fertilization
                let fert_amount = (e.metabolism.max_energy as f32 / 100.0) * 0.02;
                self.terrain
                    .fertilize(e.physics.x, e.physics.y, fert_amount);

                continue;
            }

            if e.metabolism.energy <= 0.0 {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
                if let Some(legend) = social::archive_if_legend(&e, self.tick, &self.logger) {
                    self.update_best_legend(legend);
                }

                // P44: Corpse Fertilization
                let fert_amount = (e.metabolism.max_energy as f32 / 100.0) * 0.02;
                self.terrain
                    .fertilize(e.physics.x, e.physics.y, fert_amount);

                // Do not add to alive_entities
            } else {
                alive_entities.push(e);
            }
        }
        self.entities.append(&mut alive_entities);
        self.entities.append(&mut new_babies);

        if !eaten_food_indices.is_empty() {
            let mut i = 0;
            self.food.retain(|_| {
                let keep = !eaten_food_indices.contains(&i);
                i += 1;
                keep
            });
        }

        self.killed_ids = killed_ids;
        self.eaten_food_indices = eaten_food_indices;
        self.new_babies = new_babies;
        self.alive_entities = alive_entities;
        self.perception_buffer = perception_buffer;
        self.decision_buffer = decision_buffer;
        self.energy_transfers = energy_transfers;

        social::handle_extinction(&self.entities, self.tick, &mut events, &mut self.logger);
        let pop_count = self.entities.len();
        let mut mutation_scale = self.config.evolution.mutation_rate;

        // Apply Era-based Mutation Scaling
        use crate::model::state::environment::Era;
        let era_mutation_mult = match env.current_era {
            Era::Primordial => 1.5,
            Era::DawnOfLife => 1.0,
            Era::Flourishing => 1.2,
            Era::DominanceWar => 0.8,
            Era::ApexEra => 0.5,
        };
        mutation_scale *= era_mutation_mult;

        if self.config.evolution.population_aware && pop_count > 0 {
            if pop_count < self.config.evolution.bottleneck_threshold {
                mutation_scale *=
                    (self.config.evolution.bottleneck_threshold as f32 / pop_count as f32).min(3.0);
            } else if pop_count > self.config.evolution.stasis_threshold {
                mutation_scale *= 0.5;
            }
        }

        stats::update_stats(
            self.tick,
            &self.entities,
            self.food.len(),
            env.carbon_level,
            mutation_scale,
            &mut self.pop_stats,
            &mut self.hall_of_fame,
            &self.terrain,
        );

        if self.tick.is_multiple_of(100) {
            let h_count = self
                .entities
                .iter()
                .filter(|e| e.metabolism.trophic_potential < 0.4)
                .count();
            let c_count = self.entities.len() - h_count;
            if h_count < 5 && c_count > 10 {
                let alert = LiveEvent::EcoAlert {
                    message: "Trophic Collapse: Prey scarcity!".to_string(),
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = self.logger.log_event(alert.clone());
                events.push(alert);
            }
            if self.pop_stats.biomass_h > 8000.0 {
                let alert = LiveEvent::EcoAlert {
                    message: "Overgrazing: Soil stress high!".to_string(),
                    tick: self.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = self.logger.log_event(alert.clone());
                events.push(alert);
            }
        }
        if self.tick.is_multiple_of(1000) {
            let _ = self
                .lineage_registry
                .save(format!("{}/lineages.json", self.log_dir));
            let _ = self
                .fossil_registry
                .save(&format!("{}/fossils.json", self.log_dir));
            let snap_ev = LiveEvent::Snapshot {
                tick: self.tick,
                stats: self.pop_stats.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(snap_ev.clone());
            events.push(snap_ev);
            self.handle_fossilization();
            self.lineage_registry.prune(); // P44: Registry Pruning
        }

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::config::AppConfig;

    #[test]
    fn test_handle_movement_wall_bounce() {
        let mut config = AppConfig::default();
        config.world.width = 10;
        config.world.height = 10;
        let mut world = World::new(0, config).unwrap();
        world
            .terrain
            .set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Wall);
        let mut entity = Entity::new(4.5, 4.5, 0);
        entity.physics.vx = 1.0;
        entity.physics.vy = 1.0;
        action::handle_movement(&mut entity, 1.0, &world.terrain, world.width, world.height);
        assert!(entity.physics.vx < 0.0);
        assert!(entity.physics.vy < 0.0);
        assert_eq!(entity.physics.x, 4.5);
    }
}
