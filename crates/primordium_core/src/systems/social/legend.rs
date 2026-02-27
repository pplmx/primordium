use chrono::Utc;
use primordium_data::Legend;
use primordium_data::{Identity, Intel, Metabolism, Physics};

pub fn archive_if_legend_components(
    identity: &Identity,
    metabolism: &Metabolism,
    intel: &Intel,
    physics: &Physics,
    tick: u64,
) -> Option<Legend> {
    let lifespan = tick - metabolism.birth_tick;
    if lifespan > 1000 || metabolism.offspring_count > 10 || metabolism.peak_energy > 300.0 {
        let legend = Legend {
            id: identity.id,
            parent_id: identity.parent_id,
            lineage_id: metabolism.lineage_id,
            birth_tick: metabolism.birth_tick,
            death_tick: tick,
            lifespan,
            generation: metabolism.generation,
            offspring_count: metabolism.offspring_count,
            peak_energy: metabolism.peak_energy,
            birth_timestamp: "".to_string(),
            death_timestamp: Utc::now().to_rfc3339(),
            genotype: (*intel.genotype).clone(),
            color_rgb: (physics.r, physics.g, physics.b),
        };
        Some(legend)
    } else {
        None
    }
}

pub fn is_legend_worthy_components(metabolism: &Metabolism, tick: u64) -> bool {
    let lifespan = tick - metabolism.birth_tick;
    lifespan > 1000 || metabolism.offspring_count > 10 || metabolism.peak_energy > 300.0
}
