use crate::model::entity::Entity;
use crate::model::history::{HallOfFame, PopulationStats};

/// Update population statistics and Hall of Fame.
pub fn update_stats(
    tick: u64,
    entities: &[Entity],
    pop_stats: &mut PopulationStats,
    hall_of_fame: &mut HallOfFame,
) {
    if tick % 60 == 0 {
        hall_of_fame.update(entities, tick);
        let top_fitness = hall_of_fame
            .top_living
            .first()
            .map(|(s, _)| *s)
            .unwrap_or(0.0);
        pop_stats.update_snapshot(entities, top_fitness);
    }
}
