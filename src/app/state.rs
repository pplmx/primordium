use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::config::AppConfig;
use crate::model::state::environment::{ClimateState, Environment};
use crate::model::state::terrain::TerrainType;
use crate::model::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneType {
    Trophic,
    Sensing,
    Speed,
    ReproInvest,
    Maturity,
    MaxEnergy,
}

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub tick_count: u64,
    pub world: World,
    pub config: AppConfig,
    // FPS & Timing
    pub fps: f64,
    pub frame_count: u64,
    pub last_fps_update: Instant,
    pub time_scale: f64,
    // Hardware Coupling
    pub sys: System,
    pub env: Environment,
    pub cpu_history: VecDeque<u64>,
    // Population History
    pub pop_history: VecDeque<u64>,
    // NEW: Phase 56 - Atmospheric History
    pub o2_history: VecDeque<u64>,
    // Neural Visualization
    pub show_brain: bool,
    pub selected_entity: Option<Uuid>,
    pub focused_gene: Option<GeneType>, // NEW: Phase 59
    // Divine Interface v2
    pub brush_type: TerrainType,
    pub social_brush: u8,      // NEW: 0: Normal, 1: Peace, 2: War
    pub is_social_brush: bool, // NEW: Toggle between Terrain and Social brush
    // Phase 34: Ancestry View
    pub show_ancestry: bool,
    // Last climate state for shift logging
    pub last_climate: Option<ClimateState>,
    // Blockchain Anchoring
    pub last_anchor_time: Instant,
    pub anchor_interval: Duration,
    pub is_anchoring: bool,
    // Modes
    pub screensaver: bool,
    pub show_help: bool,
    pub help_tab: u8, // 0=Controls, 1=Symbols, 2=Concepts, 3=Eras
    // Phase 40: Archeology View
    pub show_archeology: bool,
    pub archeology_snapshots: Vec<(u64, crate::model::history::PopulationStats)>,
    pub archeology_index: usize,
    pub selected_fossil_index: usize, // NEW
    pub onboarding_step: Option<u8>,  // None=done, Some(0-2)=onboarding screens
    pub view_mode: u8, // 0: Normal, 1: Fertility, 2: Social, 3: Rank, 4: Vocal, 5: Market, 6: Research
    // Layout tracking
    pub last_world_rect: Rect,
    pub last_sidebar_rect: Rect,
    pub gene_editor_offset: u16, // NEW: Phase 59
    // Live Data
    pub event_log: VecDeque<(String, Color)>,

    pub network_state: crate::model::infra::network::NetworkState, // NEW
    pub latest_snapshot: Option<crate::model::state::snapshot::WorldSnapshot>,
    pub network: Option<crate::client::manager::NetworkManager>,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let config = AppConfig::load();

        let world = if std::path::Path::new("save.json").exists() {
            if let Ok(data) = std::fs::read_to_string("save.json") {
                if let Ok(w) = serde_json::from_str::<World>(&data) {
                    w
                } else {
                    World::new(config.world.initial_population, config.clone())?
                }
            } else {
                World::new(config.world.initial_population, config.clone())?
            }
        } else {
            World::new(config.world.initial_population, config.clone())?
        };

        let latest_snapshot = Some(world.create_snapshot(None));

        Ok(Self {
            running: true,
            paused: false,
            tick_count: world.tick,
            world,
            config,
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            time_scale: 1.0,
            sys,
            env: Environment::default(),
            cpu_history: VecDeque::from(vec![0; 60]),
            pop_history: VecDeque::from(vec![0; 60]),
            o2_history: VecDeque::from(vec![0; 60]),
            show_brain: false,
            selected_entity: None,
            focused_gene: None,
            brush_type: TerrainType::Plains,
            social_brush: 0,
            is_social_brush: false,
            show_ancestry: false,
            last_climate: None,
            last_anchor_time: Instant::now(),
            anchor_interval: Duration::from_secs(3600),
            is_anchoring: false,
            screensaver: false,
            show_help: false,
            help_tab: 0,
            show_archeology: false,
            archeology_snapshots: Vec::new(),
            archeology_index: 0,
            selected_fossil_index: 0,
            onboarding_step: if std::path::Path::new(".primordium_onboarded").exists() {
                None
            } else {
                Some(0) // Start onboarding for first-time users
            },
            view_mode: 0,
            last_world_rect: Rect::default(),
            last_sidebar_rect: Rect::default(),
            gene_editor_offset: 20,
            event_log: VecDeque::with_capacity(15),
            network_state: crate::model::infra::network::NetworkState::default(),
            latest_snapshot,
            network: None,
        })
    }

    pub fn connect(&mut self, url: &str) {
        self.network = Some(crate::client::manager::NetworkManager::new(url));
    }

    pub fn save_state(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.world)?;
        std::fs::write("save.json", data)?;
        Ok(())
    }

    pub fn load_state(&mut self) -> Result<()> {
        let data = std::fs::read_to_string("save.json")?;
        let world: World = serde_json::from_str(&data)?;
        self.world = world;
        self.tick_count = self.world.tick;
        Ok(())
    }
}
