use crate::app::state::App;
use crate::model::lifecycle;
use crossterm::event::{KeyCode, KeyEvent};
use primordium_core::systems::intel;
use primordium_data::TerrainType;
use rand::Rng;
use ratatui::style::Color;
use std::fs;

impl App {
    pub fn handle_normal_key(&mut self, key: KeyEvent) {
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
            KeyCode::Char('B') => {
                if self.backup_state().is_ok() {
                    self.event_log.push_back((
                        "World state BACKED UP to backups/".to_string(),
                        Color::Green,
                    ));
                }
            }
            KeyCode::Char('a') => self.show_ancestry = !self.show_ancestry,
            KeyCode::Char('y') => {
                self.show_archeology = !self.show_archeology;
                if self.show_archeology {
                    if let Ok(snaps) = self.world.logger.get_snapshots_recent(100) {
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
                self.handle_fossil_resurrection();
            }
            KeyCode::Char('P') if self.show_archeology => {
                self.auto_play_history = !self.auto_play_history;
                self.event_log.push_back((
                    if self.auto_play_history {
                        "Replay: Auto-Play ON".to_string()
                    } else {
                        "Replay: Auto-Play OFF".to_string()
                    },
                    Color::Cyan,
                ));
            }
            KeyCode::Char('P') if !self.show_archeology => {
                if let Ok(()) = self.load_replay("logs/latest_replay.json") {
                    self.event_log.push_back((
                        "Replay STARTED from logs/latest_replay.json".to_string(),
                        Color::Green,
                    ));
                }
            }
            KeyCode::Char('R') if !self.view_mode == 5 => {
                // R is also used for Resource Boom, but let's assume it's gated or shared
                // Wait, R is Resource Boom.
                // Let's use 'E' for rEcording? No, '1-8' are views.
                // Let's use 'S' for Recording (Start/Stop)
            }
            KeyCode::Char('S') => {
                if !self.input_log.is_empty() {
                    let _ = self.save_recording();
                    self.input_log.clear();
                    self.event_log
                        .push_back(("Recording SAVED and CLEARED".to_string(), Color::Green));
                } else {
                    self.event_log.push_back((
                        "Recording STARTED (Input will be logged)".to_string(),
                        Color::Cyan,
                    ));
                }
            }
            KeyCode::Char('A') => {
                self.export_ancestry_tree();
            }
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                if self.show_help {
                    self.onboarding_step = None;
                }
            }
            KeyCode::Tab => {
                // Cycle through UI modes: Standard -> Immersive -> Expert -> Standard
                use crate::app::state::UiMode;
                self.ui_mode = match self.ui_mode {
                    UiMode::Standard => UiMode::Immersive,
                    UiMode::Immersive => UiMode::Expert,
                    UiMode::Expert => UiMode::Standard,
                };
                self.event_log
                    .push_back((format!("UI Mode: {:?}", self.ui_mode), Color::Cyan));
            }
            KeyCode::Char('u') => {
                self.audio.toggle();
                self.event_log.push_back((
                    format!(
                        "Audio {}",
                        if self.audio.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    ),
                    Color::Cyan,
                ));
            }
            KeyCode::Char('{') => {
                self.audio.decrease_volume();
                self.event_log
                    .push_back(("Audio volume decreased".to_string(), Color::Cyan));
            }
            KeyCode::Char('}') => {
                self.audio.increase_volume();
                self.event_log
                    .push_back(("Audio volume increased".to_string(), Color::Cyan));
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
                let idx = c.to_digit(10).map(|d| d as usize).unwrap_or(0);
                self.accept_trade_offer(idx);
            }
            KeyCode::Char('t') | KeyCode::Char('T') if self.view_mode == 5 => {
                self.propose_random_trade();
            }
            KeyCode::Enter if self.onboarding_step.is_some() => {
                self.advance_onboarding();
            }
            KeyCode::Esc if self.onboarding_step.is_some() => {
                let _ = fs::write(".primordium_onboarded", "1");
                self.onboarding_step = None;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.handle_increment_key();
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                self.handle_decrement_key();
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.trigger_genetic_surge();
            }
            KeyCode::Char('c') => {
                self.export_selected_dna();
            }
            KeyCode::Char('C') => {
                self.export_selected_brain();
            }
            KeyCode::Char('v') | KeyCode::Char('V') => {
                self.import_dna_infuse();
            }
            KeyCode::Char('j') | KeyCode::Char('J') => {
                self.toggle_social_brush();
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
                self.mutate_selected_entity();
            }
            KeyCode::Char('k') => {
                self.smite_selected_entity();
            }
            KeyCode::Char('p') => {
                self.reincarnate_selected_entity();
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.send_relief_to_selected();
            }
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                self.cinematic_mode = !self.cinematic_mode;
            }
            KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Char('l') => {
                self.show_legend = !self.show_legend;
            }
            KeyCode::Char('L') => {
                self.trigger_mass_extinction();
            }
            KeyCode::Char('K') => {
                self.toggle_heat_wave();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.trigger_resource_boom();
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

    fn handle_fossil_resurrection(&mut self) {
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
            e.intel.genotype = std::sync::Arc::new(fossil.genotype.clone());
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

    fn export_ancestry_tree(&mut self) {
        let mut genotypes = Vec::new();
        for (_handle, intel) in self.world.ecs.query::<&primordium_data::Intel>().iter() {
            genotypes.push(intel.genotype.clone());
        }

        if let Ok(tree) = self
            .world
            .logger
            .get_ancestry_tree_from_genotypes(&genotypes)
        {
            let dot = tree.to_dot();
            let _ = fs::write("logs/tree.dot", dot);
            self.event_log.push_back((
                "Ancestry Tree exported to logs/tree.dot".to_string(),
                Color::Green,
            ));
        }
    }

    fn accept_trade_offer(&mut self, idx: usize) {
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

    fn propose_random_trade(&mut self) {
        let mut rng = rand::thread_rng();
        use primordium_net::{TradeProposal, TradeResource};
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

    fn trigger_genetic_surge(&mut self) {
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
                &intel::MutationParams {
                    config: &self.config,
                    population: pop,
                    is_radiation_storm: is_storm,
                    specialization: intel.specialization,
                    ancestral_genotype: None,
                    stress_factor: 0.0,
                },
                &mut rng,
            );
            phys.sensing_range = intel.genotype.sensing_range;
            phys.max_speed = intel.genotype.max_speed;
            met.max_energy = intel.genotype.max_energy;
        }
        self.event_log
            .push_back(("GENETIC SURGE!".to_string(), Color::Red));
    }

    fn export_selected_dna(&mut self) {
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
                self.event_log
                    .push_back(("DNA exported to exported_dna.txt".to_string(), Color::Cyan));
            }
        }
    }

    fn export_selected_brain(&mut self) {
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

    fn import_dna_infuse(&mut self) {
        if let Ok(dna) = fs::read_to_string("dna_infuse.txt") {
            if let Ok(genotype) = primordium_data::Genotype::from_hex(dna.trim()) {
                let mut e = lifecycle::create_entity_with_rng(
                    50.0,
                    25.0,
                    self.world.tick,
                    &mut rand::thread_rng(),
                );
                e.intel.genotype = std::sync::Arc::new(genotype);
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

    fn toggle_social_brush(&mut self) {
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

    fn mutate_selected_entity(&mut self) {
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
                    &intel::MutationParams {
                        config: &self.config,
                        population: pop,
                        is_radiation_storm: is_storm,
                        specialization: intel.specialization,
                        ancestral_genotype: None,
                        stress_factor: 0.0,
                    },
                    &mut rng,
                );
                phys.sensing_range = intel.genotype.sensing_range;
                phys.max_speed = intel.genotype.max_speed;
                met.max_energy = intel.genotype.max_energy;
                self.event_log
                    .push_back(("Divine Mutation".to_string(), Color::Yellow));
            }
        }
    }

    fn smite_selected_entity(&mut self) {
        if let Some(id) = self.selected_entity {
            let mut handle_to_despawn = None;
            for (handle, identity) in self.world.ecs.query::<&primordium_data::Identity>().iter() {
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

    fn reincarnate_selected_entity(&mut self) {
        if let Some(id) = self.selected_entity {
            let mut query = self.world.ecs.query::<(
                &mut primordium_data::Intel,
                &mut primordium_data::Physics,
                &mut primordium_data::Metabolism,
                &primordium_data::Identity,
            )>();
            for (_handle, (intel, phys, met, identity)) in query.iter() {
                if identity.id == id {
                    intel.genotype =
                        std::sync::Arc::new(crate::model::brain::create_genotype_random_with_rng(
                            &mut rand::thread_rng(),
                        ));
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

    fn send_relief_to_selected(&mut self) {
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
                    use primordium_net::NetMessage;
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

    fn trigger_mass_extinction(&mut self) {
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

    fn toggle_heat_wave(&mut self) {
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

    fn trigger_resource_boom(&mut self) {
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
}
