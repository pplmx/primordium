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
            KeyCode::Char('a') => self.show_ancestry = !self.show_ancestry,
            KeyCode::Char('y') => {
                self.show_archeology = !self.show_archeology;
                if self.show_archeology {
                    if let Ok(snaps) = self.world.logger.get_snapshots() {
                        self.archeology_snapshots = snaps;
                        self.archeology_index = self.archeology_snapshots.len().saturating_sub(1);
                    }
                }
            }
            KeyCode::Char('[') if self.show_archeology => {
                self.archeology_index = self.archeology_index.saturating_sub(1);
            }
            KeyCode::Char(']') if self.show_archeology => {
                if self.archeology_index + 1 < self.archeology_snapshots.len() {
                    self.archeology_index += 1;
                }
            }
            KeyCode::Up if self.show_archeology => {
                self.selected_fossil_index = self.selected_fossil_index.saturating_sub(1);
            }
            KeyCode::Down if self.show_archeology => {
                if self.selected_fossil_index + 1 < self.world.fossil_registry.fossils.len() {
                    self.selected_fossil_index += 1;
                }
            }
            KeyCode::Char('g') | KeyCode::Char('G') if self.show_archeology => {
                if let Some(fossil) = self
                    .world
                    .fossil_registry
                    .fossils
                    .get(self.selected_fossil_index)
                {
                    let mut e =
                        crate::model::state::entity::Entity::new(50.0, 25.0, self.world.tick);
                    e.intel.genotype = fossil.genotype.clone();
                    // Sync phenotype
                    e.physics.sensing_range = e.intel.genotype.sensing_range;
                    e.physics.max_speed = e.intel.genotype.max_speed;
                    e.metabolism.max_energy = e.intel.genotype.max_energy;
                    e.metabolism.lineage_id = e.intel.genotype.lineage_id;
                    e.metabolism.energy = e.metabolism.max_energy; // Spawn with full energy

                    self.world.entities.push(e);
                    self.event_log.push_back((
                        format!("RESURRECTED: {} cloned into current world", fossil.name),
                        Color::Magenta,
                    ));
                }
            }
            KeyCode::Char('A') => {
                if let Ok(tree) = self.world.logger.get_ancestry_tree(&self.world.entities) {
                    let dot = tree.to_dot();
                    let _ = fs::write("logs/tree.dot", dot);
                    self.event_log.push_back((
                        "Ancestry Tree exported to logs/tree.dot".to_string(),
                        Color::Green,
                    ));
                }
            }
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.onboarding_step = None; // Close onboarding if help opened
                }
            }
            // View Mode Switching (only if help and onboarding closed)
            KeyCode::Char('1') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 0;
                self.event_log
                    .push_back(("View: NORMAL".to_string(), Color::White));
            }
            KeyCode::Char('2') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 1;
                self.event_log
                    .push_back(("View: FERTILITY HEATMAP".to_string(), Color::Green));
            }
            KeyCode::Char('3') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 2;
                self.event_log
                    .push_back(("View: SOCIAL ZONES".to_string(), Color::Cyan));
            }
            KeyCode::Char('4') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 3;
                self.event_log
                    .push_back(("View: RANK HEATMAP".to_string(), Color::Magenta));
            }
            KeyCode::Char('5') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 4;
                self.event_log
                    .push_back(("View: VOCAL PROPAGATION".to_string(), Color::Yellow));
            }
            KeyCode::Char('6') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 5;
                self.event_log
                    .push_back(("View: MULTIVERSE MARKET".to_string(), Color::Cyan));
            }
            KeyCode::Char('7') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 6;
                self.event_log
                    .push_back(("View: NEURAL RESEARCH".to_string(), Color::Magenta));
            }
            KeyCode::Char('0') if self.view_mode == 6 => {
                if let Some(id) = self.selected_entity {
                    self.world.clear_research_deltas(id);
                    self.event_log
                        .push_back(("Research deltas cleared".to_string(), Color::Cyan));
                }
            }

            // Help tab navigation (only when help is open)
            KeyCode::Char('1') if self.show_help => self.help_tab = 0,
            KeyCode::Char('2') if self.show_help => self.help_tab = 1,
            KeyCode::Char('3') if self.show_help => self.help_tab = 2,
            KeyCode::Char('4') if self.show_help => self.help_tab = 3,
            KeyCode::Char('5') if self.show_help => self.help_tab = 4,
            KeyCode::Char('6') if self.show_help => self.help_tab = 5,

            // Market Actions (only in Market View)
            KeyCode::Char(c) if self.view_mode == 5 && c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if let Some(offer) = self.network_state.trade_offers.get(idx).cloned() {
                    // Logic to accept trade:
                    // 1. Deduct our resources (request from offer)
                    // 2. Add offered resources

                    self.world.apply_trade(
                        &mut self.env,
                        offer.request_resource.clone(),
                        offer.request_amount,
                        false,
                    );

                    self.world.apply_trade(
                        &mut self.env,
                        offer.offer_resource.clone(),
                        offer.offer_amount,
                        true,
                    );

                    self.event_log.push_back((
                        format!(
                            "TRADE ACCEPTED: Exchanged {:?} for {:?}",
                            offer.request_resource, offer.offer_resource
                        ),
                        Color::Green,
                    ));
                    // Remove the offer locally
                    self.network_state.trade_offers.remove(idx);
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') if self.view_mode == 5 => {
                // Propose a random trade for testing
                let mut rng = rand::thread_rng();
                use crate::model::infra::network::{TradeProposal, TradeResource};
                let offer_res = match rng.gen_range(0..4) {
                    0 => TradeResource::Energy,
                    1 => TradeResource::Oxygen,
                    2 => TradeResource::SoilFertility,
                    _ => TradeResource::Biomass,
                };
                let req_res = match rng.gen_range(0..4) {
                    0 => TradeResource::Energy,
                    1 => TradeResource::Oxygen,
                    2 => TradeResource::SoilFertility,
                    _ => TradeResource::Biomass,
                };
                let proposal = TradeProposal {
                    id: uuid::Uuid::new_v4(),
                    sender_id: self
                        .network_state
                        .client_id
                        .unwrap_or_else(uuid::Uuid::new_v4),
                    offer_resource: offer_res,
                    offer_amount: 100.0,
                    request_resource: req_res,
                    request_amount: 100.0,
                };
                self.network_state.trade_offers.push(proposal);
                self.event_log
                    .push_back(("Trade offer proposed".to_string(), Color::Yellow));
            }
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
                if self.show_brain && self.focused_gene.is_some() {
                    if let (Some(id), Some(gene)) = (self.selected_entity, self.focused_gene) {
                        let delta = match gene {
                            crate::app::state::GeneType::Trophic => 0.05,
                            crate::app::state::GeneType::Sensing => 0.5,
                            crate::app::state::GeneType::Speed => 0.1,
                            crate::app::state::GeneType::MaxEnergy => 10.0,
                            crate::app::state::GeneType::ReproInvest => 0.05,
                            crate::app::state::GeneType::Maturity => 0.1,
                        };
                        self.world.apply_genetic_edit(id, gene, delta);
                    }
                } else {
                    self.time_scale = (self.time_scale + 0.5).min(4.0);
                }
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                if self.show_brain && self.focused_gene.is_some() {
                    if let (Some(id), Some(gene)) = (self.selected_entity, self.focused_gene) {
                        let delta = match gene {
                            crate::app::state::GeneType::Trophic => -0.05,
                            crate::app::state::GeneType::Sensing => -0.5,
                            crate::app::state::GeneType::Speed => -0.1,
                            crate::app::state::GeneType::MaxEnergy => -10.0,
                            crate::app::state::GeneType::ReproInvest => -0.05,
                            crate::app::state::GeneType::Maturity => -0.1,
                        };
                        self.world.apply_genetic_edit(id, gene, delta);
                    }
                } else {
                    self.time_scale = (self.time_scale - 0.5).max(0.5);
                }
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                let pop = self.world.entities.len();
                for entity in &mut self.world.entities {
                    intel::mutate_genotype(&mut entity.intel.genotype, &self.config, pop);
                    // Sync phenotype
                    entity.physics.sensing_range = entity.intel.genotype.sensing_range;
                    entity.physics.max_speed = entity.intel.genotype.max_speed;
                    entity.metabolism.max_energy = entity.intel.genotype.max_energy;
                }
                self.event_log
                    .push_back(("GENETIC SURGE!".to_string(), Color::Red));
            }
            KeyCode::Char('c') => {
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
            KeyCode::Char('C') => {
                if let Some(id) = self.selected_entity {
                    if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                        if let Ok(json) = serde_json::to_string_pretty(&entity.intel.genotype.brain)
                        {
                            let filename = format!("logs/brain_{}.json", id);
                            let _ = fs::write(&filename, json);
                            self.event_log.push_back((
                                format!("Brain JSON exported to {}", filename),
                                Color::Magenta,
                            ));
                        }
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
            KeyCode::Char('j') | KeyCode::Char('J') => {
                self.is_social_brush = !self.is_social_brush;
                self.event_log.push_back((
                    format!(
                        "Brush mode: {}",
                        if self.is_social_brush {
                            "Social (Peace/War)"
                        } else {
                            "Terrain"
                        }
                    ),
                    Color::Cyan,
                ));
            }
            KeyCode::Char('!') => {
                if self.is_social_brush {
                    self.social_brush = 0;
                } else {
                    self.brush_type = TerrainType::Plains;
                }
            }
            KeyCode::Char('@') => {
                if self.is_social_brush {
                    self.social_brush = 1;
                } else {
                    self.brush_type = TerrainType::Mountain;
                }
            }
            KeyCode::Char('#') => {
                if self.is_social_brush {
                    self.social_brush = 2;
                } else {
                    self.brush_type = TerrainType::River;
                }
            }
            KeyCode::Char('$') => self.brush_type = TerrainType::Oasis,
            KeyCode::Char('%') => self.brush_type = TerrainType::Wall,
            KeyCode::Char('^') => self.brush_type = TerrainType::Barren,

            // DIVINE INTERVENTION (Selected Entity)
            KeyCode::Char('m') => {
                if let Some(id) = self.selected_entity {
                    let pop = self.world.entities.len();
                    if let Some(entity) = self.world.entities.iter_mut().find(|e| e.id == id) {
                        intel::mutate_genotype(&mut entity.intel.genotype, &self.config, pop);
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
                // Phase 39.5:上帝模式硬重启 - 同时清理大气碳
                self.env.carbon_level = 300.0;
                self.event_log.push_back((
                    "GOD MODE: MASS EXTINCTION & CARBON SCRUB".to_string(),
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
            KeyCode::Char('w') | KeyCode::Char('W') => {
                if self.save_state().is_ok() {
                    self.event_log
                        .push_back(("World state SAVED to save.json".to_string(), Color::Green));
                }
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                if self.load_state().is_ok() {
                    self.event_log.push_back((
                        "World state LOADED from save.json".to_string(),
                        Color::Green,
                    ));
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
        if self.show_brain && mouse.column >= self.last_sidebar_rect.x {
            // Clicked in sidebar
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                let relative_y = mouse.row.saturating_sub(self.last_sidebar_rect.y + 1);
                let offset = self.gene_editor_offset;
                use crate::app::state::GeneType;
                self.focused_gene = if relative_y == offset {
                    Some(GeneType::Trophic)
                } else if relative_y == offset + 1 {
                    Some(GeneType::Sensing)
                } else if relative_y == offset + 2 {
                    Some(GeneType::Speed)
                } else if relative_y == offset + 3 {
                    Some(GeneType::MaxEnergy)
                } else if relative_y == offset + 4 {
                    Some(GeneType::ReproInvest)
                } else if relative_y == offset + 5 {
                    Some(GeneType::Maturity)
                } else {
                    None
                };
            }
            return;
        }

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
                        if self.is_social_brush {
                            let ix = (wx as usize).min(self.world.width as usize - 1);
                            let iy = (wy as usize).min(self.world.height as usize - 1);
                            self.world.social_grid[iy][ix] = self.social_brush;
                        } else {
                            self.world
                                .terrain
                                .set_cell_type(wx as u16, wy as u16, self.brush_type);
                        }
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
