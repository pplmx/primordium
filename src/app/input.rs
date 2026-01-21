use crate::app::state::App;
use crate::model::brain::Brain;
use crate::ui::renderer::WorldWidget;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::style::Color;
use std::fs;

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Char(' ') => {
                // Space advances onboarding, otherwise toggles pause
                if self.onboarding_step.is_some() {
                    self.advance_onboarding();
                } else {
                    self.paused = !self.paused;
                }
            }
            KeyCode::Char('b') => self.show_brain = !self.show_brain,
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.onboarding_step = None; // Close onboarding if help opened
                }
            }
            // Help tab navigation (only when help is open)
            KeyCode::Char('1') if self.show_help => self.help_tab = 0,
            KeyCode::Char('2') if self.show_help => self.help_tab = 1,
            KeyCode::Char('3') if self.show_help => self.help_tab = 2,
            KeyCode::Char('4') if self.show_help => self.help_tab = 3,
            // Onboarding navigation (Enter key)
            KeyCode::Enter if self.onboarding_step.is_some() => {
                self.advance_onboarding();
            }
            KeyCode::Esc if self.onboarding_step.is_some() => {
                // Skip onboarding
                let _ = fs::write(".primordium_onboarded", "1");
                self.onboarding_step = None;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.time_scale = (self.time_scale + 0.5).min(4.0)
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.time_scale = (self.time_scale - 0.5).max(0.5)
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                for entity in &mut self.world.entities {
                    entity.brain.mutate_with_config(&self.config.evolution);
                }
                self.event_log
                    .push_back(("GENETIC SURGE!".to_string(), Color::Red));
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Some(id) = self.selected_entity {
                    if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                        let dna = entity.brain.to_hex();
                        let _ = fs::write("exported_dna.txt", &dna);
                        self.event_log.push_back((
                            "DNA exported to exported_dna.txt".to_string(),
                            Color::Cyan,
                        ));
                    }
                }
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                if let Ok(dna) = fs::read_to_string("dna_infuse.txt") {
                    if let Ok(brain) = Brain::from_hex(dna.trim()) {
                        let mut e = crate::model::entity::Entity::new(50.0, 25.0, self.world.tick);
                        e.brain = brain;
                        self.world.entities.push(e);
                        self.event_log.push_back((
                            "AVATAR INFUSED from dna_infuse.txt".to_string(),
                            Color::Green,
                        ));
                        let _ = fs::remove_file("dna_infuse.txt");
                    }
                }
            }
            _ => {}
        }
    }

    fn advance_onboarding(&mut self) {
        if let Some(ref mut step) = self.onboarding_step {
            if *step >= 2 {
                // Complete onboarding
                let _ = fs::write(".primordium_onboarded", "1");
                self.onboarding_step = None;
            } else {
                *step += 1;
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some((wx, wy)) = WorldWidget::screen_to_world(
                    mouse.column,
                    mouse.row,
                    self.last_world_rect,
                    self.screensaver,
                ) {
                    let indices = self.world.spatial_hash.query(wx, wy, 2.0);
                    let mut min_dist = f64::MAX;
                    let mut closest_id = None;
                    for idx in indices {
                        if idx >= self.world.entities.len() {
                            continue;
                        }
                        let entity = &self.world.entities[idx];
                        let dx = entity.x - wx;
                        let dy = entity.y - wy;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < min_dist {
                            min_dist = dist;
                            closest_id = Some(entity.id);
                        }
                    }
                    if let Some(id) = closest_id {
                        self.selected_entity = Some(id);
                        self.show_brain = true;
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Right) => {
                if let Some((wx, wy)) = WorldWidget::screen_to_world(
                    mouse.column,
                    mouse.row,
                    self.last_world_rect,
                    self.screensaver,
                ) {
                    use crate::model::food::Food;
                    self.world.food.push(Food::new(wx as u16, wy as u16));
                    self.event_log
                        .push_back(("Divine Food Injected".to_string(), Color::Green));
                }
            }
            _ => {}
        }
    }
}
