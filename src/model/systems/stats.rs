use crate::model::history::{HallOfFame, PopulationStats};
use crate::model::snapshot::EntitySnapshot;

/// Update population statistics and Hall of Fame using snapshots.
#[allow(clippy::too_many_arguments)]
pub fn update_stats(
    tick: u64,
    entities: &[EntitySnapshot],
    food_count: usize,
    carbon_level: f64,
    mutation_scale: f32,
    pop_stats: &mut PopulationStats,
    _hall_of_fame: &mut HallOfFame,
    terrain: &crate::model::terrain::TerrainGrid,
) {
    if tick.is_multiple_of(60) {
        crate::model::history::update_population_stats_snapshots(
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
