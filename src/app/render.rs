use crate::app::state::App;
use primordium_tui::renderer::WorldWidget;
use primordium_tui::views::*;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use ratatui::Frame;

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        let snapshot = match &self.latest_snapshot {
            Some(s) => s,
            None => return,
        };

        let bg_color = self.get_climate_bg_color();
        let main_block = Block::default().style(Style::default().bg(bg_color));
        f.render_widget(main_block, f.area());

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                if self.show_brain
                    || self.show_ancestry
                    || self.show_archeology
                    || self.view_mode >= 6
                {
                    Constraint::Length(45)
                } else {
                    Constraint::Length(0)
                },
            ])
            .split(f.area());

        self.last_sidebar_rect = main_layout[1];

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),
                Constraint::Length(4),
                Constraint::Min(0),
                Constraint::Length(7),
            ])
            .split(main_layout[0]);

        self.last_world_rect = left_layout[2];

        if self.screensaver || self.cinematic_mode {
            let world_widget = WorldWidget::new(snapshot, true, self.view_mode);
            f.render_widget(world_widget, f.area());

            if self.cinematic_mode {
                f.render_widget(
                    CinematicOverlayWidget {
                        tick: snapshot.tick,
                        carbon_level: snapshot.stats.carbon_level,
                    },
                    f.area(),
                );
            }
        } else {
            f.render_widget(
                StatusWidget {
                    snapshot,
                    cpu_usage: self.env.cpu_usage as f64,
                    ram_usage_percent: self.env.ram_usage_percent,
                    app_memory_usage_mb: self.env.app_memory_usage_mb as f64,
                    current_era: self.env.current_era,
                    oxygen_level: self.env.oxygen_level,
                    view_mode: self.view_mode,
                    peer_count: self.network_state.peers.len(),
                    migrations_received: self.network_state.migrations_received as u64,
                    migrations_sent: self.network_state.migrations_sent as u64,
                    is_online: self.network_state.client_id.is_some(),
                    resource_icon: self.env.resource_state().icon().to_string(),
                },
                left_layout[0],
            );

            let pop_data: Vec<u64> = self.pop_history.iter().cloned().collect();
            let cpu_data: Vec<u64> = self.cpu_history.iter().cloned().collect();
            f.render_widget(
                SparklinesWidget {
                    pop_data: &pop_data,
                    cpu_data: &cpu_data,
                    current_era: self.env.current_era,
                },
                left_layout[1],
            );

            let world_widget = WorldWidget::new(snapshot, false, self.view_mode);
            f.render_widget(world_widget, left_layout[2]);

            let events: Vec<(String, Color)> = self.event_log.iter().cloned().collect();
            f.render_widget(ChronicleWidget { events: &events }, left_layout[3]);
        }

        let sidebar_area = main_layout[1];
        if self.show_ancestry {
            f.render_widget(AncestryWidget { snapshot }, sidebar_area);
        } else if self.show_archeology {
            f.render_widget(
                ArcheologyWidget {
                    snapshots: &self.archeology_snapshots,
                    index: self.archeology_index,
                    fossils: &self.world.fossil_registry.fossils,
                    selected_fossil_index: self.selected_fossil_index,
                },
                sidebar_area,
            );
        } else if self.show_brain {
            f.render_widget(
                BrainWidget {
                    snapshot,
                    selected_entity: self.selected_entity,
                },
                sidebar_area,
            );
        } else if self.view_mode == 6 {
            f.render_widget(
                MarketWidget {
                    trade_offers: &self.network_state.trade_offers,
                },
                sidebar_area,
            );
        } else if self.view_mode == 7 {
            f.render_widget(
                ResearchWidget {
                    snapshot,
                    selected_entity: self.selected_entity,
                },
                sidebar_area,
            );
        } else if self.view_mode == 8 {
            f.render_widget(
                CivilizationWidget {
                    registry: &self.world.lineage_registry,
                },
                sidebar_area,
            );
        }

        if self.show_help {
            f.render_widget(
                HelpWidget {
                    help_tab: self.help_tab,
                },
                f.area(),
            );
        }

        if let Some(_step) = self.onboarding_step {
            self.render_onboarding(f);
        }

        if self.show_legend {
            f.render_widget(LegendWidget, f.area());
        }
    }

    fn get_climate_bg_color(&self) -> Color {
        let carbon = self.env.carbon_level;
        let temp = self.env.cpu_usage as f64 / 100.0;

        if carbon > 800.0 || temp > 0.8 {
            Color::Rgb(30, 5, 5)
        } else if carbon > 500.0 {
            Color::Rgb(20, 15, 10)
        } else if self.env.oxygen_level < 15.0 {
            Color::Rgb(10, 10, 20)
        } else {
            Color::Rgb(5, 10, 5)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::config::AppConfig;
    use crate::model::environment::Environment;
    use crate::model::world::World;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use std::collections::VecDeque;
    use std::time::Instant;
    use sysinfo::System;

    fn create_test_app() -> App {
        let config = AppConfig::default();
        let world = World::new(0, config.clone()).unwrap();
        let mut app = App {
            running: true,
            paused: false,
            tick_count: 0,
            world,
            config: config.clone(),
            config_path: "config.toml".to_string(),
            config_last_modified: None,
            fps: 60.0,
            frame_count: 0,
            last_fps_update: Instant::now(),
            time_scale: 1.0,
            sys: System::new_all(),
            env: Environment::default(),
            cpu_history: VecDeque::new(),
            pop_history: VecDeque::new(),
            o2_history: VecDeque::new(),
            show_brain: false,
            selected_entity: None,
            focused_gene: None,
            brush_type: primordium_data::TerrainType::Plains,
            social_brush: 0,
            is_social_brush: false,
            show_ancestry: false,
            last_climate: None,
            last_anchor_time: Instant::now(),
            anchor_interval: std::time::Duration::from_secs(3600),
            is_anchoring: false,
            screensaver: false,
            cinematic_mode: false,
            show_help: false,
            show_legend: true,
            help_tab: 0,
            show_archeology: false,
            auto_play_history: false,
            archeology_snapshots: Vec::new(),
            archeology_index: 0,
            selected_fossil_index: 0,
            onboarding_step: None,
            view_mode: 0,
            last_world_rect: ratatui::layout::Rect::default(),
            last_sidebar_rect: ratatui::layout::Rect::default(),
            gene_editor_offset: 0,
            event_log: VecDeque::new(),
            network_state: primordium_net::NetworkState::default(),
            latest_snapshot: None,
            network: None,
            hof_query_rx: None,
            cached_hall_of_fame: Vec::new(),
            input_log: Vec::new(),
            replay_queue: VecDeque::new(),
            replay_mode: false,
        };
        app.latest_snapshot = Some(app.world.create_snapshot(None));
        app
    }

    #[tokio::test]
    async fn test_app_draw_no_panic() {
        let mut app = create_test_app();
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                app.draw(f);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert!(buffer.area.width > 0);
        assert!(buffer.area.height > 0);
    }

    #[tokio::test]
    async fn test_draw_help_overlay() {
        let mut app = create_test_app();
        app.show_help = true;
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                app.draw(f);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let help_found = buffer.content().iter().any(|c| {
            c.symbol().contains('H')
                || c.symbol().contains('e')
                || c.symbol().contains('l')
                || c.symbol().contains('p')
        });
        assert!(help_found);
    }

    #[tokio::test]
    async fn test_draw_sidebar_brain() {
        let mut app = create_test_app();
        app.show_brain = true;
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                app.draw(f);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let found = buffer.content().iter().any(|c| {
            c.symbol().contains('H')
                || c.symbol().contains('a')
                || c.symbol().contains('l')
                || c.symbol().contains('l')
        });
        assert!(found);
    }
}
