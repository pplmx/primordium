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

pub struct Environment {
    pub cpu_usage: f32,
    pub ram_usage_percent: f32,
    pub load_avg: f64,
    // Event counters
    pub heat_wave_timer: u32,
    pub ice_age_timer: u32,
    pub abundance_timer: u32,
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
        }
    }
}

impl Environment {
    pub fn update_events(&mut self) {
        // Heat Wave: CPU > 80% for 10s
        if self.cpu_usage > 80.0 {
            self.heat_wave_timer += 1;
        } else {
            self.heat_wave_timer = self.heat_wave_timer.saturating_sub(1);
        }

        // Ice Age: CPU < 10% for 60s
        if self.cpu_usage < 10.0 {
            self.ice_age_timer += 1;
        } else {
            self.ice_age_timer = self.ice_age_timer.saturating_sub(1);
        }

        // Abundance: RAM < 40% (No timer needed, instant effect or 30s bonus)
        // We'll use a timer for the 30s bonus as per roadmap
        if self.ram_usage_percent < 40.0 {
            self.abundance_timer = 30;
        } else {
            self.abundance_timer = self.abundance_timer.saturating_sub(1);
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

    pub fn climate(&self) -> ClimateState {
        if self.is_heat_wave() {
            ClimateState::Scorching
        } else if self.is_ice_age() {
            ClimateState::Temperate // Ice age actually reduces metabolism, let's keep it low
        } else if self.cpu_usage < 30.0 {
            ClimateState::Temperate
        } else if self.cpu_usage < 60.0 {
            ClimateState::Warm
        } else if self.cpu_usage < 80.0 {
            ClimateState::Hot
        } else {
            ClimateState::Scorching
        }
    }

    pub fn metabolism_multiplier(&self) -> f64 {
        if self.is_ice_age() {
            return 0.5;
        }
        match self.climate() {
            ClimateState::Temperate => 1.0,
            ClimateState::Warm => 1.5,
            ClimateState::Hot => 2.0,
            ClimateState::Scorching => 3.0,
        }
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

        base
    }
}
