use crate::app::state::App;
use crate::model::lifecycle;
use crate::model::systems::intel;
use crate::model::terrain::TerrainType;
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
                    let mut e = lifecycle::create_entity_with_rng(
                        50.0,
                        25.0,
                        self.world.tick,
                        &mut rand::thread_rng(),
                    );
                    e.intel.genotype = fossil.genotype.clone();
                    e.physics.sensing_range = e.intel.genotype.sensing_range;
                    e.physics.max_speed = e.intel.genotype.max_speed;
                    e.metabolism.max_energy = e.intel.genotype.max_energy;
                    e.metabolism.lineage_id = e.intel.genotype.lineage_id;
                    e.metabolism.energy = e.metabolism.max_energy;

                    self.world.ecs.spawn((
                        e.identity,
                        crate::model::state::Position {
                            x: e.physics.x,
                            y: e.physics.y,
                        },
                        e.physics,
                        e.metabolism,
                        e.health,
                        e.intel,
                    ));
                    self.event_log.push_back((
                        format!("RESURRECTED: {} cloned into current world", fossil.name),
                        Color::Magenta,
                    ));
                }
            }
            KeyCode::Char('A') => {
                let entities = self.world.get_all_entities();
                if let Ok(tree) = self.world.logger.get_ancestry_tree(&entities) {
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
                    self.onboarding_step = None;
                }
            }
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
            KeyCode::Char('8') if !self.show_help && self.onboarding_step.is_none() => {
                self.view_mode = 7;
                self.event_log
                    .push_back(("View: CIVILIZATION".to_string(), Color::Yellow));
            }
            KeyCode::Char('0') if self.view_mode == 6 => {
                if let Some(id) = self.selected_entity {
                    self.world.clear_research_deltas(id);
                    self.event_log
                        .push_back(("Research deltas cleared".to_string(), Color::Cyan));
                }
            }
            KeyCode::Char('1') if self.show_help => self.help_tab = 0,
            KeyCode::Char('2') if self.show_help => self.help_tab = 1,
            KeyCode::Char('3') if self.show_help => self.help_tab = 2,
            KeyCode::Char('4') if self.show_help => self.help_tab = 3,
            KeyCode::Char('5') if self.show_help => self.help_tab = 4,
            KeyCode::Char('6') if self.show_help => self.help_tab = 5,
            KeyCode::Char(c) if self.view_mode == 5 && c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if let Some(offer) = self.network_state.trade_offers.get(idx).cloned() {
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
                    self.network_state.trade_offers.remove(idx);
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') if self.view_mode == 5 => {
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
            KeyCode::Enter if self.onboarding_step.is_some() => {
                self.advance_onboarding();
            }
            KeyCode::Esc if self.onboarding_step.is_some() => {
                let _ = fs::write(".primordium_onboarded", "1");
                self.onboarding_step = None;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                if self.show_brain && self.focused_gene.is_some() {
                    if let (Some(id), Some(gene)) = (self.selected_entity, self.focused_gene) {
                        let delta = match gene {
                            primordium_data::GeneType::Trophic => 0.05,
                            primordium_data::GeneType::Sensing => 0.5,
                            primordium_data::GeneType::Speed => 0.1,
                            primordium_data::GeneType::MaxEnergy => 10.0,
                            primordium_data::GeneType::ReproInvest => 0.05,
                            primordium_data::GeneType::Maturity => 0.1,
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
                            primordium_data::GeneType::Trophic => -0.05,
                            primordium_data::GeneType::Sensing => -0.5,
                            primordium_data::GeneType::Speed => -0.1,
                            primordium_data::GeneType::MaxEnergy => -10.0,
                            primordium_data::GeneType::ReproInvest => -0.05,
                            primordium_data::GeneType::Maturity => -0.1,
                        };
                        self.world.apply_genetic_edit(id, gene, delta);
                    }
                } else {
                    self.time_scale = (self.time_scale - 0.5).max(0.5);
                }
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                let is_storm = self.env.is_radiation_storm();
                let mut rng = rand::thread_rng();
                let mut query = self.world.ecs.query::<(
                    &mut primordium_data::Intel,
                    &mut primordium_data::Physics,
                    &mut primordium_data::Metabolism,
                )>();
                let components: Vec<_> = query.into_iter().collect();
                let pop = components.len();
                for (_handle, (intel, phys, met)) in components {
                    intel::mutate_genotype(
                        &mut intel.genotype,
                        &self.config,
                        pop,
                        is_storm,
                        intel.specialization,
                        &mut rng,
                        None,
                    );
                    phys.sensing_range = intel.genotype.sensing_range;
                    phys.max_speed = intel.genotype.max_speed;
                    met.max_energy = intel.genotype.max_energy;
                }
                self.event_log
                    .push_back(("GENETIC SURGE!".to_string(), Color::Red));
            }
            KeyCode::Char('c') => {
                if let Some(id) = self.selected_entity {
                    let mut found_dna = None;
                    for (_handle, (identity, intel)) in self
                        .world
                        .ecs
                        .query::<(&primordium_data::Identity, &primordium_data::Intel)>()
                        .iter()
                    {
                        if identity.id == id {
                            found_dna = Some(intel.genotype.to_hex());
                            break;
                        }
                    }
                    if let Some(dna) = found_dna {
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
                    let mut found_brain = None;
                    for (_handle, (identity, intel)) in self
                        .world
                        .ecs
                        .query::<(&primordium_data::Identity, &primordium_data::Intel)>()
                        .iter()
                    {
                        if identity.id == id {
                            found_brain = Some(serde_json::to_string_pretty(&intel.genotype.brain));
                            break;
                        }
                    }
                    if let Some(Ok(json)) = found_brain {
                        let filename = format!("logs/brain_{}.json", id);
                        let _ = fs::write(&filename, json);
                        self.event_log.push_back((
                            format!("Brain JSON exported to {}", filename),
                            Color::Magenta,
                        ));
                    }
                }
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                if let Ok(dna) = fs::read_to_string("dna_infuse.txt") {
                    if let Ok(genotype) = primordium_data::Genotype::from_hex(dna.trim()) {
                        let mut e = lifecycle::create_entity_with_rng(
                            50.0,
                            25.0,
                            self.world.tick,
                            &mut rand::thread_rng(),
                        );
                        e.intel.genotype = genotype;
                        e.physics.sensing_range = e.intel.genotype.sensing_range;
                        e.physics.max_speed = e.intel.genotype.max_speed;
                        e.metabolism.max_energy = e.intel.genotype.max_energy;
                        e.metabolism.lineage_id = e.intel.genotype.lineage_id;

                        self.world.ecs.spawn((
                            e.identity,
                            crate::model::state::Position {
                                x: e.physics.x,
                                y: e.physics.y,
                            },
                            e.physics,
                            e.metabolism,
                            e.health,
                            e.intel,
                        ));
                        self.event_log.push_back((
                            "AVATAR INFUSED from dna_infuse.txt".to_string(),
                            Color::Green,
                        ));
                        let _ = fs::remove_file("dna_infuse.txt");
                    }
                }
            }
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
            KeyCode::Char('m') => {
                if let Some(id) = self.selected_entity {
                    let mut query = self.world.ecs.query::<(
                        &mut primordium_data::Intel,
                        &mut primordium_data::Physics,
                        &mut primordium_data::Metabolism,
                        &primordium_data::Identity,
                    )>();
                    let entities: Vec<_> = query.into_iter().collect();
                    let pop = entities.len();
                    let is_storm = self.env.is_radiation_storm();
                    if let Some((_handle, (intel, phys, met, _identity))) = entities
                        .into_iter()
                        .find(|(_, (_, _, _, ident))| ident.id == id)
                    {
                        let mut rng = rand::thread_rng();
                        intel::mutate_genotype(
                            &mut intel.genotype,
                            &self.config,
                            pop,
                            is_storm,
                            intel.specialization,
                            &mut rng,
                            None,
                        );
                        phys.sensing_range = intel.genotype.sensing_range;
                        phys.max_speed = intel.genotype.max_speed;
                        met.max_energy = intel.genotype.max_energy;
                        self.event_log
                            .push_back(("Divine Mutation".to_string(), Color::Yellow));
                    }
                }
            }
            KeyCode::Char('k') => {
                if let Some(id) = self.selected_entity {
                    let mut handle_to_despawn = None;
                    for (handle, identity) in
                        self.world.ecs.query::<&primordium_data::Identity>().iter()
                    {
                        if identity.id == id {
                            handle_to_despawn = Some(handle);
                            break;
                        }
                    }
                    if let Some(handle) = handle_to_despawn {
                        let _ = self.world.ecs.despawn(handle);
                        self.selected_entity = None;
                        self.event_log
                            .push_back(("Divine Smite".to_string(), Color::Red));
                    }
                }
            }
            KeyCode::Char('p') => {
                if let Some(id) = self.selected_entity {
                    let mut query = self.world.ecs.query::<(
                        &mut primordium_data::Intel,
                        &mut primordium_data::Physics,
                        &mut primordium_data::Metabolism,
                        &primordium_data::Identity,
                    )>();
                    for (_handle, (intel, phys, met, identity)) in query.iter() {
                        if identity.id == id {
                            intel.genotype = crate::model::brain::create_genotype_random_with_rng(
                                &mut rand::thread_rng(),
                            );
                            phys.sensing_range = intel.genotype.sensing_range;
                            phys.max_speed = intel.genotype.max_speed;
                            met.max_energy = intel.genotype.max_energy;
                            self.event_log
                                .push_back(("Divine Reincarnation".to_string(), Color::Magenta));
                            break;
                        }
                    }
                }
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if let Some(id) = self.selected_entity {
                    let mut found_lid = None;
                    for (_handle, (identity, met)) in self
                        .world
                        .ecs
                        .query::<(&primordium_data::Identity, &primordium_data::Metabolism)>()
                        .iter()
                    {
                        if identity.id == id {
                            found_lid = Some(met.lineage_id);
                            break;
                        }
                    }
                    if let Some(l_id) = found_lid {
                        if let Some(net) = &self.network {
                            use crate::model::infra::network::NetMessage;
                            let msg = NetMessage::Relief {
                                lineage_id: l_id,
                                amount: 500.0,
                                sender_id: self
                                    .network_state
                                    .client_id
                                    .unwrap_or_else(uuid::Uuid::new_v4),
                            };
                            net.send(&msg);
                            self.event_log.push_back((
                                format!(
                                    "RELIEF SENT: 500.0 energy broadcast for lineage {}",
                                    &l_id.to_string()[..4]
                                ),
                                Color::Yellow,
                            ));
                        }
                    }
                }
            }
            KeyCode::Char('K') => {
                if self.env.god_climate_override.is_some() {
                    self.env.god_climate_override = None;
                    self.event_log
                        .push_back(("God: Climate Restored".to_string(), Color::Cyan));
                } else {
                    self.env.god_climate_override =
                        Some(crate::model::environment::ClimateState::Scorching);
                    self.event_log
                        .push_back(("GOD MODE: HEAT WAVE INDUCED".to_string(), Color::Red));
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                let pop = self.world.get_population_count();
                let kill_count = (pop as f32 * 0.9) as usize;
                let mut handles: Vec<_> = self
                    .world
                    .ecs
                    .query::<&primordium_data::Physics>()
                    .iter()
                    .map(|(h, _)| h)
                    .collect();
                handles.truncate(kill_count);
                for h in handles {
                    let _ = self.world.ecs.despawn(h);
                }
                self.env.carbon_level = 300.0;
                self.event_log.push_back((
                    "GOD MODE: MASS EXTINCTION & CARBON SCRUB".to_string(),
                    Color::Magenta,
                ));
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                use crate::model::state::{MetabolicNiche, Position};
                use primordium_data::Food;
                let mut rng = rand::thread_rng();
                for _ in 0..100 {
                    let fx = rng.gen_range(1..self.world.width - 1);
                    let fy = rng.gen_range(1..self.world.height - 1);
                    let n_type = rng.gen_range(0.0..1.0);
                    self.world.ecs.spawn((
                        Food::new(fx, fy, n_type),
                        Position {
                            x: fx as f64,
                            y: fy as f64,
                        },
                        MetabolicNiche(n_type),
                    ));
                }
                self.world.food_dirty = true;
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
                let _ = fs::write(".primordium_onboarded", "1");
                self.onboarding_step = None;
            } else {
                *step += 1;
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.show_brain && mouse.column >= self.last_sidebar_rect.x {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                let relative_y = mouse.row.saturating_sub(self.last_sidebar_rect.y + 1);
                let offset = self.gene_editor_offset;
                use primordium_data::GeneType;
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
                    let painted = if matches!(mouse.kind, MouseEventKind::Drag(MouseButton::Left)) {
                        true
                    } else {
                        let mut closest_id = None;
                        for (_handle, (identity, pos)) in self
                            .world
                            .ecs
                            .query::<(&primordium_data::Identity, &crate::model::state::Position)>()
                            .iter()
                        {
                            let dx = pos.x - wx;
                            let dy = pos.y - wy;
                            if dx * dx + dy * dy < 4.0 {
                                closest_id = Some(identity.id);
                                break;
                            }
                        }

                        if let Some(id) = closest_id {
                            self.selected_entity = Some(id);
                            self.show_brain = true;
                            false
                        } else {
                            true
                        }
                    };

                    if painted {
                        if self.is_social_brush {
                            let ix = (wx as usize).min(self.world.width as usize - 1);
                            let iy = (wy as usize).min(self.world.height as usize - 1);
                            let width = self.world.width as usize;
                            self.world.social_grid[iy * width + ix] = self.social_brush;
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
                    use crate::model::state::{MetabolicNiche, Position};
                    use primordium_data::Food;
                    let n_type = rand::thread_rng().gen_range(0.0..1.0);
                    self.world.ecs.spawn((
                        Food::new(wx as u16, wy as u16, n_type),
                        Position { x: wx, y: wy },
                        MetabolicNiche(n_type),
                    ));
                    self.world.food_dirty = true;
                    self.event_log
                        .push_back(("Divine Food Injected".to_string(), Color::Green));
                }
            }
            _ => {}
        }
    }
}
