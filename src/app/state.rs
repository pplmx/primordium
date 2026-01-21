use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::config::AppConfig;
use crate::model::environment::{ClimateState, Environment};
use crate::model::world::World;

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
    // Neural Visualization
    pub show_brain: bool,
    pub selected_entity: Option<Uuid>,
    // Last climate state for shift logging
    pub last_climate: Option<ClimateState>,
    // Blockchain Anchoring
    pub last_anchor_time: Instant,
    pub anchor_interval: Duration,
    pub is_anchoring: bool,
    // Modes
    pub screensaver: bool,
    pub show_help: bool,
    pub help_tab: u8,                // 0=Controls, 1=Symbols, 2=Concepts, 3=Eras
    pub onboarding_step: Option<u8>, // None=done, Some(0-2)=onboarding screens
    // Layout tracking
    pub last_world_rect: Rect,
    // Live Data
    pub event_log: VecDeque<(String, Color)>,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let config = AppConfig::load();
        let world = World::new(config.world.initial_population, config.clone())?;
        Ok(Self {
            running: true,
            paused: false,
            tick_count: 0,
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
            show_brain: false,
            selected_entity: None,
            last_climate: None,
            last_anchor_time: Instant::now(),
            anchor_interval: Duration::from_secs(3600),
            is_anchoring: false,
            screensaver: false,
            show_help: false,
            help_tab: 0,
            onboarding_step: if std::path::Path::new(".primordium_onboarded").exists() {
                None
            } else {
                Some(0) // Start onboarding for first-time users
            },
            last_world_rect: Rect::default(),
            event_log: VecDeque::with_capacity(15),
        })
    }
}
