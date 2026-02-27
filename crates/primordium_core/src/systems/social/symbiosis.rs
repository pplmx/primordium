use crate::config::AppConfig;
use crate::history::{LiveEvent, PopulationStats};
use crate::pheromone::PheromoneGrid;
use crate::snapshot::InternalEntitySnapshot;
use crate::spatial_hash::SpatialHash;

use primordium_data::{AncestralTrait, Specialization};
use rand::Rng;
use std::collections::HashSet;
use uuid::Uuid;

pub struct PredationContext<'a> {
    pub snapshots: &'a [InternalEntitySnapshot],
    pub killed_ids: &'a mut HashSet<Uuid>,
    pub events: &'a mut Vec<LiveEvent>,
    pub config: &'a AppConfig,
    pub spatial_hash: &'a SpatialHash,
    pub pheromones: &'a mut PheromoneGrid,
    pub pop_stats: &'a mut PopulationStats,
    // logger removed
    pub tick: u64,
}

pub fn handle_symbiosis_components(
    idx: usize,
    snapshots: &[InternalEntitySnapshot],
    outputs: [f32; 12],
    spatial_hash: &SpatialHash,
    config: &AppConfig,
) -> Option<Uuid> {
    if outputs[8] > 0.5 {
        let mut partner_id = None;
        let self_snap = &snapshots[idx];

        spatial_hash.query_callback(
            self_snap.x,
            self_snap.y,
            config.social.territorial_range,
            |t_idx| {
                if idx != t_idx && partner_id.is_none() {
                    let target_snap = &snapshots[t_idx];
                    let color_dist = (self_snap.r as i32 - target_snap.r as i32).abs()
                        + (self_snap.g as i32 - target_snap.g as i32).abs()
                        + (self_snap.b as i32 - target_snap.b as i32).abs();

                    if color_dist < config.social.tribe_color_threshold
                        && target_snap.status != primordium_data::EntityStatus::Bonded
                    {
                        partner_id = Some(target_snap.id);
                    }
                }
            },
        );
        partner_id
    } else {
        None
    }
}

pub struct ReproductionContext<'a, R: Rng> {
    pub tick: u64,
    pub config: &'a crate::config::AppConfig,
    pub population: usize,
    pub traits: std::collections::HashSet<AncestralTrait>,
    pub is_radiation_storm: bool,
    pub rng: &'a mut R,
    pub ancestral_genotype: Option<&'a primordium_data::Genotype>,
}

pub struct AsexualReproductionContext<'a, R: Rng> {
    pub pos: &'a primordium_data::Position,
    pub energy: f64,
    pub generation: u32,
    pub genotype: &'a primordium_data::Genotype,
    pub specialization: Option<Specialization>,
    pub ctx: &'a mut ReproductionContext<'a, R>,
}
