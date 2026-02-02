use crate::history::{Fossil, FossilRegistry, Legend};
use crate::lineage_registry::LineageRegistry;
use std::collections::HashMap;
use uuid::Uuid;

pub fn handle_fossilization(
    lineage_registry: &LineageRegistry,
    fossil_registry: &mut FossilRegistry,
    best_legends: &mut HashMap<Uuid, Legend>,
    tick: u64,
) {
    let extinct = lineage_registry.get_extinct_lineages();
    for l_id in extinct {
        if let Some(legend) = best_legends.remove(&l_id) {
            if !fossil_registry.fossils.iter().any(|f| f.lineage_id == l_id) {
                if let Some(record) = lineage_registry.lineages.get(&l_id) {
                    fossil_registry.add_fossil(Fossil {
                        lineage_id: l_id,
                        name: record.name.clone(),
                        color_rgb: legend.color_rgb,
                        avg_lifespan: legend.lifespan as f64,
                        max_generation: record.max_generation,
                        total_offspring: record.total_entities_produced as u32,
                        extinct_tick: tick,
                        peak_population: record.peak_population,
                        genotype: legend.genotype.clone(),
                    });
                }
            }
        }
    }
}

pub fn update_best_legend(
    lineage_registry: &mut LineageRegistry,
    best_legends: &mut HashMap<Uuid, Legend>,
    legend: Legend,
) {
    let entry = best_legends
        .entry(legend.lineage_id)
        .or_insert_with(|| legend.clone());
    if (legend.lifespan as f64 * 0.5 + legend.offspring_count as f64 * 10.0)
        > (entry.lifespan as f64 * 0.5 + entry.offspring_count as f64 * 10.0)
    {
        *entry = legend.clone();
        // Phase 64: Genetic Memory - Update max fitness genotype
        if let Some(record) = lineage_registry.lineages.get_mut(&legend.lineage_id) {
            record.max_fitness_genotype = Some(legend.genotype);
        }
    }
}
