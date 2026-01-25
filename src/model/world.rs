use crate::model::config::AppConfig;
use crate::model::history::{
    FossilRegistry, HallOfFame, HistoryLogger, LiveEvent, PopulationStats,
};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::lineage_registry::LineageRegistry;
use crate::model::state::pheromone::PheromoneGrid;
use crate::model::state::terrain::TerrainGrid;
use crate::model::systems::{action, biological, ecological, environment, intel, social, stats};
use chrono::Utc;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug)]
pub enum InteractionCommand {
    Kill {
        target_idx: usize,
        attacker_idx: usize,
        attacker_lineage: uuid::Uuid,
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
    BondBreak {
        target_idx: usize,
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
    pub social_grid: Vec<Vec<u8>>,
    pub lineage_registry: LineageRegistry,
    pub fossil_registry: FossilRegistry,
    pub log_dir: String,
    pub active_pathogens: Vec<crate::model::state::pathogen::Pathogen>,
    #[serde(skip, default)]
    pub best_legends: HashMap<uuid::Uuid, crate::model::history::Legend>,
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
            food.push(Food::new(
                rng.gen_range(1..config.world.width - 1),
                rng.gen_range(1..config.world.height - 1),
                rng.gen_range(0.0..1.0),
            ));
        }
        let terrain = TerrainGrid::generate(config.world.width, config.world.height, 42);
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
            best_legends: HashMap::new(),
            killed_ids: HashSet::new(),
            eaten_food_indices: HashSet::new(),
            new_babies: Vec::new(),
            alive_entities: Vec::new(),
            perception_buffer: Vec::new(),
            decision_buffer: Vec::new(),
            lineage_consumption: Vec::new(),
        })
    }

    pub fn load_persistent(&mut self) -> anyhow::Result<()> {
        self.lineage_registry =
            LineageRegistry::load(format!("{}/lineages.json", self.log_dir)).unwrap_or_default();
        self.fossil_registry =
            FossilRegistry::load(&format!("{}/fossils.json.gz", self.log_dir)).unwrap_or_default();
        Ok(())
    }

    pub(crate) fn handle_fossilization(&mut self) {
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
                        self.fossil_registry
                            .add_fossil(crate::model::history::Fossil {
                                lineage_id: l_id,
                                name: record.name.clone(),
                                color_rgb: legend.color_rgb,
                                avg_lifespan: legend.lifespan as f64,
                                max_generation: record.max_generation,
                                total_offspring: record.total_entities_produced as u32,
                                extinct_tick: self.tick,
                                peak_population: record.peak_population,
                                genotype: legend.genotype.clone(),
                            });
                    }
                }
            }
        }
    }

    fn update_best_legend(&mut self, legend: crate::model::history::Legend) {
        let entry = self
            .best_legends
            .entry(legend.lineage_id)
            .or_insert_with(|| legend.clone());
        if (legend.lifespan as f64 * 0.5 + legend.offspring_count as f64 * 10.0)
            > (entry.lifespan as f64 * 0.5 + entry.offspring_count as f64 * 10.0)
        {
            *entry = legend;
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
        if self.tick.is_multiple_of(50) {
            for row in &mut self.social_grid {
                for cell in row {
                    if *cell > 0 && rng.gen_bool(0.1) {
                        *cell = 0;
                    }
                }
            }
        }
        self.pheromones.decay();
        environment::handle_disasters(
            env,
            self.entities.len(),
            &mut self.terrain,
            &mut rng,
            &self.config,
        );
        let total_plant_biomass = self.terrain.update(self.pop_stats.biomass_h);
        env.sequestrate_carbon(total_plant_biomass * 0.00001);
        env.add_carbon(self.entities.len() as f64 * 0.01);
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
            self.food_hash.insert(f.x as f64, f.y as f64, i);
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
                status: e.status(0.0, self.tick, self.config.metabolism.maturity_age),
            })
            .collect();

        let mut killed_ids = std::mem::take(&mut self.killed_ids);
        killed_ids.clear();
        let mut eaten_food_indices = std::mem::take(&mut self.eaten_food_indices);
        eaten_food_indices.clear();
        let mut new_babies = std::mem::take(&mut self.new_babies);
        new_babies.clear();
        let mut alive_entities = std::mem::take(&mut self.alive_entities);
        alive_entities.clear();
        let mut perception_buffer = std::mem::take(&mut self.perception_buffer);
        let mut decision_buffer = std::mem::take(&mut self.decision_buffer);

        current_entities.par_iter_mut().for_each(|e| {
            e.intel.genotype.brain.learn(
                e.intel.last_inputs,
                e.intel.last_hidden,
                ((e.metabolism.energy - e.metabolism.prev_energy)
                    / e.metabolism.max_energy.max(1.0)) as f32
                    * 10.0,
            );
            e.metabolism.prev_energy = e.metabolism.energy;
            e.intel.rank = social::calculate_social_rank(e, self.tick);
            if let Some(p_id) = e.intel.bonded_to {
                if let Some(partner) = entity_snapshots.iter().find(|s| s.id == p_id) {
                    let dx = partner.x - e.physics.x;
                    let dy = partner.y - e.physics.y;
                    if (dx * dx + dy * dy) > 400.0 {
                        // Break bond if distance > 20.0
                        e.intel.bonded_to = None;
                    }
                } else {
                    e.intel.bonded_to = None;
                }
            }
        });

        current_entities
            .par_iter()
            .enumerate()
            .map(|(_i, e)| {
                let (dx_f, dy_f, f_type) =
                    ecological::sense_nearest_food(e, &self.food, &self.food_hash);
                let nearby =
                    self.spatial_hash
                        .query(e.physics.x, e.physics.y, e.physics.sensing_range);
                let (ph_f, _, ph_a, ph_b) = self.pheromones.sense_all(
                    e.physics.x,
                    e.physics.y,
                    e.physics.sensing_range / 2.0,
                );
                let mut partner_energy = 0.0;
                if let Some(p_id) = e.intel.bonded_to {
                    if let Some(p_idx) = entity_snapshots.iter().position(|s| s.id == p_id) {
                        partner_energy = (entity_snapshots[p_idx].energy
                            / e.metabolism.max_energy.max(1.0))
                            as f32;
                    }
                }
                [
                    (dx_f / 20.0) as f32,
                    (dy_f / 20.0) as f32,
                    (e.metabolism.energy / e.metabolism.max_energy.max(1.0)) as f32,
                    (nearby.len() as f32 / 10.0).min(1.0),
                    ph_f,
                    0.0,
                    0.0,
                    0.0,
                    ph_a,
                    ph_b,
                    0.0,
                    0.0,
                    f_type,
                    e.metabolism.trophic_potential,
                    e.intel.last_vocalization,
                    partner_energy,
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

        let interaction_commands: Vec<InteractionCommand> = current_entities
            .par_iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let mut cmds = Vec::new();
                let mut rng = rand::thread_rng();
                let (outputs, _) = decision_buffer[i];
                if e.intel.bonded_to.is_none() {
                    if let Some(p_id) =
                        social::handle_symbiosis(i, &current_entities, outputs, &self.spatial_hash)
                    {
                        cmds.push(InteractionCommand::Bond {
                            target_idx: i,
                            partner_id: p_id,
                        });
                    }
                }
                if let Some(p_id) = e.intel.bonded_to {
                    // Voluntary bond breaking: if Bond output < 0.2
                    if outputs[8] < 0.2 {
                        cmds.push(InteractionCommand::BondBreak { target_idx: i });
                    } else if let Some(p_idx) = entity_snapshots.iter().position(|s| s.id == p_id) {
                        // Phase 51: Metabolic Fusion (Bidirectional Equalization)
                        let self_energy = e.metabolism.energy;
                        let partner_energy = entity_snapshots[p_idx].energy;

                        // If I have significantly more energy, share it to equalize
                        if self_energy > partner_energy + 2.0 {
                            let diff = self_energy - partner_energy;
                            // Transfer 5% of the difference per tick
                            let amount = diff * 0.05;
                            if amount > 0.1 {
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: p_idx,
                                    amount,
                                });
                                cmds.push(InteractionCommand::TransferEnergy {
                                    target_idx: i,
                                    amount: -amount,
                                });
                            }
                        }
                    }
                } else if social::can_share(e)
                    && (outputs[4] > 0.5 || e.intel.last_share_intent >= 0.5)
                {
                    let nearby = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                    for &t_idx in &nearby {
                        if t_idx != i && social::are_same_tribe(e, &current_entities[t_idx]) {
                            let t_snap = &entity_snapshots[t_idx];
                            if t_snap.energy < e.metabolism.energy {
                                let r = e
                                    .intel
                                    .genotype
                                    .relatedness(&current_entities[t_idx].intel.genotype);
                                let amount = e.metabolism.energy * 0.05 * r as f64;
                                if amount > 1.0 {
                                    cmds.push(InteractionCommand::TransferEnergy {
                                        target_idx: t_idx,
                                        amount,
                                    });
                                    cmds.push(InteractionCommand::TransferEnergy {
                                        target_idx: i,
                                        amount: -amount,
                                    });
                                }
                            }
                        }
                    }
                }
                if e.intel.rank > 0.9 && rng.gen_bool(0.1) {
                    cmds.push(InteractionCommand::TribalTerritory {
                        x: e.physics.x,
                        y: e.physics.y,
                        is_war: outputs[3] > 0.5,
                    });
                }
                if outputs[3] > 0.5 {
                    let targets = self.spatial_hash.query(e.physics.x, e.physics.y, 1.5);
                    for t_idx in targets {
                        if t_idx != i && !social::are_same_tribe(e, &current_entities[t_idx]) {
                            cmds.push(InteractionCommand::Kill {
                                target_idx: t_idx,
                                attacker_idx: i,
                                attacker_lineage: e.metabolism.lineage_id,
                                cause: "predation".to_string(),
                            });
                            break;
                        }
                    }
                }
                if e.is_mature(self.tick, self.config.metabolism.maturity_age)
                    && e.metabolism.energy > self.config.metabolism.reproduction_threshold
                {
                    let (baby, dist) = social::reproduce_asexual_parallel(
                        e,
                        self.tick,
                        &self.config.evolution,
                        current_entities.len(),
                    );
                    cmds.push(InteractionCommand::Birth {
                        parent_idx: i,
                        baby: Box::new(baby),
                        genetic_distance: dist,
                    });
                }
                let food_near = self.food_hash.query(e.physics.x, e.physics.y, 2.0);
                if let Some(&f_idx) = food_near.first() {
                    let niche_eff = 1.0
                        - (e.intel.genotype.metabolic_niche - self.food[f_idx].nutrient_type).abs();
                    let gain = self.config.metabolism.food_value
                        * niche_eff as f64
                        * (1.0 - e.metabolism.trophic_potential) as f64;
                    if gain > 0.1 {
                        cmds.push(InteractionCommand::EatFood {
                            food_index: f_idx,
                            attacker_idx: i,
                        });
                    }
                }
                if let Some(ref path) = e.health.pathogen {
                    let targets = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                    for t_idx in targets {
                        if entity_snapshots[t_idx].id != e.id
                            && rng.gen::<f32>() < path.transmission
                        {
                            cmds.push(InteractionCommand::Infect {
                                target_idx: t_idx,
                                pathogen: path.clone(),
                            });
                        }
                    }
                }
                if cmds.is_empty() {
                    None
                } else {
                    Some(cmds)
                }
            })
            .flatten()
            .collect();

        let current_population = current_entities.len();
        for (i, e) in current_entities.iter_mut().enumerate() {
            let (outputs, next_hidden) = decision_buffer[i];
            e.intel.last_hidden = next_hidden;
            e.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;
            action::action_system(
                e,
                outputs,
                &mut action::ActionContext {
                    env,
                    config: &self.config,
                    terrain: &self.terrain,
                    pheromones: &mut self.pheromones,
                    snapshots: &entity_snapshots,
                    width: self.width,
                    height: self.height,
                },
            );
            biological::biological_system(e, current_population);
        }

        for cmd in interaction_commands {
            match cmd {
                InteractionCommand::Kill {
                    target_idx,
                    attacker_idx,
                    attacker_lineage,
                    cause,
                } => {
                    let tid = current_entities[target_idx].id;
                    if !killed_ids.contains(&tid) {
                        killed_ids.insert(tid);
                        let energy_gain = current_entities[target_idx].metabolism.energy * 0.5;
                        self.pop_stats.record_death(
                            self.tick - current_entities[target_idx].metabolism.birth_tick,
                        );
                        let ev = LiveEvent::Death {
                            id: tid,
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
                        current_entities[attacker_idx].metabolism.energy =
                            (current_entities[attacker_idx].metabolism.energy + energy_gain)
                                .min(current_entities[attacker_idx].metabolism.max_energy);
                    }
                }
                InteractionCommand::Bond {
                    target_idx,
                    partner_id,
                } => {
                    current_entities[target_idx].intel.bonded_to = Some(partner_id);
                }
                InteractionCommand::BondBreak { target_idx } => {
                    current_entities[target_idx].intel.bonded_to = None;
                }
                InteractionCommand::Birth {
                    parent_idx,
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
                    let inv = current_entities[parent_idx]
                        .intel
                        .genotype
                        .reproductive_investment as f64;
                    let c_e = current_entities[parent_idx].metabolism.energy * inv;
                    current_entities[parent_idx].metabolism.energy -= c_e;
                    current_entities[parent_idx].metabolism.offspring_count += 1;
                }
                InteractionCommand::EatFood {
                    food_index,
                    attacker_idx,
                } => {
                    if !eaten_food_indices.contains(&food_index) {
                        eaten_food_indices.insert(food_index);
                        let e = &current_entities[attacker_idx];
                        let niche_eff = 1.0
                            - (e.intel.genotype.metabolic_niche
                                - self.food[food_index].nutrient_type)
                                .abs();
                        let energy_gain = self.config.metabolism.food_value
                            * niche_eff as f64
                            * (1.0 - e.metabolism.trophic_potential) as f64;

                        current_entities[attacker_idx].metabolism.energy =
                            (current_entities[attacker_idx].metabolism.energy + energy_gain)
                                .min(current_entities[attacker_idx].metabolism.max_energy);
                        self.terrain.deplete(
                            current_entities[attacker_idx].physics.x,
                            current_entities[attacker_idx].physics.y,
                            0.01,
                        );
                    }
                }
                InteractionCommand::TribalTerritory { x, y, is_war } => {
                    let ix = (x as usize).min(self.width as usize - 1);
                    let iy = (y as usize).min(self.height as usize - 1);
                    self.social_grid[iy][ix] = if is_war { 2 } else { 1 };
                }
                InteractionCommand::TransferEnergy { target_idx, amount } => {
                    current_entities[target_idx].metabolism.energy =
                        (current_entities[target_idx].metabolism.energy + amount)
                            .clamp(0.0, current_entities[target_idx].metabolism.max_energy);
                }
                InteractionCommand::Infect {
                    target_idx,
                    pathogen,
                } => {
                    current_entities[target_idx].health.pathogen = Some(pathogen.clone());
                    current_entities[target_idx].health.infection_timer = pathogen.duration;
                }
                InteractionCommand::UpdateReputation { target_idx, delta } => {
                    current_entities[target_idx].intel.reputation =
                        (current_entities[target_idx].intel.reputation + delta).clamp(0.0, 1.0);
                }
                _ => {}
            }
        }

        for (l_id, amount) in &self.lineage_consumption {
            self.lineage_registry.record_consumption(*l_id, *amount);
        }
        self.lineage_consumption.clear();
        for e in current_entities {
            if killed_ids.contains(&e.id) || e.metabolism.energy <= 0.0 {
                self.lineage_registry.record_death(e.metabolism.lineage_id);
                if let Some(legend) = social::archive_if_legend(&e, self.tick, &self.logger) {
                    self.update_best_legend(legend);
                }
            } else {
                alive_entities.push(e);
            }
        }
        self.entities.append(&mut alive_entities);
        self.entities.append(&mut new_babies);
        let mut i = 0;
        self.food.retain(|_| {
            let k = !eaten_food_indices.contains(&i);
            i += 1;
            k
        });
        self.perception_buffer = perception_buffer;
        self.decision_buffer = decision_buffer;
        self.killed_ids = killed_ids;
        self.eaten_food_indices = eaten_food_indices;
        if self.tick.is_multiple_of(1000) {
            let _ = self
                .lineage_registry
                .save(format!("{}/lineages.json", self.log_dir));
            let _ = self
                .fossil_registry
                .save(&format!("{}/fossils.json.gz", self.log_dir));
            let snap_ev = LiveEvent::Snapshot {
                tick: self.tick,
                stats: self.pop_stats.clone(),
                timestamp: Utc::now().to_rfc3339(),
            };
            let _ = self.logger.log_event(snap_ev.clone());
            events.push(snap_ev);
            self.handle_fossilization();
            self.lineage_registry.prune();
        }
        stats::update_stats(
            self.tick,
            &self.entities,
            self.food.len(),
            env.carbon_level,
            self.config.evolution.mutation_rate,
            &mut self.pop_stats,
            &mut self.hall_of_fame,
            &self.terrain,
        );
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    }
}
