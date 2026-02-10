use anyhow::Result;
use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::config::AppConfig;
use crate::model::environment::{ClimateState, Environment};
use crate::model::terrain::TerrainType;
use crate::model::world::World;
use primordium_data::GeneType;

#[derive(Serialize, Deserialize, Clone)]
pub struct InputEvent {
    pub tick: u64,
    pub event: Event,
}

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub tick_count: u64,
    pub world: World,
    pub config: AppConfig,
    pub config_path: String,
    pub config_last_modified: Option<std::time::SystemTime>,
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
    pub cinematic_mode: bool,
    pub show_help: bool,
    pub show_legend: bool,
    pub help_tab: u8, // 0=Controls, 1=Symbols, 2=Concepts, 3=Eras
    // Phase 40: Archeology View
    pub show_archeology: bool,
    pub auto_play_history: bool, // NEW: Replay functionality
    pub archeology_snapshots: Vec<(u64, crate::model::history::PopulationStats)>,
    pub archeology_index: usize,
    pub selected_fossil_index: usize, // NEW
    pub onboarding_step: Option<u8>,  // None=done, Some(0-2)=onboarding screens
    pub view_mode: u8,
    // Layout tracking
    pub last_world_rect: Rect,
    pub last_sidebar_rect: Rect,
    pub gene_editor_offset: u16, // NEW: Phase 59
    // Live Data
    pub event_log: VecDeque<(String, Color)>,

    pub network_state: crate::model::infra::network::NetworkState,
    pub latest_snapshot: Option<crate::model::snapshot::WorldSnapshot>,
    pub network: Option<crate::client::manager::NetworkManager>,

    pub hof_query_rx: Option<std::sync::mpsc::Receiver<Vec<(Uuid, u32, bool)>>>,
    pub cached_hall_of_fame: Vec<(Uuid, u32, bool)>,

    pub input_log: Vec<InputEvent>,
    pub replay_queue: VecDeque<InputEvent>,
    pub replay_mode: bool,
}

impl App {
    pub fn load_config() -> AppConfig {
        let config_path = "config.toml";
        if let Ok(content) = std::fs::read_to_string(config_path) {
            match AppConfig::from_toml(&content) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Warning: Failed to load {}: {}", config_path, e);
                }
            }
        }
        let default = AppConfig::default();
        if !std::path::Path::new(config_path).exists() {
            if let Ok(toml_str) = toml::to_string(&default) {
                let _ = std::fs::write(config_path, toml_str);
            }
        }
        default
    }

    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let config = Self::load_config();

        let world = if std::path::Path::new("save.json").exists() {
            match crate::model::persistence::load_world("save.json") {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Failed to load save file: {}", e);
                    World::new(config.world.initial_population, config.clone())?
                }
            }
        } else {
            World::new(config.world.initial_population, config.clone())?
        };

        let latest_snapshot = Some(world.create_snapshot(None));
        let config_path = "config.toml".to_string();
        let config_last_modified = std::fs::metadata(&config_path)
            .ok()
            .and_then(|m| m.modified().ok());

        Ok(Self {
            running: true,
            paused: false,
            tick_count: world.tick,
            world,
            config,
            config_path,
            config_last_modified,
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
            cinematic_mode: false,
            show_help: false,
            show_legend: false,
            help_tab: 0,
            show_archeology: false,
            auto_play_history: false,
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
            hof_query_rx: None,
            cached_hall_of_fame: Vec::new(),
            input_log: Vec::new(),
            replay_queue: VecDeque::new(),
            replay_mode: false,
        })
    }

    pub fn connect(&mut self, url: &str) {
        self.network = Some(crate::client::manager::NetworkManager::new(url));
    }

    pub fn save_state(&mut self) -> Result<()> {
        crate::model::persistence::save_world(&mut self.world, "save.json")?;
        Ok(())
    }

    pub fn backup_state(&mut self) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("backups/world_{}.json", timestamp);
        std::fs::create_dir_all("backups")?;
        crate::model::persistence::save_world(&mut self.world, &filename)?;
        Ok(())
    }

    pub fn load_state(&mut self) -> Result<()> {
        let world = crate::model::persistence::load_world("save.json")?;
        self.world = world;
        self.tick_count = self.world.tick;
        Ok(())
    }

    pub fn save_recording(&self) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("logs/input_trace_{}.json", timestamp);
        std::fs::create_dir_all("logs")?;
        let data = serde_json::to_string_pretty(&self.input_log)?;
        std::fs::write(&filename, data)?;
        tracing::info!("Input trace saved to {}", filename);
        Ok(())
    }

    pub fn load_replay(&mut self, path: &str) -> Result<()> {
        let data = std::fs::read_to_string(path)?;
        let log: Vec<InputEvent> = serde_json::from_str(&data)?;
        self.replay_queue = VecDeque::from(log);
        self.replay_mode = true;
        tracing::info!("Replay loaded: {} events", self.replay_queue.len());
        Ok(())
    }

    pub fn check_config_reload(&mut self) -> Result<bool> {
        let config_path = &self.config_path;
        if let Ok(metadata) = std::fs::metadata(config_path) {
            let modified = metadata.modified()?;
            if Some(modified) != self.config_last_modified {
                let new_config = Self::load_config();

                // Only update non-critical config values
                // Don't change world dimensions or initial population
                self.config.metabolism = new_config.metabolism;
                self.config.evolution = new_config.evolution;
                self.config.brain = new_config.brain;
                self.config.social = new_config.social;
                self.config.terraform = new_config.terraform;
                self.config.ecosystem = new_config.ecosystem;
                self.config.target_fps = new_config.target_fps;

                self.config_last_modified = Some(modified);

                self.event_log.push_back((
                    "Configuration reloaded from config.toml".to_string(),
                    ratatui::style::Color::Green,
                ));

                return Ok(true);
            }
        }
        Ok(false)
    }
}
