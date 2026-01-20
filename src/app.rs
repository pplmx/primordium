use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEvent, MouseButton, MouseEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph, Sparkline};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;
use uuid::Uuid;

use crate::model::config::AppConfig;
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
    // Layout tracking
    pub last_world_rect: Rect,
    // Live Data
    pub event_log: VecDeque<(String, Color)>,
    pub species_count: usize,
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
            anchor_interval: Duration::from_secs(3600),
            is_anchoring: false,
            screensaver: false,
            show_help: false,
            last_world_rect: Rect::default(),
            event_log: VecDeque::with_capacity(15),
            species_count: 0,
        })
    }

    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);

        while self.running {
            let effective_tick_rate = Duration::from_secs_f64(tick_rate.as_secs_f64() / self.time_scale);

            // 1. Draw
            tui.terminal.draw(|f| {
                let main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(0),
                        if self.show_brain { Constraint::Length(45) } else { Constraint::Length(0) },
                    ])
                    .split(f.size());

                let left_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(5), 
                        Constraint::Length(3), 
                        Constraint::Min(0),    
                        Constraint::Length(7), 
                    ])
                    .split(main_layout[0]);

                self.last_world_rect = left_layout[2];

                if self.screensaver {
                    let world_widget = WorldWidget::new(&self.world, true);
                    f.render_widget(world_widget, f.size());
                } else {
                    let status_lines = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
                        .split(left_layout[0]);

                    let line1 = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(40), Constraint::Percentage(60)]).split(status_lines[0]);
                    let cpu_gauge = Gauge::default().gauge_style(Style::default().fg(Color::Yellow)).percent(self.env.cpu_usage as u16).label(format!("CPU: {:.1}%", self.env.cpu_usage));
                    f.render_widget(cpu_gauge, line1[0]);
                    f.render_widget(Paragraph::new(format!(" | Climate: {}", self.env.climate().icon())), line1[1]);

                    let line2 = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(40), Constraint::Percentage(60)]).split(status_lines[1]);
                    let ram_gauge = Gauge::default().gauge_style(Style::default().fg(Color::Cyan)).percent(self.env.ram_usage_percent as u16).label(format!("RAM: {:.1}%", self.env.ram_usage_percent));
                    f.render_widget(ram_gauge, line2[0]);
                    f.render_widget(Paragraph::new(format!(" | Resources: {}", self.env.resource_state().icon())), line2[1]);

                    let max_gen = self.world.entities.iter().map(|e| e.generation).max().unwrap_or(0);
                    let world_stats = format!(
                        "Pop: {} | Species: {} | Gen: {} | Temp: x{:.1} | Scale: x{:.1} | FPS: {:.1}",
                        self.world.entities.len(),
                        self.species_count,
                        max_gen,
                        self.env.metabolism_multiplier(),
                        self.time_scale,
                        self.fps
                    );
                    f.render_widget(Paragraph::new(world_stats).style(Style::default().fg(Color::DarkGray)), status_lines[2]);

                    let history_data: Vec<u64> = self.cpu_history.iter().cloned().collect();
                    let sparkline = Sparkline::default().block(Block::default().title("CPU Load (60s)").borders(Borders::LEFT | Borders::RIGHT)).data(&history_data).style(Style::default().fg(Color::Yellow));
                    f.render_widget(sparkline, left_layout[1]);

                    let world_widget = WorldWidget::new(&self.world, false);
                    f.render_widget(world_widget, left_layout[2]);

                    let chronicle_block = Block::default().title(" 📜 Live Chronicle ").borders(Borders::ALL);
                    let chronicle_lines: Vec<_> = self.event_log.iter().map(|(msg, color)| ratatui::text::Line::from(ratatui::text::Span::styled(msg, Style::default().fg(*color)))).collect();
                    f.render_widget(Paragraph::new(chronicle_lines).block(chronicle_block), left_layout[3]);

                    if self.show_brain {
                        if let Some(id) = self.selected_entity {
                            if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                                let brain_block = Block::default().title(format!(" 🧬 {} ", entity.name())).borders(Borders::ALL).border_style(Style::default().fg(entity.color()));
                                let mut lines = Vec::new();
                                lines.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(" Energy: ", Style::default().add_modifier(Modifier::BOLD)), ratatui::text::Span::raw(format!("{:.1}/{:.1}", entity.energy, entity.max_energy))]));
                                lines.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(" Age:    ", Style::default().add_modifier(Modifier::BOLD)), ratatui::text::Span::raw(format!("{} ticks", self.world.tick - entity.birth_tick))]));
                                lines.push(ratatui::text::Line::from(vec![ratatui::text::Span::styled(" Offspring: ", Style::default().add_modifier(Modifier::BOLD)), ratatui::text::Span::raw(format!("{}", entity.offspring_count))]));
                                lines.push(ratatui::text::Line::from(""));
                                lines.push(ratatui::text::Line::from(" Neural Network Weights:"));
                                for i in 0..4 {
                                    let mut spans = Vec::new();
                                    spans.push(ratatui::text::Span::raw(format!("  {} ", match i { 0 => "FX", 1 => "FY", 2 => "NR", 3 => "CR", _ => "??" })));
                                    for j in 0..6 {
                                        let w = entity.brain.weights_ih[i * 6 + j];
                                        let symbol = if w > 0.5 { "█" } else if w > 0.0 { "▓" } else if w > -0.5 { "▒" } else { "░" };
                                        spans.push(ratatui::text::Span::styled(symbol, Style::default().fg(if w > 0.0 { Color::Green } else { Color::Red })));
                                    }
                                    lines.push(ratatui::text::Line::from(spans));
                                }
                                lines.push(ratatui::text::Line::from(""));
                                for i in 0..6 {
                                    let mut spans = Vec::new();
                                    spans.push(ratatui::text::Span::raw("    "));
                                    for j in 0..3 {
                                        let w = entity.brain.weights_ho[i * 3 + j];
                                        let symbol = if w > 0.5 { "█" } else if w > 0.0 { "▓" } else if w > -0.5 { "▒" } else { "░" };
                                        spans.push(ratatui::text::Span::styled(symbol, Style::default().fg(if w > 0.0 { Color::Green } else { Color::Red })));
                                    }
                                    if i < 3 { spans.push(ratatui::text::Span::raw(format!("  <- {}", match i { 0 => "Move X", 1 => "Move Y", 2 => "Boost", _ => "" }))); }
                                    lines.push(ratatui::text::Line::from(spans));
                                }
                                f.render_widget(Paragraph::new(lines).block(brain_block), main_layout[1]);
                            }
                        }
                    }
                    if self.show_help {
                        let area = f.size();
                        let help_area = Rect::new(area.width / 4, area.height / 4, area.width / 2, area.height / 2);
                        f.render_widget(Clear, help_area);
                        let help_text = vec![" [Q]     Quit", " [Space] Pause/Resume", " [B]     Toggle Brain View", " [H]     Toggle Help", " [+]     Speed Up", " [-]     Slow Down", " [X]     Genetic Surge", "", " [Left Click]  Select Organism", " [Right Click] Inject Food"];
                        f.render_widget(Paragraph::new(help_text.join("\n")).block(Block::default().title(" Help ").borders(Borders::ALL)), help_area);
                    }
                }
            })?;

            // 2. Hardware & Stats
            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.fps = self.frame_count as f64;
                self.frame_count = 0;
                self.sys.refresh_cpu();
                self.sys.refresh_memory();
                let cpu_usage = self.sys.global_cpu_info().cpu_usage();
                self.env.cpu_usage = cpu_usage;
                self.env.ram_usage_percent = (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;
                let current_climate = self.env.climate();
                if let Some(last) = self.last_climate {
                    if last != current_climate {
                        let ev = LiveEvent::ClimateShift { from: format!("{:?}", last), to: format!("{:?}", current_climate), tick: self.world.tick, timestamp: Utc::now().to_rfc3339() };
                        let _ = self.world.logger.log_event(ev.clone());
                        let (msg, color) = ev.to_ui_message();
                        self.event_log.push_back((msg, color));
                    }
                }
                self.last_climate = Some(current_climate);
                self.env.update_events();
                self.cpu_history.pop_front();
                self.cpu_history.push_back(cpu_usage as u64);
                
                // Genotype-based Species Counting
                let mut representatives: Vec<&crate::model::brain::Brain> = Vec::new();
                let threshold = 5.0; // Distance threshold for new species
                
                for e in &self.world.entities {
                    let mut found = false;
                    for rep in &representatives {
                        if e.brain.genotype_distance(rep) < threshold {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        representatives.push(&e.brain);
                    }
                }
                self.species_count = representatives.len();

                self.last_fps_update = Instant::now();
                // Anchoring logic ...
            }

            // 3. Handle Events
            while event::poll(Duration::ZERO)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Char('q') => self.running = false,
                            KeyCode::Char(' ') => self.paused = !self.paused,
                            KeyCode::Char('b') => self.show_brain = !self.show_brain,
                            KeyCode::Char('h') => self.show_help = !self.show_help,
                            KeyCode::Char('+') | KeyCode::Char('=') => self.time_scale = (self.time_scale + 0.5).min(4.0),
                            KeyCode::Char('-') | KeyCode::Char('_') => self.time_scale = (self.time_scale - 0.5).max(0.5),
                            KeyCode::Char('x') | KeyCode::Char('X') => {
                                for entity in &mut self.world.entities { entity.brain.mutate_with_config(&self.config.evolution); }
                                self.event_log.push_back(("GENETIC SURGE!".to_string(), Color::Red));
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse) => { self.handle_mouse(mouse); }
                    _ => {}
                }
            }

            // 4. Update State
            if last_tick.elapsed() >= effective_tick_rate {
                if !self.paused {
                    let events = self.world.update(&self.env)?;
                    for ev in events {
                        let (msg, color) = ev.to_ui_message();
                        self.event_log.push_back((msg, color));
                        if self.event_log.len() > 15 { self.event_log.pop_front(); }
                    }
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some((wx, wy)) = WorldWidget::screen_to_world(mouse.column, mouse.row, self.last_world_rect, self.screensaver) {
                    let indices = self.world.spatial_hash.query(wx, wy, 2.0);
                    let mut min_dist = f64::MAX;
                    let mut closest_id = None;
                    for idx in indices {
                        if idx >= self.world.entities.len() { continue; }
                        let entity = &self.world.entities[idx];
                        let dx = entity.x - wx;
                        let dy = entity.y - wy;
                        let dist = (dx*dx + dy*dy).sqrt();
                        if dist < min_dist { min_dist = dist; closest_id = Some(entity.id); }
                    }
                    if let Some(id) = closest_id { self.selected_entity = Some(id); self.show_brain = true; }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if let Some((wx, wy)) = WorldWidget::screen_to_world(mouse.column, mouse.row, self.last_world_rect, self.screensaver) {
                    use crate::model::food::Food;
                    self.world.food.push(Food::new(wx as u16, wy as u16));
                    self.event_log.push_back(("Divine Food Injected".to_string(), Color::Green));
                }
            }
            _ => {}
        }
    }
}
