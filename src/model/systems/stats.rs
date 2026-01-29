use crate::model::history::{HallOfFame, PopulationStats};
use primordium_data::Entity;

/// Update population statistics and Hall of Fame.
#[allow(clippy::too_many_arguments)]
pub fn update_stats(
    tick: u64,
    entities: &[Entity],
    food_count: usize,
    carbon_level: f64,
    mutation_scale: f32,
    pop_stats: &mut PopulationStats,
    hall_of_fame: &mut HallOfFame,
    terrain: &crate::model::terrain::TerrainGrid,
) {
    if tick.is_multiple_of(60) {
        crate::model::history::update_hall_of_fame(hall_of_fame, entities, tick);
        let top_fitness = hall_of_fame
            .top_living
            .first()
            .map(|(s, _)| *s)
            .unwrap_or(0.0);
        crate::model::history::update_population_stats(
            pop_stats,
            entities,
            food_count,
            top_fitness,
            carbon_level,
            mutation_scale,
            terrain,
        );
    }
}
