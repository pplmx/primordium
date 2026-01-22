use crate::model::environment::Environment;
use crate::model::terrain::TerrainGrid;
use rand::Rng;

/// Handle global environmental disasters.
pub fn handle_disasters(
    env: &Environment,
    entity_count: usize,
    terrain: &mut TerrainGrid,
    rng: &mut impl Rng,
) {
    // Trigger Dust Bowl disaster
    if env.is_heat_wave() && entity_count > 300 && rng.gen_bool(0.01) {
        terrain.trigger_dust_bowl(500);
    }
}
