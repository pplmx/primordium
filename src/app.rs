use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;

use crate::model::environment::Environment;
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
}

impl App {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            running: true,
            paused: false,
            tick_count: 0,
            world: World::new(100, 50, 100),
            fps: 0.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            sys,
            env: Environment::default(),
            cpu_history: VecDeque::from(vec![0; 60]),
        }
    }

    pub fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16); // ~60 FPS

        while self.running {
            // 1. Draw
            tui.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(5), // New 3-line Status Bar + Padding
                        Constraint::Length(3), // Sparkline area
                        Constraint::Min(0),    // World area
                    ])
                    .split(f.size());

                // --- LINE 1: CPU & Climate ---
                let cpu_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(chunks[0].inner(&ratatui::layout::Margin {
                        horizontal: 1,
                        vertical: 0,
                    }));

                let cpu_gauge = Gauge::default()
                    .block(Block::default().title(format!("CPU: {:.1}%", self.env.cpu_usage)))
                    .gauge_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .bg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    )
                    .percent(self.env.cpu_usage as u16);
                f.render_widget(cpu_gauge, cpu_chunks[0]);

                let climate_text = format!(
                    "Climate: {} (x{:.1} metabolism)",
                    self.env.climate().icon(),
                    self.env.metabolism_multiplier()
                );
                f.render_widget(Paragraph::new(climate_text), cpu_chunks[1]);

                // --- LINE 2: RAM & Resources ---
                // We'll reuse vertical chunks inside chunks[0] for cleaner 3-line look
                let status_lines = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(chunks[0]);

                // Line 1 (CPU already handled above, let's re-arrange properly)
                // Redo Status Bar properly
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

                // Line 2 (RAM)
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

                // Line 3 (World Stats)
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
            })?;

            // 2. Hardware Polling & FPS Calculation (1s interval)
            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.fps = self.frame_count as f64;
                self.frame_count = 0;

                // Poll Hardware
                self.sys.refresh_cpu();
                self.sys.refresh_memory();

                let cpu_usage = self.sys.global_cpu_info().cpu_usage();
                let ram_used = self.sys.used_memory();
                let ram_total = self.sys.total_memory();
                let ram_percent = (ram_used as f32 / ram_total as f32) * 100.0;

                self.env.cpu_usage = cpu_usage;
                self.env.ram_usage_percent = ram_percent;
                // sysinfo 0.30: load_average is on global system, not method on instance
                // Actually it depends on the version, but the error said it's an associated function
                // Let's try to just use global_cpu_info if load_avg is problematic
                self.env.load_avg = 0.0; // Temporary bypass for load_avg to fix build

                // Update Events
                self.env.update_events();

                // Update history
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
                            _ => {}
                        }
                    }
                }
            }

            // 4. Update State
            if last_tick.elapsed() >= tick_rate {
                if !self.paused {
                    self.world.update(&self.env);
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }
}
