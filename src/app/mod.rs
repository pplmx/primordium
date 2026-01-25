pub mod help;
pub mod input;
pub mod onboarding;
pub mod render;
pub mod state;

pub use state::App;

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyEventKind};
use std::time::{Duration, Instant};
// use sysinfo::System; (removed as unused)

use crate::model::history::LiveEvent;
use crate::model::systems::environment as environment_system;
use crate::ui::tui::Tui;

impl App {
    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);

        while self.running {
            let effective_tick_rate =
                Duration::from_secs_f64(tick_rate.as_secs_f64() / self.time_scale);

            // 1. Draw
            tui.terminal.draw(|f| {
                self.draw(f);
            })?;

            // 2. Hardware & Stats
            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.update_hardware_metrics();
            }

            // 3. Handle Events
            while event::poll(Duration::ZERO)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key);
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse);
                    }
                    _ => {}
                }
            }

            // 4. Update State
            if last_tick.elapsed() >= effective_tick_rate {
                if !self.paused {
                    self.update_world()?;
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    fn update_hardware_metrics(&mut self) {
        self.fps = self.frame_count as f64;
        self.frame_count = 0;
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        self.env.cpu_usage = cpu_usage;
        self.env.ram_usage_percent =
            (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;

        environment_system::update_era(&mut self.env, self.world.tick, &self.world.pop_stats);

        let current_climate = self.env.climate();
        if let Some(last) = self.last_climate {
            if last != current_climate {
                let ev = LiveEvent::ClimateShift {
                    from: format!("{:?}", last),
                    to: format!("{:?}", current_climate),
                    tick: self.world.tick,
                    timestamp: Utc::now().to_rfc3339(),
                };
                let _ = self.world.logger.log_event(ev.clone());
                let (msg, color) = ev.to_ui_message();
                self.event_log.push_back((msg, color));
            }
        }
        self.last_climate = Some(current_climate);
        environment_system::update_events(&mut self.env);

        self.cpu_history.pop_front();
        self.cpu_history.push_back(cpu_usage as u64);

        self.pop_history.pop_front();
        self.pop_history.push_back(self.world.entities.len() as u64);

        self.o2_history.pop_front();
        self.o2_history.push_back(self.env.oxygen_level as u64);

        self.last_fps_update = Instant::now();
    }

    fn update_world(&mut self) -> Result<()> {
        let events = self.world.update(&mut self.env)?;
        for ev in events {
            let (msg, color) = ev.to_ui_message();
            self.event_log.push_back((msg, color));
            if self.event_log.len() > 15 {
                self.event_log.pop_front();
            }
        }
        Ok(())
    }
}
