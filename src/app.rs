use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::environment::{ClimateState, Environment};
use crate::model::history::LiveEvent;
use crate::model::world::World;
use crate::ui::renderer::WorldWidget;
use crate::ui::tui::Tui;

pub struct App {
    pub running: bool,
    pub paused: bool,
    pub tick_count: u64,
    pub world: World,
    // FPS Tracking
    pub fps: f64,
    pub frame_count: u64,
    pub last_fps_update: Instant,
    // Hardware Coupling
    pub sys: System,
    pub env: Environment,
    pub cpu_history: VecDeque<u64>,
    // Neural Visualization
    pub show_brain: bool,
    pub selected_entity: Option<Uuid>,
    // Last climate state for shift logging
    pub last_climate: Option<ClimateState>,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let world = World::new(100, 50, 100)?;

        Ok(Self {
            running: true,
            paused: false,
            tick_count: 0,
            world,
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            sys,
            env: Environment::default(),
            cpu_history: VecDeque::from(vec![0; 60]),
            show_brain: false,
            selected_entity: None,
            last_climate: None,
        })
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16); // ~60 FPS

        while self.running {
            // 1. Draw
            tui.terminal.draw(|f| {
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

                // --- STATUS BAR ---
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
                let world_stats = format!(
                    "Pop: {} | Gen: {} | Temp: x{:.1} | Food: x{:.1} | FPS: {:.1}",
                    self.world.entities.len(),
                    max_gen,
                    self.env.metabolism_multiplier(),
                    self.env.food_spawn_multiplier(),
                    self.fps
                );
                f.render_widget(
                    Paragraph::new(world_stats).style(Style::default().fg(Color::DarkGray)),
                    status_lines[2],
                );

                // --- Sparkline ---
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

                // --- World ---
                let world_widget = WorldWidget::new(&self.world);
                f.render_widget(world_widget, chunks[2]);

                // --- Brain Visualization ---
                if self.show_brain {
                    if self.selected_entity.is_none()
                        || !self
                            .world
                            .entities
                            .iter()
                            .any(|e| Some(e.id) == self.selected_entity)
                    {
                        self.selected_entity = self
                            .world
                            .entities
                            .iter()
                            .max_by_key(|e| e.generation)
                            .map(|e| e.id);
                    }

                    if let Some(id) = self.selected_entity {
                        if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                            let brain_block = Block::default()
                                .title(format!(
                                    " Brain: {} (Gen {}) ",
                                    entity.id.to_string().get(0..8).unwrap_or(""),
                                    entity.generation
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

                            f.render_widget(
                                Paragraph::new(lines).block(brain_block),
                                main_layout[1],
                            );
                        }
                    }
                }
            })?;

            // 2. Hardware Polling & FPS Calculation (1s interval)
            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.fps = self.frame_count as f64;
                self.frame_count = 0;

                self.sys.refresh_cpu();
                self.sys.refresh_memory();

                let cpu_usage = self.sys.global_cpu_info().cpu_usage();
                let ram_used = self.sys.used_memory();
                let ram_total = self.sys.total_memory();
                let ram_percent = (ram_used as f32 / ram_total as f32) * 100.0;

                self.env.cpu_usage = cpu_usage;
                self.env.ram_usage_percent = ram_percent;
                self.env.load_avg = 0.0;

                let current_climate = self.env.climate();
                if let Some(last) = self.last_climate {
                    if last != current_climate {
                        self.world.logger.log_event(LiveEvent::ClimateShift {
                            from: format!("{:?}", last),
                            to: format!("{:?}", current_climate),
                            tick: self.world.tick,
                            timestamp: Utc::now().to_rfc3339(),
                        })?;
                    }
                }
                self.last_climate = Some(current_climate);

                self.env.update_events();
                self.cpu_history.pop_front();
                self.cpu_history.push_back(cpu_usage as u64);

                self.last_fps_update = Instant::now();
            }

            // 3. Handle Events
            let elapsed = last_tick.elapsed();
            let timeout = if elapsed >= tick_rate {
                Duration::from_millis(0)
            } else {
                tick_rate - elapsed
            };

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') => self.running = false,
                            KeyCode::Char(' ') => self.paused = !self.paused,
                            KeyCode::Char('b') | KeyCode::Char('B') => {
                                self.show_brain = !self.show_brain
                            }
                            _ => {}
                        }
                    }
                }
            }

            // 4. Update State
            if last_tick.elapsed() >= tick_rate {
                if !self.paused {
                    self.world.update(&self.env)?;
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }
}
