pub mod input;
pub mod onboarding;
pub mod render;
pub mod shutdown;
pub mod state;

pub use shutdown::ShutdownManager;
pub use state::App;

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyEventKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::model::history::LiveEvent;
use primordium_core::systems::environment as environment_system;
use primordium_tui::Tui;
use ratatui::style::Color;
use sysinfo::Pid;
use uuid::Uuid;

impl App {
    pub async fn run(&mut self, tui: &mut Tui) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(16);
        let mut last_config_check = Instant::now();

        // Setup shutdown handler
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            tracing::info!("Ctrl+C received, initiating graceful shutdown...");
            shutdown_clone.store(true, Ordering::SeqCst);
        });

        while self.running && !shutdown.load(Ordering::SeqCst) {
            // Check for config reload every 2 seconds
            if last_config_check.elapsed() >= Duration::from_secs(2) {
                if let Ok(reloaded) = self.check_config_reload() {
                    if reloaded {
                        tracing::info!("Configuration hot-reloaded successfully");
                    }
                }
                last_config_check = Instant::now();
            }

            let effective_tick_rate =
                Duration::from_secs_f64(tick_rate.as_secs_f64() / self.time_scale);

            tui.terminal.draw(|f| {
                self.draw(f);
            })?;

            self.frame_count += 1;
            if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
                self.update_hardware_metrics();
            }

            if self.replay_mode {
                while let Some(evt) = self.replay_queue.front() {
                    if evt.tick <= self.world.tick {
                        if let Some(evt) = self.replay_queue.pop_front() {
                            match evt.event {
                                Event::Key(key) if key.kind == KeyEventKind::Press => {
                                    self.handle_key(key);
                                }
                                Event::Mouse(mouse) => {
                                    self.handle_mouse(mouse);
                                }
                                _ => {}
                            }
                        }
                    } else {
                        break;
                    }
                }
            } else {
                // Use 1ms poll interval to prevent busy-waiting while remaining responsive
                while event::poll(Duration::from_millis(1))? {
                    let evt = event::read()?;
                    if !self.screensaver {
                        self.input_log.push(crate::app::state::InputEvent {
                            tick: self.world.tick,
                            event: evt.clone(),
                        });
                    }
                    match evt {
                        Event::Key(key) if key.kind == KeyEventKind::Press => {
                            self.handle_key(key);
                        }
                        Event::Mouse(mouse) => {
                            self.handle_mouse(mouse);
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= effective_tick_rate {
                if !self.paused {
                    self.update_world()?;
                }

                if self.show_archeology
                    && self.auto_play_history
                    && self.frame_count.is_multiple_of(10)
                {
                    if self.archeology_index + 1 < self.archeology_snapshots.len() {
                        self.archeology_index += 1;
                    } else {
                        self.auto_play_history = false;
                    }
                }

                last_tick = Instant::now();
            }
        }

        // Perform graceful shutdown
        if shutdown.load(Ordering::SeqCst) {
            tracing::info!("Saving state before exit...");
            self.save_state()?;
            if !self.input_log.is_empty() {
                let _ = self.save_recording();
            }
        }

        Ok(())
    }

    fn update_hardware_metrics(&mut self) {
        self.fps = self.frame_count as f64;
        self.frame_count = 0;

        if !self.config.world.deterministic {
            self.sys.refresh_cpu();
            self.sys.refresh_memory();
            self.sys.refresh_processes();

            let pid = Pid::from_u32(std::process::id());
            if let Some(process) = self.sys.process(pid) {
                // memory() returns bytes, convert to MB
                self.env.app_memory_usage_mb = process.memory() as f32 / 1024.0 / 1024.0;
            }

            let cpu_usage = self.sys.global_cpu_info().cpu_usage();
            self.env.cpu_usage = cpu_usage;
            self.env.ram_usage_percent =
                (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;
        }

        environment_system::update_era(
            &mut self.env,
            self.world.tick,
            &self.world.pop_stats,
            &self.config,
        );

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
        environment_system::update_events(&mut self.env, &self.config);

        let current_cpu = self.env.cpu_usage;
        self.cpu_history.pop_front();
        self.cpu_history.push_back(current_cpu as u64);

        self.pop_history.pop_front();
        self.pop_history
            .push_back(self.world.get_population_count() as u64);

        self.o2_history.pop_front();
        self.o2_history.push_back(self.env.oxygen_level as u64);

        self.last_fps_update = Instant::now();
    }

    fn update_world(&mut self) -> Result<()> {
        let events = self.world.update(&mut self.env)?;
        self.latest_snapshot = Some(self.world.create_snapshot(self.selected_entity));

        if let Some(net) = &self.network {
            self.network_state = net.get_state();

            for msg in net.pop_pending_limited(5) {
                use crate::model::infra::network::NetMessage;
                match msg {
                    NetMessage::MigrateEntity {
                        migration_id,
                        dna,
                        energy,
                        generation,
                        fingerprint,
                        checksum,
                        ..
                    } => {
                        let _ = self.world.import_migrant(
                            dna,
                            energy,
                            generation,
                            &fingerprint,
                            &checksum,
                        );
                        self.event_log.push_back((
                            "MIGRANT ARRIVED: An entity has entered this universe!".to_string(),
                            Color::Cyan,
                        ));
                        net.send(&NetMessage::MigrateAck { migration_id });
                    }
                    NetMessage::MigrateAck { migration_id } => {
                        let mut handles_to_despawn = Vec::new();
                        for (handle, met) in self
                            .world
                            .ecs
                            .query::<&crate::model::state::Metabolism>()
                            .iter()
                        {
                            if met.migration_id == Some(migration_id) {
                                handles_to_despawn.push(handle);
                            }
                        }
                        for h in handles_to_despawn {
                            let _ = self.world.ecs.despawn(h);
                        }

                        self.event_log.push_back((
                            "MIGRATION CONFIRMED: Entity successfully reached another universe."
                                .to_string(),
                            Color::Green,
                        ));
                    }
                    NetMessage::Relief {
                        lineage_id, amount, ..
                    } => {
                        self.world.apply_relief(lineage_id, amount);
                        self.event_log.push_back((
                            format!(
                                "RELIEF RECEIVED: {:.1} energy granted to kin of this universe!",
                                amount
                            ),
                            Color::Yellow,
                        ));
                    }
                    NetMessage::GlobalEvent { event_type, .. } => {
                        match event_type.as_str() {
                            "SolarFlare" => self.env.radiation_timer = 500,
                            "DeepFreeze" => self.env.ice_age_timer = 1000,
                            _ => {}
                        }
                        self.event_log.push_back((
                            format!("GLOBAL EVENT: {} detected across the Hive!", event_type),
                            Color::Red,
                        ));
                    }
                    _ => {}
                }
            }

            let mut migrants = Vec::new();
            let width = self.world.width as f64;
            let height = self.world.height as f64;
            let config_fingerprint = self.world.config.fingerprint();

            for (_handle, (identity, phys, met, intel)) in self
                .world
                .ecs
                .query::<(
                    &primordium_data::Identity,
                    &primordium_data::Physics,
                    &mut primordium_data::Metabolism,
                    &primordium_data::Intel,
                )>()
                .iter()
            {
                if met.is_in_transit {
                    continue;
                }

                let leaving = phys.x < 1.0
                    || phys.x > (width - 2.0)
                    || phys.y < 1.0
                    || phys.y > (height - 2.0);

                if leaving {
                    use crate::model::infra::network::NetMessage;
                    use sha2::{Digest, Sha256};
                    let dna = intel.genotype.to_hex();
                    let energy = met.energy as f32;
                    let generation = met.generation;

                    let mut hasher = Sha256::new();
                    hasher.update(dna.as_bytes());
                    hasher.update(energy.to_be_bytes());
                    hasher.update(generation.to_be_bytes());
                    let checksum = hex::encode(hasher.finalize());

                    let migration_id = Uuid::new_v4();
                    met.is_in_transit = true;
                    met.migration_id = Some(migration_id);

                    migrants.push(NetMessage::MigrateEntity {
                        migration_id,
                        dna,
                        energy,
                        generation,
                        species_name: crate::model::lifecycle::get_name_components(
                            &identity.id,
                            met,
                        ),
                        fingerprint: config_fingerprint.clone(),
                        checksum,
                    });
                }
            }

            for msg in migrants {
                net.send(&msg);
                self.event_log.push_back((
                    "MIGRANT DEPARTED: An entity is in transit to another universe...".to_string(),
                    Color::Magenta,
                ));
            }

            if self.world.tick.is_multiple_of(300) {
                net.announce(self.world.get_population_count());
            }
        }

        for ev in events {
            let _ = self.world.logger.log_event(ev.clone());
            let (msg, color) = ev.to_ui_message();
            self.event_log.push_back((msg, color));
            if self.event_log.len() > 15 {
                self.event_log.pop_front();
            }
        }
        Ok(())
    }
}

trait LiveEventExt {
    fn to_ui_message(&self) -> (String, ratatui::style::Color);
}

impl LiveEventExt for LiveEvent {
    fn to_ui_message(&self) -> (String, ratatui::style::Color) {
        use ratatui::style::Color;
        match self {
            LiveEvent::Birth { gen, id, .. } => (
                format!("Gen {} #{} born", gen, &id.to_string()[..4]),
                Color::Cyan,
            ),
            LiveEvent::Death { age, id, cause, .. } => {
                let msg = if cause.is_empty() {
                    format!("#{} died at age {}", &id.to_string()[..4], age)
                } else {
                    format!(
                        "#{} killed by {} at age {}",
                        &id.to_string()[..4],
                        cause,
                        age
                    )
                };
                (msg, Color::Red)
            }
            LiveEvent::ClimateShift { from: _, to, .. } => {
                let effect = match to.as_str() {
                    "Temperate" => "☀️ Temperate - ×1.0",
                    "Warm" => "🔥 Warm - ×1.5",
                    "Hot" => "🌋 Hot - ×2.0",
                    "Scorching" => "☀️ SCORCHING - ×3.0",
                    _ => to.as_str(),
                };
                (format!("Climate: {}", effect), Color::Yellow)
            }
            LiveEvent::Extinction { tick, .. } => {
                (format!("Extinction at tick {}", tick), Color::Magenta)
            }
            LiveEvent::EcoAlert { message, .. } => (format!("⚠️ {}", message), Color::Yellow),
            LiveEvent::Metamorphosis { name, .. } => {
                (format!("✨ {} has metamorphosed!", name), Color::Yellow)
            }
            LiveEvent::TribalSplit { id, .. } => (
                format!("⚔️ #{} split into a new tribe!", &id.to_string()[..4]),
                Color::Magenta,
            ),
            LiveEvent::Snapshot { tick, .. } => (
                format!("🏛️ Snapshot saved at tick {}", tick),
                Color::DarkGray,
            ),
            LiveEvent::Narration { text, .. } => (format!("📜 {}", text), Color::Green),
        }
    }
}
