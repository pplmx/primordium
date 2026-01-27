use crate::model::config::AppConfig;
use crate::model::history::{
    FossilRegistry, HallOfFame, HistoryLogger, LiveEvent, PopulationStats,
};
use crate::model::quadtree::SpatialHash;
use crate::model::state::entity::Entity;
use crate::model::state::environment::Environment;
use crate::model::state::food::Food;
use crate::model::state::interaction::InteractionCommand;
use crate::model::state::lineage_registry::LineageRegistry;
use crate::model::state::pheromone::PheromoneGrid;
use crate::model::state::sound::SoundGrid;
use crate::model::state::terrain::TerrainGrid;
use crate::model::systems::{
    action, biological, ecological, environment, intel, interaction, social, stats,
};
use chrono::Utc;
use rand::Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize)]
pub struct InternalEntitySnapshot {
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

#[derive(Default)]
pub struct EntityDecision {
    pub outputs: [f32; 11],
    pub next_hidden: [f32; 6],
    pub activations: std::collections::HashMap<usize, f32>,
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
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub spatial_hash: SpatialHash,
    #[serde(skip, default = "SpatialHash::new_empty")]
    pub food_hash: SpatialHash,
    pub pop_stats: PopulationStats,
    pub hall_of_fame: HallOfFame,
    pub terrain: TerrainGrid,
    pub pheromones: PheromoneGrid,
    pub sound: SoundGrid,
    pub pressure: crate::model::state::pressure::PressureGrid,
    pub social_grid: Vec<u8>,
    pub lineage_registry: LineageRegistry,
    pub fossil_registry: FossilRegistry,
    pub config: AppConfig,
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
    decision_buffer: Vec<EntityDecision>,
    #[serde(skip, default)]
    lineage_consumption: Vec<(uuid::Uuid, f64)>,
    #[serde(skip)]
    cached_terrain: std::sync::Arc<TerrainGrid>,
    #[serde(skip)]
    cached_pheromones: std::sync::Arc<PheromoneGrid>,
    #[serde(skip)]
    cached_sound: std::sync::Arc<SoundGrid>,
    #[serde(skip)]
    cached_pressure: std::sync::Arc<crate::model::state::pressure::PressureGrid>,
    #[serde(skip)]
    cached_social_grid: std::sync::Arc<Vec<u8>>,
    #[serde(skip, default)]
    food_dirty: bool,
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
        let sound = SoundGrid::new(config.world.width, config.world.height);
        let pressure = crate::model::state::pressure::PressureGrid::new(
            config.world.width,
            config.world.height,
        );
        let social_grid = vec![0; config.world.width as usize * config.world.height as usize];

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
            cached_terrain: std::sync::Arc::new(terrain.clone()),
            cached_pheromones: std::sync::Arc::new(pheromones.clone()),
            cached_sound: std::sync::Arc::new(sound.clone()),
            cached_pressure: std::sync::Arc::new(pressure.clone()),
            cached_social_grid: std::sync::Arc::new(social_grid.clone()),
            terrain,
            pheromones,
            sound,
            pressure,
            social_grid,
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
            food_dirty: true,
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

    pub fn apply_genetic_edit(
        &mut self,
        entity_id: uuid::Uuid,
        gene: crate::app::state::GeneType,
        delta: f32,
    ) {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity_id) {
            use crate::app::state::GeneType;
            match gene {
                GeneType::Trophic => {
                    e.intel.genotype.trophic_potential =
                        (e.intel.genotype.trophic_potential + delta).clamp(0.0, 1.0);
                    e.metabolism.trophic_potential = e.intel.genotype.trophic_potential;
                }
                GeneType::Sensing => {
                    e.intel.genotype.sensing_range =
                        (e.intel.genotype.sensing_range + delta as f64).clamp(3.0, 30.0);
                    e.physics.sensing_range = e.intel.genotype.sensing_range;
                }
                GeneType::Speed => {
                    e.intel.genotype.max_speed =
                        (e.intel.genotype.max_speed + delta as f64).clamp(0.1, 5.0);
                    e.physics.max_speed = e.intel.genotype.max_speed;
                }
                GeneType::ReproInvest => {
                    e.intel.genotype.reproductive_investment =
                        (e.intel.genotype.reproductive_investment + delta).clamp(0.1, 0.9);
                }
                GeneType::Maturity => {
                    e.intel.genotype.maturity_gene =
                        (e.intel.genotype.maturity_gene + delta).clamp(0.1, 5.0);
                }
                GeneType::MaxEnergy => {
                    e.intel.genotype.max_energy =
                        (e.intel.genotype.max_energy + delta as f64).clamp(50.0, 2000.0);
                    e.metabolism.max_energy = e.intel.genotype.max_energy;
                }
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
                // Distribute energy to all entities or specific pool
                // For now, let's distribute to the top 10% fittest living entities
                let count = (self.entities.len() / 10).max(1);
                let amount_per = (amount * sign) / count as f32;
                for e in self.entities.iter_mut().take(count) {
                    e.metabolism.energy = (e.metabolism.energy + amount_per as f64)
                        .clamp(0.0, e.metabolism.max_energy);
                }
            }
            TradeResource::Oxygen => {
                env.oxygen_level = (env.oxygen_level + (amount * sign) as f64).clamp(0.0, 50.0);
            }
            TradeResource::SoilFertility => {
                self.terrain.add_global_fertility(amount * sign);
            }
            TradeResource::Biomass => {
                // Add/Remove food
                if incoming {
                    let mut rng = rand::thread_rng();
                    for _ in 0..(amount as usize) {
                        let fx = rng.gen_range(1..self.width - 1);
                        let fy = rng.gen_range(1..self.height - 1);
                        self.food.push(Food::new(fx, fy, 0.0));
                    }
                } else {
                    self.food
                        .truncate(self.food.len().saturating_sub(amount as usize));
                }
                self.food_dirty = true;
            }
        }
    }

    pub fn clear_research_deltas(&mut self, entity_id: uuid::Uuid) {
        if let Some(e) = self.entities.iter_mut().find(|e| e.id == entity_id) {
            e.intel.genotype.brain.weight_deltas.clear();
        }
    }

    pub fn create_snapshot(
        &self,
        selected_id: Option<uuid::Uuid>,
    ) -> crate::model::state::snapshot::WorldSnapshot {
        use crate::model::state::snapshot::{EntitySnapshot, WorldSnapshot};
        let entities = self
            .entities
            .iter()
            .map(|e| EntitySnapshot {
                id: e.id,
                name: e.name(),
                x: e.physics.x,
                y: e.physics.y,
                r: e.physics.r,
                g: e.physics.g,
                b: e.physics.b,
                energy: e.metabolism.energy,
                max_energy: e.metabolism.max_energy,
                generation: e.metabolism.generation,
                age: self.tick - e.metabolism.birth_tick,
                offspring: e.metabolism.offspring_count,
                lineage_id: e.metabolism.lineage_id,
                rank: e.intel.rank,
                status: e.status(
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
                specialization: e.intel.specialization,
                last_vocalization: e.intel.last_vocalization,
                bonded_to: e.intel.bonded_to,
                trophic_potential: e.metabolism.trophic_potential,
                last_activations: e.intel.last_activations.clone(),
                last_inputs: e.intel.last_inputs,
                last_hidden: e.intel.last_hidden,
                genotype_hex: if Some(e.id) == selected_id {
                    Some(e.intel.genotype.to_hex())
                } else {
                    None
                },
            })
            .collect();

        WorldSnapshot {
            tick: self.tick,
            entities,
            food: self.food.clone(),
            stats: self.pop_stats.clone(),
            hall_of_fame: self.hall_of_fame.clone(),
            terrain: self.cached_terrain.clone(),
            pheromones: self.cached_pheromones.clone(),
            sound: self.cached_sound.clone(),
            pressure: self.cached_pressure.clone(),
            social_grid: self.cached_social_grid.clone(),
            width: self.width,
            height: self.height,
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
            for val in &mut self.social_grid {
                *val = 0;
            }
        }
        self.pheromones.update();
        self.sound.update();
        self.pressure.update();
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
        env.consume_oxygen(
            self.entities.len() as f64 * self.config.metabolism.oxygen_consumption_rate,
        );
        env.tick();
        biological::handle_pathogen_emergence(&mut self.active_pathogens, &mut rng);
        let old_food_len = self.food.len();
        ecological::spawn_food(
            &mut self.food,
            env,
            &self.terrain,
            &self.config,
            self.width,
            self.height,
            &mut rng,
        );
        if self.food.len() != old_food_len {
            self.food_dirty = true;
        }
        if self.food_dirty {
            self.food_hash.clear();
            for (i, f) in self.food.iter().enumerate() {
                self.food_hash.insert(f.x as f64, f.y as f64, i);
            }
            self.food_dirty = false;
        }
        let mut current_entities = std::mem::take(&mut self.entities);
        let spatial_data: Vec<(f64, f64, uuid::Uuid)> = current_entities
            .iter()
            .map(|e| (e.physics.x, e.physics.y, e.metabolism.lineage_id))
            .collect();
        self.spatial_hash.build_with_lineage(&spatial_data);
        let entity_snapshots: Vec<InternalEntitySnapshot> = current_entities
            .par_iter()
            .map(|e| InternalEntitySnapshot {
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
                    self.config.brain.activation_threshold,
                    self.tick,
                    self.config.metabolism.maturity_age,
                ),
            })
            .collect();

        let entity_id_map: HashMap<uuid::Uuid, usize> = current_entities
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id, i))
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
                    * self.config.brain.learning_reinforcement,
            );
            e.metabolism.prev_energy = e.metabolism.energy;
            e.intel.rank = social::calculate_social_rank(e, self.tick, &self.config);
            if let Some(p_id) = e.intel.bonded_to {
                if let Some(partner) = entity_id_map.get(&p_id).map(|&idx| &entity_snapshots[idx]) {
                    let dx = partner.x - e.physics.x;
                    let dy = partner.y - e.physics.y;
                    if (dx * dx + dy * dy) > self.config.social.bond_break_dist.powi(2) {
                        // Break bond if distance > dist
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
                let (ph_f, _, _, _) = self.pheromones.sense_all(
                    e.physics.x,
                    e.physics.y,
                    e.physics.sensing_range / 2.0,
                );
                let sound_sense =
                    self.sound
                        .sense(e.physics.x, e.physics.y, e.physics.sensing_range);
                let mut partner_energy = 0.0;
                if let Some(p_id) = e.intel.bonded_to {
                    if let Some(&p_idx) = entity_id_map.get(&p_id) {
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
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    f_type,
                    e.metabolism.trophic_potential,
                    sound_sense,
                    partner_energy,
                ]
            })
            .collect_into_vec(&mut perception_buffer);

        current_entities
            .par_iter()
            .zip(perception_buffer.par_iter())
            .map(|(e, inputs)| {
                let (mut outputs, next_hidden, activations) = e
                    .intel
                    .genotype
                    .brain
                    .forward_internal(*inputs, e.intel.last_hidden);

                // Phase 55: Parasitic Manipulation
                if let Some(ref path) = e.health.pathogen {
                    if let Some((idx, offset)) = path.behavior_manipulation {
                        let out_idx = idx.saturating_sub(22);
                        if out_idx < 11 {
                            outputs[out_idx] = (outputs[out_idx] + offset).clamp(-1.0, 1.0);
                        }
                    }
                }

                EntityDecision {
                    outputs,
                    next_hidden,
                    activations,
                }
            })
            .collect_into_vec(&mut decision_buffer);

        let interaction_commands: Vec<InteractionCommand> = current_entities
            .par_iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let mut cmds = Vec::new();
                let mut rng = rand::thread_rng();
                let decision = &decision_buffer[i];
                let outputs = decision.outputs;
                if e.intel.bonded_to.is_none() && e.metabolism.has_metamorphosed {
                    if let Some(p_id) = social::handle_symbiosis(
                        i,
                        &current_entities,
                        outputs,
                        &self.spatial_hash,
                        &self.config,
                    ) {
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
                    } else if let Some(&p_idx) = entity_id_map.get(&p_id) {
                        // Phase 54: Sexual Reproduction (Bonded Partners)
                        // If both are mature and have high energy, reproduce sexually.
                        let partner = &current_entities[p_idx];
                        if e.is_mature(self.tick, self.config.metabolism.maturity_age)
                            && partner.is_mature(self.tick, self.config.metabolism.maturity_age)
                            && e.metabolism.energy > self.config.metabolism.reproduction_threshold
                            && partner.metabolism.energy
                                > self.config.metabolism.reproduction_threshold
                        {
                            let mut child_genotype = intel::crossover_genotypes(
                                &e.intel.genotype,
                                &partner.intel.genotype,
                            );
                            intel::mutate_genotype(
                                &mut child_genotype,
                                &self.config,
                                current_entities.len(),
                            );
                            let dist = e.intel.genotype.distance(&child_genotype);
                            if dist > self.config.evolution.speciation_threshold {
                                child_genotype.lineage_id = uuid::Uuid::new_v4();
                            }
                            let baby =
                                social::reproduce_with_mate_parallel(e, self.tick, child_genotype);
                            cmds.push(InteractionCommand::Birth {
                                parent_idx: i,
                                baby: Box::new(baby),
                                genetic_distance: dist,
                            });
                            // Also need to deduct energy from partner, but we handle it in InteractionCommand
                            // Actually, let's just make the partner lose energy too.
                            cmds.push(InteractionCommand::TransferEnergy {
                                target_idx: p_idx,
                                amount: -(partner.metabolism.energy
                                    * partner.intel.genotype.reproductive_investment as f64),
                            });
                        }

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
                } else if social::can_share(e, &self.config)
                    && (outputs[4] > self.config.brain.activation_threshold
                        || e.intel.last_share_intent >= self.config.brain.activation_threshold)
                {
                    let nearby = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                    for &t_idx in &nearby {
                        if t_idx != i
                            && social::are_same_tribe(e, &current_entities[t_idx], &self.config)
                        {
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
                        let is_partner = e.intel.bonded_to == Some(entity_snapshots[t_idx].id);
                        if t_idx != i
                            && !social::are_same_tribe(e, &current_entities[t_idx], &self.config)
                            && !is_partner
                        {
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
                // Phase 49: Tribal Splitting
                let nearby = self.spatial_hash.query(e.physics.x, e.physics.y, 2.0);
                let crowding =
                    (nearby.len() as f32 / self.config.evolution.crowding_normalization).min(1.0);
                if let Some(new_color) = social::start_tribal_split(e, crowding, &self.config) {
                    cmds.push(InteractionCommand::TribalSplit {
                        target_idx: i,
                        new_color,
                    });
                }
                if e.is_mature(self.tick, self.config.metabolism.maturity_age)
                    && e.metabolism.energy > self.config.metabolism.reproduction_threshold
                {
                    let (baby, dist) = social::reproduce_asexual_parallel(
                        e,
                        self.tick,
                        &self.config,
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
                // Phase 52: Terraforming
                if e.metabolism.has_metamorphosed && outputs[9] > 0.5 {
                    cmds.push(InteractionCommand::Dig {
                        x: e.physics.x,
                        y: e.physics.y,
                        attacker_idx: i,
                    });
                }
                if e.metabolism.has_metamorphosed && outputs[10] > 0.5 {
                    cmds.push(InteractionCommand::Build {
                        x: e.physics.x,
                        y: e.physics.y,
                        attacker_idx: i,
                        is_nest: outputs[10] > 0.9,
                    });
                }
                // Phase 58: Metamorphosis trigger (80% maturity to allow Juvenile stage)
                let actual_maturity = (self.config.metabolism.maturity_age as f32
                    * e.intel.genotype.maturity_gene) as u64;
                let age = self.tick - e.metabolism.birth_tick;
                if !e.metabolism.has_metamorphosed
                    && age
                        >= (actual_maturity as f32
                            * self.config.metabolism.metamorphosis_trigger_maturity)
                            as u64
                {
                    cmds.push(InteractionCommand::Metamorphosis { target_idx: i });
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
        let action_results: Vec<action::ActionResult> = current_entities
            .par_iter_mut()
            .zip(decision_buffer.par_iter_mut())
            .map(|(e, decision)| {
                let EntityDecision {
                    outputs,
                    next_hidden,
                    activations,
                } = std::mem::take(decision);
                e.intel.last_hidden = next_hidden;
                e.intel.last_activations = activations
                    .into_iter()
                    .map(|(k, v)| (k as i32, v))
                    .collect();
                e.intel.last_vocalization = (outputs[6] + outputs[7] + 2.0) / 4.0;
                let res = action::action_system(
                    e,
                    outputs,
                    &mut action::ActionContext {
                        env,
                        config: &self.config,
                        terrain: &self.terrain,
                        snapshots: &entity_snapshots,
                        entity_id_map: &entity_id_map,
                        spatial_hash: &self.spatial_hash,
                        width: self.width,
                        height: self.height,
                    },
                );
                biological::biological_system(e, current_population, &self.config);
                res
            })
            .collect();

        for res in action_results {
            for p in res.pheromones {
                self.pheromones.deposit(p.x, p.y, p.ptype, p.amount);
            }
            for s in res.sounds {
                self.sound.deposit(s.x, s.y, s.amount);
            }
            env.consume_oxygen(res.oxygen_drain);
        }

        let mut interaction_ctx = interaction::InteractionContext {
            terrain: &mut self.terrain,
            env,
            pop_stats: &mut self.pop_stats,
            lineage_registry: &mut self.lineage_registry,
            fossil_registry: &mut self.fossil_registry,
            logger: &mut self.logger,
            config: &self.config,
            tick: self.tick,
            width: self.width,
            height: self.height,
            social_grid: &mut self.social_grid,
            lineage_consumption: &mut self.lineage_consumption,
            food: &mut self.food,
        };

        let interaction_result = interaction::process_interaction_commands(
            &mut current_entities,
            interaction_commands,
            &mut interaction_ctx,
        );

        events.extend(interaction_result.events);
        let killed_ids = interaction_result.killed_ids;
        let eaten_food_indices = interaction_result.eaten_food_indices;
        let mut new_babies = interaction_result.new_babies;

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
                // Corpse Fertilization: transfer fraction of max energy to soil
                let fertilize_amount = (e.metabolism.max_energy
                    * self.config.ecosystem.corpse_fertility_mult as f64)
                    as f32
                    / 100.0;
                self.terrain
                    .fertilize(e.physics.x, e.physics.y, fertilize_amount);
                self.terrain
                    .add_biomass(e.physics.x, e.physics.y, fertilize_amount * 10.0);
            } else {
                alive_entities.push(e);
            }
        }
        self.entities.append(&mut alive_entities);
        self.entities.append(&mut new_babies);
        if !eaten_food_indices.is_empty() {
            let mut i = 0;
            self.food.retain(|_| {
                let k = !eaten_food_indices.contains(&i);
                i += 1;
                k
            });
            self.food_dirty = true;
        }
        self.perception_buffer = perception_buffer;
        self.decision_buffer = decision_buffer;
        self.killed_ids = killed_ids;
        self.eaten_food_indices = eaten_food_indices;
        if self.tick.is_multiple_of(1000) {
            self.lineage_registry.check_goals(
                self.tick,
                &self.social_grid,
                self.width,
                self.height,
            );
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
        if self.terrain.is_dirty {
            self.cached_terrain = std::sync::Arc::new(self.terrain.clone());
            self.terrain.is_dirty = false;
        }
        if self.pheromones.is_dirty {
            self.cached_pheromones = std::sync::Arc::new(self.pheromones.clone());
            self.pheromones.is_dirty = false;
        }
        if self.sound.is_dirty {
            self.cached_sound = std::sync::Arc::new(self.sound.clone());
            self.sound.is_dirty = false;
        }
        if self.pressure.is_dirty {
            self.cached_pressure = std::sync::Arc::new(self.pressure.clone());
            self.pressure.is_dirty = false;
        }
        self.cached_social_grid = std::sync::Arc::new(self.social_grid.clone());

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
