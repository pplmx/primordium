use crate::brain::BrainLogic;
use primordium_data::{Entity, HallOfFame, PopulationStats};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct StatsContext<'a, T> {
    pub stats: &'a mut PopulationStats,
    pub entities: &'a [T],
    pub food_count: usize,
    pub top_fitness: f64,
    pub carbon_level: f64,
    pub mutation_scale: f32,
    pub terrain: &'a crate::terrain::TerrainGrid,
    pub tick: u64,
}

pub fn update_population_stats(ctx: StatsContext<Entity>) {
    ctx.stats.population = ctx.entities.len();
    ctx.stats.food_count = ctx.food_count;
    ctx.stats.top_fitness = ctx.top_fitness;
    ctx.stats.carbon_level = ctx.carbon_level;
    ctx.stats.mutation_scale = ctx.mutation_scale;
    ctx.stats.global_fertility = ctx.terrain.average_fertility();
    ctx.stats.max_generation = ctx
        .entities
        .iter()
        .map(|e| e.metabolism.generation)
        .max()
        .unwrap_or(0);
    ctx.stats.lineage_counts.clear();
    ctx.stats.biomass_h = 0.0;
    ctx.stats.biomass_c = 0.0;
    ctx.stats.biodiversity_hotspots = 0;

    if ctx.entities.is_empty() {
        ctx.stats.avg_brain_entropy = 0.0;
        ctx.stats.species_count = 0;
        return;
    }

    let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
    for e in ctx.entities {
        *ctx.stats
            .lineage_counts
            .entry(e.metabolism.lineage_id)
            .or_insert(0) += 1;
        let tp = e.metabolism.trophic_potential;
        if tp < 0.4 {
            ctx.stats.biomass_h += e.metabolism.energy;
        } else if tp > 0.6 {
            ctx.stats.biomass_c += e.metabolism.energy;
        }
        let sx = (e.physics.x / 10.0) as i32;
        let sy = (e.physics.y / 10.0) as i32;
        sectors
            .entry((sx, sy))
            .or_default()
            .insert(e.metabolism.lineage_id);
    }
    ctx.stats.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();

    let mut complexity_freq = HashMap::new();
    for e in ctx.entities {
        let conn_count = e
            .intel
            .genotype
            .brain
            .connections
            .iter()
            .filter(|c| c.enabled)
            .count();
        let bucket = (conn_count / 10) * 10;
        *complexity_freq.entry(bucket).or_insert(0.0) += 1.0;
    }
    let total_samples = complexity_freq.values().sum::<f64>();
    let mut entropy = 0.0;
    for &count in complexity_freq.values() {
        let p = count / total_samples;
        if p > 0.0 {
            entropy -= p * p.log2();
        }
    }
    ctx.stats.avg_brain_entropy = entropy;

    let mut representatives: Vec<&primordium_data::Brain> = Vec::new();
    let threshold = 2.0;
    for e in ctx.entities {
        let mut found = false;
        for rep in &representatives {
            if e.intel.genotype.brain.genotype_distance(rep) < threshold {
                found = true;
                break;
            }
        }
        if !found {
            representatives.push(&e.intel.genotype.brain);
        }
    }
    ctx.stats.species_count = representatives.len();

    // Phase 67 Task C: Calculate average fitness for DDA
    let mut total_fitness = 0.0;
    let mut max_fitness = 0.0;
    for e in ctx.entities {
        let age = ctx.tick - e.metabolism.birth_tick;
        // Fitness formula: age * 0.5 + offspring * 10 + current_energy * 0.2
        let fitness = (age as f64 * 0.5)
            + (e.metabolism.offspring_count as f64 * 10.0)
            + (e.metabolism.energy * 0.2);
        total_fitness += fitness;
        if fitness > max_fitness {
            max_fitness = fitness;
        }
    }
    ctx.stats.avg_fitness = total_fitness / ctx.entities.len() as f64;
    ctx.stats.top_fitness = max_fitness;
}

pub fn record_stat_death(stats: &mut PopulationStats, lifespan: u64) {
    stats.recent_deaths.push_back(lifespan as f64);
    if stats.recent_deaths.len() > 100 {
        stats.recent_deaths.pop_front();
    }
    if !stats.recent_deaths.is_empty() {
        stats.avg_lifespan =
            stats.recent_deaths.iter().sum::<f64>() / stats.recent_deaths.len() as f64;
    }
}

pub fn record_stat_birth_distance(stats: &mut PopulationStats, distance: f32) {
    stats.recent_distances.push_back(distance);
    if stats.recent_distances.len() > 100 {
        stats.recent_distances.pop_front();
    }
    if !stats.recent_distances.is_empty() {
        stats.evolutionary_velocity =
            stats.recent_distances.iter().sum::<f32>() / stats.recent_distances.len() as f32;
    }
}

pub fn update_hall_of_fame(hof: &mut HallOfFame, entities: &[Entity], tick: u64) {
    let mut scores: Vec<(f64, Entity)> = entities
        .iter()
        .map(|e| {
            let age = tick - e.metabolism.birth_tick;
            let score = (age as f64 * 0.5)
                + (e.metabolism.offspring_count as f64 * 10.0)
                + (e.metabolism.peak_energy * 0.2);
            (score, e.clone())
        })
        .collect();
    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    hof.top_living = scores.into_iter().take(3).collect();
}

/// Read-only world state for statistics computation.
pub struct StatsInput<'a> {
    pub tick: u64,
    pub entities: &'a [crate::snapshot::InternalEntitySnapshot],
    pub food_count: usize,
    pub carbon_level: f64,
    pub mutation_scale: f32,
    pub terrain: &'a crate::terrain::TerrainGrid,
}

/// Update population statistics and Hall of Fame using snapshots.
pub fn update_stats(
    input: &StatsInput<'_>,
    pop_stats: &mut PopulationStats,
    _hall_of_fame: &mut HallOfFame,
) {
    if input.tick.is_multiple_of(60) {
        update_population_stats_snapshots(StatsContext {
            stats: pop_stats,
            entities: input.entities,
            food_count: input.food_count,
            top_fitness: 0.0,
            carbon_level: input.carbon_level,
            mutation_scale: input.mutation_scale,
            terrain: input.terrain,
            tick: input.tick,
        });
    }
}

pub fn update_population_stats_snapshots(
    ctx: StatsContext<crate::snapshot::InternalEntitySnapshot>,
) {
    ctx.stats.population = ctx.entities.len();
    ctx.stats.food_count = ctx.food_count;
    ctx.stats.top_fitness = ctx.top_fitness;
    ctx.stats.carbon_level = ctx.carbon_level;
    ctx.stats.mutation_scale = ctx.mutation_scale;
    ctx.stats.global_fertility = ctx.terrain.average_fertility();
    ctx.stats.max_generation = ctx.entities.iter().map(|e| e.generation).max().unwrap_or(0);
    ctx.stats.lineage_counts.clear();
    ctx.stats.biomass_h = 0.0;
    ctx.stats.biomass_c = 0.0;
    ctx.stats.biodiversity_hotspots = 0;

    if ctx.entities.is_empty() {
        ctx.stats.avg_brain_entropy = 0.0;
        ctx.stats.species_count = 0;
        ctx.stats.avg_fitness = 0.0;
        return;
    }

    let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
    for e in ctx.entities {
        *ctx.stats.lineage_counts.entry(e.lineage_id).or_insert(0) += 1;
        let tp = e.trophic_potential;
        if tp < 0.4 {
            ctx.stats.biomass_h += e.energy;
        } else if tp > 0.6 {
            ctx.stats.biomass_c += e.energy;
        }
        let sx = (e.x / 10.0) as i32;
        let sy = (e.y / 10.0) as i32;
        sectors.entry((sx, sy)).or_default().insert(e.lineage_id);
    }
    ctx.stats.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();

    // Phase 67 Task C: Calculate average fitness for DDA
    let mut total_fitness = 0.0;
    let mut max_fitness = 0.0;
    for e in ctx.entities {
        let age = ctx.tick - e.birth_tick;
        // Fitness formula: age * 0.5 + offspring * 10 + current_energy * 0.2
        let fitness = (age as f64 * 0.5) + (e.offspring_count as f64 * 10.0) + (e.energy * 0.2);
        total_fitness += fitness;
        if fitness > max_fitness {
            max_fitness = fitness;
        }
    }
    ctx.stats.avg_fitness = total_fitness / ctx.entities.len() as f64;
    ctx.stats.top_fitness = max_fitness;
}
