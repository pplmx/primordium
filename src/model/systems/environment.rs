use crate::model::environment::Environment;
use crate::model::history::PopulationStats;
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

/// Update environmental event timers based on system metrics.
pub fn update_events(env: &mut Environment) {
    if env.cpu_usage > 80.0 {
        env.heat_wave_timer += 1;
    } else {
        env.heat_wave_timer = env.heat_wave_timer.saturating_sub(1);
    }

    if env.cpu_usage < 10.0 {
        env.ice_age_timer += 1;
    } else {
        env.ice_age_timer = env.ice_age_timer.saturating_sub(1);
    }

    if env.ram_usage_percent < 40.0 {
        env.abundance_timer = 30;
    } else {
        env.abundance_timer = env.abundance_timer.saturating_sub(1);
    }
}

/// Update simulation era and season cycle.
pub fn update_era(env: &mut Environment, tick: u64, pop_stats: &PopulationStats) {
    // Season cycling
    env.season_tick += 1;
    if env.season_tick >= env.season_duration {
        env.season_tick = 0;
        env.current_season = env.current_season.next();
    }

    // Era Transition Logic
    use crate::model::environment::Era;
    if env.current_era == Era::Primordial {
        if tick > 5000 && pop_stats.avg_lifespan > 200.0 {
            env.current_era = Era::DawnOfLife;
        }
    } else if env.current_era == Era::DawnOfLife {
        if pop_stats.population > 200 && pop_stats.species_count > 3 {
            env.current_era = Era::Flourishing;
        }
    } else if env.current_era == Era::Flourishing && env.cpu_usage > 70.0 {
        env.current_era = Era::DominanceWar;
    }

    if pop_stats.top_fitness > 5000.0 {
        env.current_era = Era::ApexEra;
    }
}
