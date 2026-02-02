use crate::brain::BrainLogic;
use primordium_data::{Entity, HallOfFame, PopulationStats};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn update_population_stats_snapshots(
    stats: &mut PopulationStats,
    entities: &[crate::snapshot::EntitySnapshot],
    food_count: usize,
    top_fitness: f64,
    carbon_level: f64,
    mutation_scale: f32,
    terrain: &crate::terrain::TerrainGrid,
) {
    stats.population = entities.len();
    stats.food_count = food_count;
    stats.top_fitness = top_fitness;
    stats.carbon_level = carbon_level;
    stats.mutation_scale = mutation_scale;
    stats.global_fertility = terrain.average_fertility();
    stats.max_generation = entities.iter().map(|e| e.generation).max().unwrap_or(0);
    stats.lineage_counts.clear();
    stats.biomass_h = 0.0;
    stats.biomass_c = 0.0;
    stats.biodiversity_hotspots = 0;

    if entities.is_empty() {
        stats.avg_brain_entropy = 0.0;
        stats.species_count = 0;
        return;
    }

    let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
    for e in entities {
        *stats.lineage_counts.entry(e.lineage_id).or_insert(0) += 1;
        let tp = e.trophic_potential;
        if tp < 0.4 {
            stats.biomass_h += e.energy;
        } else if tp > 0.6 {
            stats.biomass_c += e.energy;
        }
        let sx = (e.x / 10.0) as i32;
        let sy = (e.y / 10.0) as i32;
        sectors.entry((sx, sy)).or_default().insert(e.lineage_id);
    }
    stats.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();
}

pub fn update_population_stats(
    stats: &mut PopulationStats,
    entities: &[Entity],
    food_count: usize,
    top_fitness: f64,
    carbon_level: f64,
    mutation_scale: f32,
    terrain: &crate::terrain::TerrainGrid,
) {
    stats.population = entities.len();
    stats.food_count = food_count;
    stats.top_fitness = top_fitness;
    stats.carbon_level = carbon_level;
    stats.mutation_scale = mutation_scale;
    stats.global_fertility = terrain.average_fertility();
    stats.max_generation = entities
        .iter()
        .map(|e| e.metabolism.generation)
        .max()
        .unwrap_or(0);
    stats.lineage_counts.clear();
    stats.biomass_h = 0.0;
    stats.biomass_c = 0.0;
    stats.biodiversity_hotspots = 0;

    if entities.is_empty() {
        stats.avg_brain_entropy = 0.0;
        stats.species_count = 0;
        return;
    }

    let mut sectors: HashMap<(i32, i32), HashSet<Uuid>> = HashMap::new();
    for e in entities {
        *stats
            .lineage_counts
            .entry(e.metabolism.lineage_id)
            .or_insert(0) += 1;
        let tp = e.metabolism.trophic_potential;
        if tp < 0.4 {
            stats.biomass_h += e.metabolism.energy;
        } else if tp > 0.6 {
            stats.biomass_c += e.metabolism.energy;
        }
        let sx = (e.physics.x / 10.0) as i32;
        let sy = (e.physics.y / 10.0) as i32;
        sectors
            .entry((sx, sy))
            .or_default()
            .insert(e.metabolism.lineage_id);
    }
    stats.biodiversity_hotspots = sectors.values().filter(|s| s.len() >= 5).count();

    let mut complexity_freq = HashMap::new();
    for e in entities {
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
    stats.avg_brain_entropy = entropy;

    let mut representatives: Vec<&primordium_data::Brain> = Vec::new();
    let threshold = 2.0;
    for e in entities {
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
    stats.species_count = representatives.len();
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

/// Update population statistics and Hall of Fame using snapshots.
#[allow(clippy::too_many_arguments)]
pub fn update_stats(
    tick: u64,
    entities: &[crate::snapshot::EntitySnapshot],
    food_count: usize,
    carbon_level: f64,
    mutation_scale: f32,
    pop_stats: &mut PopulationStats,
    _hall_of_fame: &mut HallOfFame,
    terrain: &crate::terrain::TerrainGrid,
) {
    if tick.is_multiple_of(60) {
        update_population_stats_snapshots(
            pop_stats,
            entities,
            food_count,
            0.0,
            carbon_level,
            mutation_scale,
            terrain,
        );
    }
}
