use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Era {
    #[default]
    Primordial,
    DawnOfLife,
    Flourishing,
    DominanceWar,
    ApexEra,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClimateState {
    Temperate,
    Warm,
    Hot,
    Scorching,
}

impl ClimateState {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            ClimateState::Temperate => "🌡️ Temperate",
            ClimateState::Warm => "🔥 Warm",
            ClimateState::Hot => "🌋 Hot",
            ClimateState::Scorching => "☀️ Scorching",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResourceState {
    Abundant,
    Strained,
    Scarce,
    Famine,
}

impl ResourceState {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            ResourceState::Abundant => "🌾 Abundant",
            ResourceState::Strained => "⚠️ Strained",
            ResourceState::Scarce => "🚨 Scarce",
            ResourceState::Famine => "💀 Famine",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Season {
    #[default]
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Season::Spring => "🌸 Spring",
            Season::Summer => "☀️ Summer",
            Season::Fall => "🍂 Fall",
            Season::Winter => "❄️ Winter",
        }
    }

    #[must_use]
    pub fn food_multiplier(&self) -> f64 {
        match self {
            Season::Spring => 1.5,
            Season::Summer => 1.0,
            Season::Fall => 1.2,
            Season::Winter => 0.5,
        }
    }

    #[must_use]
    pub fn metabolism_multiplier(&self) -> f64 {
        match self {
            Season::Spring => 0.8,
            Season::Summer => 1.2,
            Season::Fall => 1.0,
            Season::Winter => 1.5,
        }
    }

    #[must_use]
    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    fn smooth_step(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    #[must_use]
    pub fn food_multiplier_smooth(&self, next: Season, progress: f64) -> f64 {
        let t = Self::smooth_step(progress.clamp(0.0, 1.0));
        let from = self.food_multiplier();
        let to = next.food_multiplier();
        from + (to - from) * t
    }

    #[must_use]
    pub fn metabolism_multiplier_smooth(&self, next: Season, progress: f64) -> f64 {
        let t = Self::smooth_step(progress.clamp(0.0, 1.0));
        let from = self.metabolism_multiplier();
        let to = next.metabolism_multiplier();
        from + (to - from) * t
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Day,
    Night,
}

impl TimeOfDay {
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            TimeOfDay::Day => "☀️ Day",
            TimeOfDay::Night => "🌙 Night",
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Environment {
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
    pub load_avg: f64,
    pub heat_wave_timer: u32,
    pub ice_age_timer: u32,
    pub abundance_timer: u32,
    pub radiation_timer: u32,
    pub current_era: Era,
    pub current_season: Season,
    pub next_season: Season,
    pub season_tick: u64,
    pub season_duration: u64,
    pub transition_duration: u64,
    pub world_time: u64,
    pub day_cycle_ticks: u64,
    pub god_climate_override: Option<ClimateState>,
    pub carbon_level: f64,
    pub oxygen_level: f64,
    /// Memory usage of the Primordium process in MB
    pub app_memory_usage_mb: f32,
    /// Global energy pool available for spawning food/life
    pub available_energy: f64,
    /// Phase 67 Task C: DDA solar multiplier (adjusts solar_energy_rate dynamically)
    pub dda_solar_multiplier: f64,
    /// Phase 67 Task C: DDA base idle multiplier (adjusts base_idle_cost dynamically)
    pub dda_base_idle_multiplier: f64,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            ram_usage_percent: 0.0,
            load_avg: 0.0,
            heat_wave_timer: 0,
            ice_age_timer: 0,
            abundance_timer: 0,
            radiation_timer: 0,
            current_era: Era::Primordial,
            current_season: Season::Spring,
            next_season: Season::Summer,
            season_tick: 0,
            season_duration: 10000,
            transition_duration: 1000,
            world_time: 0,
            day_cycle_ticks: 2000,
            god_climate_override: None,
            carbon_level: 300.0,
            oxygen_level: 21.0,
            app_memory_usage_mb: 0.0,
            available_energy: 10000.0,
            dda_solar_multiplier: 1.0,
            dda_base_idle_multiplier: 1.0,
        }
    }
}

impl Environment {
    pub fn tick(&mut self) {
        self.world_time = (self.world_time + 1) % self.day_cycle_ticks;
        self.carbon_level = (self.carbon_level * 0.9999 + 300.0 * 0.0001).clamp(0.0, 2000.0);
        self.oxygen_level = (self.oxygen_level * 0.9 + 21.0 * 0.1).clamp(5.0, 50.0);
    }

    pub fn tick_deterministic(&mut self, tick: u64) {
        self.heat_wave_timer = 0;
        self.ice_age_timer = 0;
        self.abundance_timer = 0;
        self.radiation_timer = 0;

        self.current_era = Era::Primordial;
        self.current_season = Season::Spring;
        self.world_time = 500;

        let t = tick as f32 * 0.01;
        self.cpu_usage = 50.0 + (t.sin() * 20.0);
        self.ram_usage_percent = 60.0 + (t.cos() * 10.0);
        self.carbon_level = 300.0;
        self.oxygen_level = 21.0;
        self.available_energy = 10000.0;
        self.dda_solar_multiplier = 1.0;
        self.dda_base_idle_multiplier = 1.0;
    }

    pub fn add_carbon(&mut self, amount: f64) {
        self.carbon_level = (self.carbon_level + amount).min(2000.0);
        self.oxygen_level = (self.oxygen_level - amount * 0.001).max(5.0);
    }

    pub fn sequestrate_carbon(&mut self, amount: f64) {
        self.carbon_level = (self.carbon_level - amount).max(0.0);
        self.oxygen_level = (self.oxygen_level + amount * 2.0).min(50.0);
    }

    pub fn consume_oxygen(&mut self, amount: f64) {
        self.oxygen_level = (self.oxygen_level - amount).max(5.0);
    }

    #[must_use]
    pub fn time_of_day(&self) -> TimeOfDay {
        if self.world_time < self.day_cycle_ticks / 2 {
            TimeOfDay::Day
        } else {
            TimeOfDay::Night
        }
    }

    #[must_use]
    pub fn light_level(&self) -> f32 {
        let half_cycle = self.day_cycle_ticks as f32 / 2.0;
        let progress = (self.world_time as f32 % self.day_cycle_ticks as f32) / half_cycle;

        if progress < 1.0 {
            let x = progress - 0.5;
            1.0 - (x * x * 4.0)
        } else {
            0.1
        }
    }

    #[must_use]
    pub fn current_food_multiplier(&self) -> f64 {
        let transition_start = self.season_duration - self.transition_duration;
        if self.season_tick >= transition_start {
            let progress =
                (self.season_tick - transition_start) as f64 / self.transition_duration as f64;
            self.current_season
                .food_multiplier_smooth(self.next_season, progress)
        } else {
            self.current_season.food_multiplier()
        }
    }

    #[must_use]
    pub fn current_metabolism_multiplier(&self) -> f64 {
        let transition_start = self.season_duration - self.transition_duration;
        if self.season_tick >= transition_start {
            let progress =
                (self.season_tick - transition_start) as f64 / self.transition_duration as f64;
            self.current_season
                .metabolism_multiplier_smooth(self.next_season, progress)
        } else {
            self.current_season.metabolism_multiplier()
        }
    }

    #[must_use]
    pub fn is_heat_wave(&self) -> bool {
        self.heat_wave_timer >= 10
    }
    #[must_use]
    pub fn is_ice_age(&self) -> bool {
        self.ice_age_timer >= 60
    }
    #[must_use]
    pub fn is_abundance(&self) -> bool {
        self.abundance_timer > 0
    }
    #[must_use]
    pub fn is_radiation_storm(&self) -> bool {
        self.radiation_timer >= 50
    }

    #[must_use]
    pub fn is_hypoxia(&self) -> bool {
        self.oxygen_level < 10.0
    }

    #[must_use]
    pub fn climate(&self) -> ClimateState {
        if let Some(over) = self.god_climate_override {
            return over;
        }

        let carbon_forcing = ((self.carbon_level - 300.0) / 100.0).max(0.0) as f32;
        let effective_cpu = self.cpu_usage + carbon_forcing * 10.0;

        if self.is_heat_wave() {
            ClimateState::Scorching
        } else if self.is_ice_age() || effective_cpu < 30.0 {
            ClimateState::Temperate
        } else if effective_cpu < 60.0 {
            ClimateState::Warm
        } else if effective_cpu < 80.0 {
            ClimateState::Hot
        } else {
            ClimateState::Scorching
        }
    }

    #[must_use]
    pub fn resource_state(&self) -> ResourceState {
        if self.ram_usage_percent < 50.0 {
            ResourceState::Abundant
        } else if self.ram_usage_percent < 70.0 {
            ResourceState::Strained
        } else if self.ram_usage_percent < 85.0 {
            ResourceState::Scarce
        } else {
            ResourceState::Famine
        }
    }

    pub fn metabolism_multiplier(&self) -> f64 {
        let base = if self.is_ice_age() {
            0.5
        } else {
            match self.climate() {
                ClimateState::Temperate => 1.0,
                ClimateState::Warm => 1.5,
                ClimateState::Hot => 2.0,
                ClimateState::Scorching => 3.0,
            }
        };

        let era_mult = match self.current_era {
            Era::Primordial => 1.0,
            Era::DawnOfLife => 0.9,
            Era::Flourishing => 1.1,
            Era::DominanceWar => 1.5,
            Era::ApexEra => 1.2,
        };

        let circadian = if matches!(self.time_of_day(), TimeOfDay::Night) {
            0.6
        } else {
            1.0
        };

        let mut final_mult = base * era_mult * self.current_metabolism_multiplier() * circadian;

        if self.is_hypoxia() {
            final_mult *= 1.5;
        }

        final_mult
    }

    pub fn food_spawn_multiplier(&self) -> f64 {
        let mut base = match self.resource_state() {
            ResourceState::Abundant => 1.0,
            ResourceState::Strained => 0.7,
            ResourceState::Scarce => 0.4,
            ResourceState::Famine => 0.1,
        };
        if self.is_heat_wave() {
            base *= 0.5;
        }
        if self.is_abundance() {
            base *= 2.0;
        }
        base * self.current_food_multiplier()
    }

    pub fn carbon_stress_factor(&self) -> f64 {
        if self.carbon_level < 400.0 {
            0.0
        } else if self.carbon_level < 600.0 {
            0.1
        } else if self.carbon_level < 800.0 {
            0.25
        } else if self.carbon_level < 1200.0 {
            0.5
        } else {
            0.75
        }
    }

    pub fn carbon_enhanced_forest_growth_multiplier(&self) -> f64 {
        if self.carbon_level > 500.0 {
            (self.carbon_level / 500.0).min(2.5)
        } else {
            1.0
        }
    }

    /// Phase 67 Task C: Update DDA multipliers based on average fitness
    /// If avg_fitness > target_fitness, increase difficulty (reduce solar, increase idle cost)
    /// If avg_fitness < target_fitness, decrease difficulty (increase solar, reduce idle cost)
    pub fn update_dda(&mut self, avg_fitness: f64, target_fitness: f64, population: usize) {
        // Only adjust DDA when population is significant
        if population < 10 {
            return;
        }

        let fitness_ratio = avg_fitness / target_fitness;
        let adjustment_rate = 0.001; // Slow, gradual adjustment

        if fitness_ratio > 1.1 {
            // Population too fit - increase difficulty
            self.dda_solar_multiplier =
                (self.dda_solar_multiplier * (1.0 - adjustment_rate)).max(0.5);
            self.dda_base_idle_multiplier =
                (self.dda_base_idle_multiplier * (1.0 + adjustment_rate)).min(2.0);
        } else if fitness_ratio < 0.9 {
            // Population struggling - decrease difficulty
            self.dda_solar_multiplier =
                (self.dda_solar_multiplier * (1.0 + adjustment_rate)).min(2.0);
            self.dda_base_idle_multiplier =
                (self.dda_base_idle_multiplier * (1.0 - adjustment_rate)).max(0.5);
        }
        // If fitness_ratio is within 0.9-1.1, maintain current difficulty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circadian_cycle_progression() {
        let mut env = Environment {
            day_cycle_ticks: 100,
            ..Environment::default()
        };

        assert_eq!(env.time_of_day(), TimeOfDay::Day);

        env.world_time = 25;
        assert!(env.light_level() > 0.9);

        for _ in 0..50 {
            env.tick();
        }
        assert_eq!(env.time_of_day(), TimeOfDay::Night);
        assert_eq!(env.light_level(), 0.1);

        for _ in 0..50 {
            env.tick();
        }
        assert_eq!(env.time_of_day(), TimeOfDay::Day);
    }

    #[test]
    fn test_circadian_metabolism() {
        let mut env = Environment {
            day_cycle_ticks: 100,
            ..Environment::default()
        };

        env.world_time = 0;
        let day_met = env.metabolism_multiplier();

        env.world_time = 60;
        let night_met = env.metabolism_multiplier();

        assert!(night_met < day_met, "Metabolism should be lower at night");
    }
}
