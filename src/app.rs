use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph, Sparkline};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::blockchain::{AnchorRecord, BlockchainProvider, OpenTimestampsProvider};
use crate::model::config::AppConfig;
use crate::model::environment::{ClimateState, Environment};
use crate::model::history::{HistoryLogger, LiveEvent};
use crate::model::world::World;
use crate::ui::renderer::WorldWidget;
use crate::ui::tui::Tui;

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
            show_brain: false,
            selected_entity: None,
            last_climate: None,
            last_anchor_time: Instant::now(),
            anchor_interval: Duration::from_secs(3600), // 1 hour
            is_anchoring: false,
            screensaver: false,
            show_help: false,
        })
    }

    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16); // ~60 FPS

        while self.running {
            // Adjust tick rate based on time scale
            let effective_tick_rate =
                Duration::from_secs_f64(tick_rate.as_secs_f64() / self.time_scale);

            // 1. Draw
            tui.terminal.draw(|f| {
                if self.screensaver {
                    let world_widget = WorldWidget::new(&self.world, true);
                    f.render_widget(world_widget, f.size());
                } else {
                    let main_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Min(0),
                            if self.show_brain {
                                Constraint::Length(40)
                            } else {
                                Constraint::Length(0)
                            },
                        ])
                        .split(f.size());

                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(5),
                            Constraint::Length(3),
                            Constraint::Min(0),
                        ])
                        .split(main_layout[0]);

                    // STATUS BAR
                    let status_lines = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ])
                        .split(chunks[0]);

                    let line1 = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(status_lines[0]);

                    let cpu_gauge = Gauge::default()
                        .gauge_style(Style::default().fg(Color::Yellow))
                        .percent(self.env.cpu_usage as u16)
                        .label(format!("CPU: {:.1}%", self.env.cpu_usage));
                    f.render_widget(cpu_gauge, line1[0]);
                    f.render_widget(
                        Paragraph::new(format!(" | Climate: {}", self.env.climate().icon())),
                        line1[1],
                    );

                    let line2 = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                        .split(status_lines[1]);

                    let ram_gauge = Gauge::default()
                        .gauge_style(Style::default().fg(Color::Cyan))
                        .percent(self.env.ram_usage_percent as u16)
                        .label(format!("RAM: {:.1}%", self.env.ram_usage_percent));
                    f.render_widget(ram_gauge, line2[0]);
                    f.render_widget(
                        Paragraph::new(format!(
                            " | Resources: {}",
                            self.env.resource_state().icon()
                        )),
                        line2[1],
                    );

                    let max_gen = self
                        .world
                        .entities
                        .iter()
                        .map(|e| e.generation)
                        .max()
                        .unwrap_or(0);
                    let anchor_status = if self.is_anchoring { " [ANCHORING...]" } else { "" };
                    let world_stats = format!(
                        "Pop: {} | Gen: {} | Temp: x{:.1} | Food: x{:.1} | FPS: {:.1} | Scale: x{:.1}{}",
                        self.world.entities.len(),
                        max_gen,
                        self.env.metabolism_multiplier(),
                        self.env.food_spawn_multiplier(),
                        self.fps,
                        self.time_scale,
                        anchor_status
                    );
                    f.render_widget(
                        Paragraph::new(world_stats).style(Style::default().fg(Color::DarkGray)),
                        status_lines[2],
                    );

                    let history_data: Vec<u64> = self.cpu_history.iter().cloned().collect();
                    let sparkline = Sparkline::default()
                        .block(
                            Block::default()
                                .title("CPU Load (60s)")
                                .borders(Borders::LEFT | Borders::RIGHT),
                        )
                        .data(&history_data)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(sparkline, chunks[1]);

                    let world_widget = WorldWidget::new(&self.world, false);
                    f.render_widget(world_widget, chunks[2]);

                    if self.show_brain {
                        if let Some(id) = self.selected_entity {
                            if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                                let brain_block = Block::default()
                                    .title(format!(
                                        " Brain: {} ",
                                        entity.id.to_string()[..8].to_string()
                                    ))
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(entity.color()));

                                let mut lines = Vec::new();
                                lines.push(ratatui::text::Line::from("Input -> Hidden Weights:"));
                                for i in 0..4 {
                                    let mut spans = Vec::new();
                                    for j in 0..6 {
                                        let w = entity.brain.weights_ih[i * 6 + j];
                                        let symbol = if w > 0.5 {
                                            "█"
                                        } else if w > 0.0 {
                                            "▓"
                                        } else if w > -0.5 {
                                            "▒"
                                        } else {
                                            "░"
                                        };
                                        let color = if w > 0.0 { Color::Green } else { Color::Red };
                                        spans.push(ratatui::text::Span::styled(
                                            symbol,
                                            Style::default().fg(color),
                                        ));
                                    }
                                    lines.push(ratatui::text::Line::from(spans));
                                }
                                lines.push(ratatui::text::Line::from(""));
                                lines.push(ratatui::text::Line::from("Hidden -> Output Weights:"));
                                for i in 0..6 {
                                    let mut spans = Vec::new();
                                    for j in 0..3 {
                                        let w = entity.brain.weights_ho[i * 3 + j];
                                        let symbol = if w > 0.5 {
                                            "█"
                                        } else if w > 0.0 {
                                            "▓"
                                        } else if w > -0.5 {
                                            "▒"
                                        } else {
                                            "░"
                                        };
                                        let color = if w > 0.0 { Color::Green } else { Color::Red };
                                        spans.push(ratatui::text::Span::styled(
                                            symbol,
                                            Style::default().fg(color),
                                        ));
                                    }
                                    lines.push(ratatui::text::Line::from(spans));
                                }
                                f.render_widget(Paragraph::new(lines).block(brain_block), main_layout[1]);
                            }
                        }
                    }

                    if self.show_help {
                        let area = f.size();
                        let help_area = Rect::new(
                            area.width / 4,
                            area.height / 4,
                            area.width / 2,
                            area.height / 2,
                        );
                        f.render_widget(Clear, help_area);
                        let help_text = vec![
                            " [Q]     Quit",
                            " [Space] Pause/Resume",
                            " [B]     Toggle Brain View",
                            " [H]     Toggle Help",
                            " [+]     Speed Up",
                            " [-]     Slow Down",
                            " [R]     Reset World (TBD)",
                        ];
                        let help_block = Paragraph::new(help_text.join("\n"))
                            .block(Block::default().title(" Help ").borders(Borders::ALL));
                        f.render_widget(help_block, help_area);
                    }
                }
            })?;

            // 2. Hardware Polling (1s interval)
            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.fps = self.frame_count as f64;
                self.frame_count = 0;
                self.sys.refresh_cpu();
                self.sys.refresh_memory();
                let cpu_usage = self.sys.global_cpu_info().cpu_usage();
                self.env.cpu_usage = cpu_usage;
                self.env.ram_usage_percent =
                    (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;

                let current_climate = self.env.climate();
                if let Some(last) = self.last_climate {
                    if last != current_climate {
                        let _ = self.world.logger.log_event(LiveEvent::ClimateShift {
                            from: format!("{:?}", last),
                            to: format!("{:?}", current_climate),
                            tick: self.world.tick,
                            timestamp: Utc::now().to_rfc3339(),
                        });
                    }
                }
                self.last_climate = Some(current_climate);

                self.env.update_events();
                self.cpu_history.pop_front();
                self.cpu_history.push_back(cpu_usage as u64);
                self.last_fps_update = Instant::now();

                // Blockchain Anchoring Check
                if self.last_anchor_time.elapsed() >= self.anchor_interval && !self.is_anchoring {
                    self.is_anchoring = true;
                    self.last_anchor_time = Instant::now();

                    if let Ok(legends) = self.world.logger.get_all_legends() {
                        if !legends.is_empty() {
                            if let Ok(hash) = HistoryLogger::compute_legends_hash(&legends) {
                                tokio::spawn(async move {
                                    let provider = OpenTimestampsProvider;
                                    if let Ok(tx_id) = provider.anchor_hash(&hash).await {
                                        let record = AnchorRecord {
                                            hash,
                                            tx_id,
                                            timestamp: Utc::now().to_rfc3339(),
                                            provider: "OpenTimestamps".to_string(),
                                        };
                                        if let Ok(mut file) = OpenOptions::new()
                                            .create(true)
                                            .append(true)
                                            .open("logs/anchors.jsonl")
                                        {
                                            if let Ok(json) = serde_json::to_string(&record) {
                                                let _ = writeln!(file, "{}", json);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                    self.is_anchoring = false;
                }
            }

            // 3. Handle Events
            let timeout = effective_tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => self.running = false,
                            KeyCode::Char(' ') => self.paused = !self.paused,
                            KeyCode::Char('b') => self.show_brain = !self.show_brain,
                            KeyCode::Char('h') => self.show_help = !self.show_help,
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                self.time_scale = (self.time_scale + 0.5).min(4.0)
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                self.time_scale = (self.time_scale - 0.5).max(0.5)
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 4. Update State
            if last_tick.elapsed() >= effective_tick_rate {
                if !self.paused {
                    self.world.update(&self.env)?;
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }
}
