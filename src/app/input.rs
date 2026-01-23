use crate::app::state::App;
use crate::model::state::terrain::TerrainType;
use crate::model::systems::intel;
use crate::ui::renderer::WorldWidget;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use rand::Rng;
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
                    intel::mutate_genotype(&mut entity.intel.genotype, &self.config.evolution);
                    // Sync phenotype
                    entity.physics.sensing_range = entity.intel.genotype.sensing_range;
                    entity.physics.max_speed = entity.intel.genotype.max_speed;
                    entity.metabolism.max_energy = entity.intel.genotype.max_energy;
                }
                self.event_log
                    .push_back(("GENETIC SURGE!".to_string(), Color::Red));
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if let Some(id) = self.selected_entity {
                    if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                        let dna = entity.intel.genotype.to_hex();
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
                    if let Ok(genotype) =
                        crate::model::state::entity::Genotype::from_hex(dna.trim())
                    {
                        let mut e =
                            crate::model::state::entity::Entity::new(50.0, 25.0, self.world.tick);
                        e.intel.genotype = genotype;
                        // Sync phenotype
                        e.physics.sensing_range = e.intel.genotype.sensing_range;
                        e.physics.max_speed = e.intel.genotype.max_speed;
                        e.metabolism.max_energy = e.intel.genotype.max_energy;
                        e.metabolism.lineage_id = e.intel.genotype.lineage_id;

                        self.world.entities.push(e);
                        self.event_log.push_back((
                            "AVATAR INFUSED from dna_infuse.txt".to_string(),
                            Color::Green,
                        ));
                        let _ = fs::remove_file("dna_infuse.txt");
                    }
                }
            }
            // BRUSH SELECTION
            KeyCode::Char('!') => self.brush_type = TerrainType::Plains,
            KeyCode::Char('@') => self.brush_type = TerrainType::Mountain,
            KeyCode::Char('#') => self.brush_type = TerrainType::River,
            KeyCode::Char('$') => self.brush_type = TerrainType::Oasis,
            KeyCode::Char('%') => self.brush_type = TerrainType::Wall,
            KeyCode::Char('^') => self.brush_type = TerrainType::Barren,

            // DIVINE INTERVENTION (Selected Entity)
            KeyCode::Char('m') => {
                if let Some(id) = self.selected_entity {
                    if let Some(entity) = self.world.entities.iter_mut().find(|e| e.id == id) {
                        intel::mutate_genotype(&mut entity.intel.genotype, &self.config.evolution);
                        // Sync phenotype
                        entity.physics.sensing_range = entity.intel.genotype.sensing_range;
                        entity.physics.max_speed = entity.intel.genotype.max_speed;
                        entity.metabolism.max_energy = entity.intel.genotype.max_energy;
                        self.event_log
                            .push_back(("Divine Mutation".to_string(), Color::Yellow));
                    }
                }
            }
            KeyCode::Char('k') => {
                if let Some(id) = self.selected_entity {
                    self.world.entities.retain(|e| e.id != id);
                    self.selected_entity = None;
                    self.event_log
                        .push_back(("Divine Smite".to_string(), Color::Red));
                }
            }
            KeyCode::Char('p') => {
                if let Some(id) = self.selected_entity {
                    if let Some(entity) = self.world.entities.iter_mut().find(|e| e.id == id) {
                        use crate::model::state::entity::Genotype;
                        entity.intel.genotype = Genotype::new_random();
                        // Sync phenotype
                        entity.physics.sensing_range = entity.intel.genotype.sensing_range;
                        entity.physics.max_speed = entity.intel.genotype.max_speed;
                        entity.metabolism.max_energy = entity.intel.genotype.max_energy;
                        self.event_log
                            .push_back(("Divine Reincarnation".to_string(), Color::Magenta));
                    }
                }
            }
            // GOD MODE COMMANDS
            KeyCode::Char('K') => {
                if self.env.god_climate_override.is_some() {
                    self.env.god_climate_override = None;
                    self.event_log
                        .push_back(("God: Climate Restored".to_string(), Color::Cyan));
                } else {
                    self.env.god_climate_override =
                        Some(crate::model::state::environment::ClimateState::Scorching);
                    self.event_log
                        .push_back(("GOD MODE: HEAT WAVE INDUCED".to_string(), Color::Red));
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                let kill_count = (self.world.entities.len() as f32 * 0.9) as usize;
                self.world
                    .entities
                    .truncate(self.world.entities.len().saturating_sub(kill_count));
                self.event_log.push_back((
                    "GOD MODE: MASS EXTINCTION TRIGGERED".to_string(),
                    Color::Magenta,
                ));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                use crate::model::state::food::Food;
                let mut rng = rand::thread_rng();
                for _ in 0..100 {
                    let fx = rng.gen_range(1..self.world.width - 1);
                    let fy = rng.gen_range(1..self.world.height - 1);
                    let n_type = rng.gen_range(0.0..1.0);
                    self.world.food.push(Food::new(fx, fy, n_type));
                }
                self.event_log
                    .push_back(("GOD MODE: RESOURCE BOOM!".to_string(), Color::Green));
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
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                if let Some((wx, wy)) = WorldWidget::screen_to_world(
                    mouse.column,
                    mouse.row,
                    self.last_world_rect,
                    self.screensaver,
                ) {
                    // Try to paint terrain if no entity selected or if dragging
                    let painted = if matches!(mouse.kind, MouseEventKind::Drag(MouseButton::Left)) {
                        true
                    } else {
                        // Check for entity selection on Down
                        let indices = self.world.spatial_hash.query(wx, wy, 2.0);
                        let mut closest_id = None;
                        for idx in indices {
                            if idx < self.world.entities.len() {
                                closest_id = Some(self.world.entities[idx].id);
                                break;
                            }
                        }
                        if let Some(id) = closest_id {
                            self.selected_entity = Some(id);
                            self.show_brain = true;
                            false // Selection, not painting
                        } else {
                            true // Empty space, paint!
                        }
                    };

                    if painted {
                        self.world
                            .terrain
                            .set_cell_type(wx as u16, wy as u16, self.brush_type);
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
                    use crate::model::state::food::Food;
                    let n_type = rand::thread_rng().gen_range(0.0..1.0);
                    self.world
                        .food
                        .push(Food::new(wx as u16, wy as u16, n_type));
                    self.event_log
                        .push_back(("Divine Food Injected".to_string(), Color::Green));
                }
            }
            _ => {}
        }
    }
}
