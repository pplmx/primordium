pub mod genetic_edit;
pub mod normal;
pub mod terrain_edit;

use crate::app::state::App;
use crossterm::event::KeyEvent;

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        self.handle_normal_key(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::config::AppConfig;
    use crate::model::environment::Environment;
    use crate::model::world::World;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::collections::VecDeque;
    use std::time::Instant;
    use sysinfo::System;

    fn create_test_app() -> App {
        let config = AppConfig::default();
        let world = World::new(0, config.clone()).expect("World creation should not fail in tests");

        App {
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
        }
    }

    #[test]
    fn test_handle_key_quit() {
        let mut app = create_test_app();
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
        assert!(!app.running);
    }

    #[test]
    fn test_handle_key_pause() {
        let mut app = create_test_app();
        assert!(!app.paused);
        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert!(app.paused);
        app.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::empty()));
        assert!(!app.paused);
    }

    #[test]
    fn test_handle_key_ui_toggles() {
        let mut app = create_test_app();

        app.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()));
        assert!(app.show_brain);

        app.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        assert!(app.show_ancestry);

        app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty()));
        assert!(app.show_help);
    }

    #[test]
    fn test_view_mode_switching() {
        let mut app = create_test_app();
        app.handle_key(KeyEvent::new(KeyCode::Char('2'), KeyModifiers::empty()));
        assert_eq!(app.view_mode, 1);
        app.handle_key(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty()));
        assert_eq!(app.view_mode, 2);
    }

    #[test]
    fn test_time_scale_adjustment() {
        let mut app = create_test_app();
        let initial = app.time_scale;
        app.handle_key(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::empty()));
        assert!(app.time_scale > initial);
        app.handle_key(KeyEvent::new(KeyCode::Char('-'), KeyModifiers::empty()));
        assert_eq!(app.time_scale, initial);
    }
}
