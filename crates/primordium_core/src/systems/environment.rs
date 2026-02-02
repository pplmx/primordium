use crate::config::AppConfig;
use crate::environment::Environment;
use crate::terrain::TerrainGrid;
use primordium_data::PopulationStats;
use rand::Rng;

/// Handle global environmental disasters.
pub fn handle_disasters(
    env: &Environment,
    entity_count: usize,
    terrain: &mut TerrainGrid,
    rng: &mut impl Rng,
    config: &AppConfig,
) {
    // Trigger Dust Bowl disaster
    if env.is_heat_wave() && entity_count > 300 && rng.gen_bool(config.world.disaster_chance as f64)
    {
        terrain.trigger_dust_bowl(500);
    }
}

/// Update environmental event timers based on system metrics.
pub fn update_events(env: &mut Environment, config: &AppConfig) {
    if env.cpu_usage > config.world.heat_wave_cpu {
        env.heat_wave_timer += 1;
    } else {
        env.heat_wave_timer = env.heat_wave_timer.saturating_sub(1);
    }

    if env.cpu_usage < config.world.ice_age_cpu {
        env.ice_age_timer += 1;
    } else {
        env.ice_age_timer = env.ice_age_timer.saturating_sub(1);
    }

    if env.ram_usage_percent < config.world.abundance_ram {
        env.abundance_timer = 30;
    } else {
        env.abundance_timer = env.abundance_timer.saturating_sub(1);
    }

    if env.radiation_timer > 0 {
        env.radiation_timer = env.radiation_timer.saturating_sub(1);
    }
}

/// Update simulation era and season cycle.
pub fn update_era(
    env: &mut Environment,
    tick: u64,
    pop_stats: &PopulationStats,
    config: &AppConfig,
) {
    // Season cycling
    env.season_tick += 1;
    if env.season_tick >= env.season_duration {
        env.season_tick = 0;
        env.current_season = env.current_season.next();
    }

    // Era Transition Logic
    use crate::environment::Era;
    if env.current_era == Era::Primordial {
        // Dawn of Life: Needs either stability or a critical mass of biomass
        if (tick > 5000 && pop_stats.avg_lifespan > 200.0) || pop_stats.biomass_h > 2000.0 {
            env.current_era = Era::DawnOfLife;
        }
    } else if env.current_era == Era::DawnOfLife {
        // Flourishing: Needs biodiversity hotspots and healthy population
        if pop_stats.biodiversity_hotspots >= 2 && pop_stats.population > 150 {
            env.current_era = Era::Flourishing;
        }
    } else if env.current_era == Era::Flourishing {
        // Dominance War: Triggered by resource scarcity or high carbon (climate stress)
        if env.carbon_level > 800.0 || (pop_stats.biomass_c / pop_stats.biomass_h.max(1.0)) > 0.3 {
            env.current_era = Era::DominanceWar;
        }
    }

    // Apex Era: Peak fitness reached
    if pop_stats.top_fitness > config.world.apex_fitness_req {
        env.current_era = Era::ApexEra;
    }
}
