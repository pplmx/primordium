#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Era {
    #[default]
    Primordial, // chaotic adaptation
    DawnOfLife,   // stable population
    Flourishing,  // high diversity
    DominanceWar, // high predation
    ApexEra,      // peak fitness
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClimateState {
    Temperate,
    Warm,
    Hot,
    Scorching,
}

impl ClimateState {
    pub fn icon(&self) -> &'static str {
        match self {
            ClimateState::Temperate => "🌡️ Temperate",
            ClimateState::Warm => "🔥 Warm",
            ClimateState::Hot => "🌋 Hot",
            ClimateState::Scorching => "☀️ Scorching",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResourceState {
    Abundant,
    Strained,
    Scarce,
    Famine,
}

impl ResourceState {
    pub fn icon(&self) -> &'static str {
        match self {
            ResourceState::Abundant => "🌾 Abundant",
            ResourceState::Strained => "⚠️ Strained",
            ResourceState::Scarce => "🚨 Scarce",
            ResourceState::Famine => "💀 Famine",
        }
    }
}

/// Seasons that affect metabolism and food spawning
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Season {
    #[default]
    Spring, // Food ×1.5, Metabolism ×0.8
    Summer, // Food ×1.0, Metabolism ×1.2
    Fall,   // Food ×1.2, Metabolism ×1.0
    Winter, // Food ×0.5, Metabolism ×1.5
}

impl Season {
    pub fn icon(&self) -> &'static str {
        match self {
            Season::Spring => "🌸 Spring",
            Season::Summer => "☀️ Summer",
            Season::Fall => "🍂 Fall",
            Season::Winter => "❄️ Winter",
        }
    }

    /// Food spawn multiplier for this season
    pub fn food_multiplier(&self) -> f64 {
        match self {
            Season::Spring => 1.5,
            Season::Summer => 1.0,
            Season::Fall => 1.2,
            Season::Winter => 0.5,
        }
    }

    /// Metabolism multiplier for this season
    pub fn metabolism_multiplier(&self) -> f64 {
        match self {
            Season::Spring => 0.8,
            Season::Summer => 1.2,
            Season::Fall => 1.0,
            Season::Winter => 1.5,
        }
    }

    /// Get next season in the cycle
    pub fn next(&self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Day,
    Night,
}

impl TimeOfDay {
    pub fn icon(&self) -> &'static str {
        match self {
            TimeOfDay::Day => "☀️ Day",
            TimeOfDay::Night => "🌙 Night",
        }
    }
}

pub struct Environment {
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
    pub load_avg: f64,
    // Event counters
    pub heat_wave_timer: u32,
    pub ice_age_timer: u32,
    pub abundance_timer: u32,
    // Era System
    pub current_era: Era,
    // Season System
    pub current_season: Season,
    pub season_tick: u64,
    pub season_duration: u64, // Ticks per season (default: 10000)
    // Circadian System
    pub world_time: u64,      // 0 to day_cycle_ticks
    pub day_cycle_ticks: u64, // Default: 2000
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
            current_era: Era::Primordial,
            current_season: Season::Spring,
            season_tick: 0,
            season_duration: 10000,
            world_time: 0,
            day_cycle_ticks: 2000,
        }
    }
}

impl Environment {
    pub fn tick(&mut self) {
        self.world_time = (self.world_time + 1) % self.day_cycle_ticks;
    }

    pub fn time_of_day(&self) -> TimeOfDay {
        if self.world_time < self.day_cycle_ticks / 2 {
            TimeOfDay::Day
        } else {
            TimeOfDay::Night
        }
    }

    /// Light level from 0.0 (darkest night) to 1.0 (brightest day)
    pub fn light_level(&self) -> f32 {
        let half_cycle = self.day_cycle_ticks as f32 / 2.0;
        let progress = (self.world_time as f32 % self.day_cycle_ticks as f32) / half_cycle;

        if progress < 1.0 {
            // Day: peak at middle of day
            let x = progress - 0.5;
            1.0 - (x * x * 4.0) // Simple parabolic arc
        } else {
            // Night: very low light
            0.1
        }
    }

    pub fn is_heat_wave(&self) -> bool {
        self.heat_wave_timer >= 10
    }
    pub fn is_ice_age(&self) -> bool {
        self.ice_age_timer >= 60
    }
    pub fn is_abundance(&self) -> bool {
        self.abundance_timer > 0
    }

    pub fn climate(&self) -> ClimateState {
        if self.is_heat_wave() {
            ClimateState::Scorching
        } else if self.is_ice_age() || self.cpu_usage < 30.0 {
            ClimateState::Temperate
        } else if self.cpu_usage < 60.0 {
            ClimateState::Warm
        } else if self.cpu_usage < 80.0 {
            ClimateState::Hot
        } else {
            ClimateState::Scorching
        }
    }

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

        // Apply circadian mult (rest at night)
        let circadian = if matches!(self.time_of_day(), TimeOfDay::Night) {
            0.6 // 40% reduction at night
        } else {
            1.0
        };

        // Apply season modifier
        base * self.current_season.metabolism_multiplier() * circadian
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
        // Apply season modifier
        base * self.current_season.food_multiplier()
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

        // Initially day
        assert_eq!(env.time_of_day(), TimeOfDay::Day);

        // Midday (ticks/4 = 25)
        env.world_time = 25;
        assert!(env.light_level() > 0.9);

        // Progress to night
        for _ in 0..50 {
            env.tick(); // Now 75
        }
        assert_eq!(env.time_of_day(), TimeOfDay::Night);
        assert_eq!(env.light_level(), 0.1);

        // Reset to day
        for _ in 0..50 {
            env.tick(); // Now 125 % 100 = 25
        }
        assert_eq!(env.time_of_day(), TimeOfDay::Day);
    }

    #[test]
    fn test_circadian_metabolism() {
        let mut env = Environment {
            day_cycle_ticks: 100,
            ..Environment::default()
        };

        // Day metabolism
        env.world_time = 0;
        let day_met = env.metabolism_multiplier();

        // Night metabolism
        env.world_time = 60;
        let night_met = env.metabolism_multiplier();

        assert!(night_met < day_met, "Metabolism should be lower at night");
    }
}
