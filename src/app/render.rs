use crate::app::state::App;
use crate::model::state::environment::Era;
use crate::ui::renderer::WorldWidget;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        let snapshot = match &self.latest_snapshot {
            Some(s) => s,
            None => return,
        };

        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                if self.show_brain || self.show_ancestry {
                    Constraint::Length(45)
                } else {
                    Constraint::Length(0)
                },
            ])
            .split(f.size());

        self.last_sidebar_rect = main_layout[1];

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Status
                Constraint::Length(4), // Sparklines
                Constraint::Min(0),    // World
                Constraint::Length(7), // Chronicle
            ])
            .split(main_layout[0]);

        self.last_world_rect = left_layout[2];

        if self.screensaver {
            let world_widget = WorldWidget::new(snapshot, true, self.view_mode);
            f.render_widget(world_widget, f.size());
        } else {
            // STATUS BAR
            let status_lines = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // CPU + Era
                    Constraint::Length(1), // RAM + Resources
                    Constraint::Length(1), // World Stats
                    Constraint::Length(1), // Hive Dashboard
                    Constraint::Length(1), // Legend
                ])
                .split(left_layout[0]);

            let line1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(status_lines[0]);

            let cpu_gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(self.env.cpu_usage as u16)
                .label(format!("CPU: {:.1}%", self.env.cpu_usage));
            f.render_widget(cpu_gauge, line1[0]);

            let era = self.env.current_era;
            let era_info = match era {
                Era::Primordial => ("Primordial", Color::Green),
                Era::DawnOfLife => ("Dawn of Life", Color::Cyan),
                Era::Flourishing => ("Flourishing", Color::Yellow),
                Era::DominanceWar => ("Dominance War", Color::Red),
                Era::ApexEra => ("Apex Era", Color::Magenta),
            };

            f.render_widget(
                Paragraph::new(format!(" | Era: {} | Tick: {}", era_info.0, snapshot.tick))
                    .style(Style::default().fg(era_info.1)),
                line1[1],
            );

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

            let total_biomass = snapshot.stats.biomass_h + snapshot.stats.biomass_c;
            let h_percent = if total_biomass > 0.0 {
                (snapshot.stats.biomass_h / total_biomass * 10.0) as usize
            } else {
                5
            };
            let mut biomass_bar = String::from("[");
            for i in 0..10 {
                if i < h_percent {
                    biomass_bar.push('H');
                } else {
                    biomass_bar.push('C');
                }
            }
            biomass_bar.push(']');

            let view_str = match self.view_mode {
                1 => " [Fertility] ",
                2 => " [Social] ",
                3 => " [Rank] ",
                4 => " [Vocal] ",
                5 => " [Market] ",
                6 => " [Research] ",
                7 => " [Civilization] ",
                _ => " [Normal] ",
            };

            let world_stats = vec![
                ratatui::text::Span::raw(format!("Pop: {} | ", snapshot.entities.len())),
                ratatui::text::Span::styled(view_str, Style::default().fg(Color::Cyan)),
                ratatui::text::Span::styled(
                    "Biomass: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(biomass_bar, Style::default().fg(Color::Yellow)),
                ratatui::text::Span::raw(format!(
                    " | Species: {} | Gen: {} | AvgLife: {:.0} | CO2: {:.0} | O2: {:.1}% | Soil: {:.2}",
                    snapshot.stats.species_count,
                    snapshot.stats.max_generation,
                    snapshot.stats.avg_lifespan,
                    snapshot.stats.carbon_level,
                    self.env.oxygen_level,
                    snapshot.stats.global_fertility,
                )),
            ];

            f.render_widget(
                Paragraph::new(ratatui::text::Line::from(world_stats))
                    .style(Style::default().fg(Color::DarkGray)),
                status_lines[2],
            );

            // HIVE DASHBOARD
            let ns = &self.network_state;
            let hive_stats = vec![
                ratatui::text::Span::styled(" 🕸  Hive: ", Style::default().fg(Color::Cyan)),
                ratatui::text::Span::raw(format!(
                    "{} Peers | In: {} | Out: {} | Status: ",
                    ns.peers.len(),
                    ns.migrations_received,
                    ns.migrations_sent
                )),
                if ns.client_id.is_some() {
                    ratatui::text::Span::styled("Online", Style::default().fg(Color::Green))
                } else {
                    ratatui::text::Span::styled("Offline", Style::default().fg(Color::Red))
                },
            ];

            f.render_widget(
                Paragraph::new(ratatui::text::Line::from(hive_stats)),
                status_lines[3],
            );

            // LEGEND
            let legend = " [Space] Pause | [M] Mutate | [K] Smite | [P] Reincarnate | [A] Ancestry | [Y] Archeology | [H] Help ";
            f.render_widget(
                Paragraph::new(legend).style(Style::default().fg(Color::DarkGray)),
                status_lines[4],
            );

            // SPARKLINE
            let spark_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(left_layout[1]);

            let pop_data: Vec<u64> = self.pop_history.iter().cloned().collect();
            let pop_spark = Sparkline::default()
                .block(Block::default().title(" Population "))
                .data(&pop_data)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(pop_spark, spark_layout[0]);

            let cpu_data: Vec<u64> = self.cpu_history.iter().cloned().collect();
            let cpu_spark = Sparkline::default()
                .block(Block::default().title(" CPU Stress "))
                .data(&cpu_data)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(cpu_spark, spark_layout[1]);

            // WORLD
            let world_widget = WorldWidget::new(snapshot, false, self.view_mode);
            f.render_widget(world_widget, left_layout[2]);

            // CHRONICLE
            let events: Vec<ratatui::text::Line> = self
                .event_log
                .iter()
                .rev()
                .take(6)
                .map(|(msg, color)| {
                    ratatui::text::Line::from(ratatui::text::Span::styled(
                        msg,
                        Style::default().fg(*color),
                    ))
                })
                .collect();
            let chronicle = Paragraph::new(events)
                .block(Block::default().borders(Borders::ALL).title(" Chronicles "));
            f.render_widget(chronicle, left_layout[3]);
        }

        // SIDEBAR
        if self.show_ancestry {
            self.render_ancestry_tree(f, main_layout[1], snapshot);
        } else if self.show_archeology {
            self.render_archeology(f, main_layout[1], snapshot);
        } else if self.show_brain {
            self.gene_editor_offset =
                self.render_hall_of_fame_and_brain(f, main_layout[1], snapshot);
        } else if self.view_mode == 5 {
            self.render_market(f, main_layout[1]);
        } else if self.view_mode == 6 {
            self.render_research(f, main_layout[1], snapshot);
        } else if self.view_mode == 7 {
            self.render_civilization(f, main_layout[1], snapshot);
        }

        if self.show_help {
            self.render_help(f);
        }

        if let Some(_step) = self.onboarding_step {
            self.render_onboarding(f);
        }
    }

    fn render_ancestry_tree(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        snapshot: &crate::model::state::snapshot::WorldSnapshot,
    ) {
        let tree_block = Block::default()
            .title(" 🌳 Tree of Life (Top Lineages) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let mut lines = Vec::new();
        let mut top_lineages: Vec<_> = snapshot.stats.lineage_counts.iter().collect();
        top_lineages.sort_by(|a, b| b.1.cmp(a.1));

        for (id, count) in top_lineages.iter().take(5) {
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" Dynasty #{} ", &id.to_string()[..4]),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ),
                ratatui::text::Span::raw(format!(" ({} alive)", count)),
            ]));

            let members: Vec<_> = snapshot
                .entities
                .iter()
                .filter(|e| e.lineage_id == **id)
                .take(3)
                .collect();

            for m in members {
                let tp = m.trophic_potential;
                let role_icon = if tp < 0.3 {
                    "🌿"
                } else if tp > 0.7 {
                    "🥩"
                } else {
                    "🍪"
                };
                lines.push(ratatui::text::Line::from(format!(
                    "   └── {} {} (Gen {})",
                    role_icon, m.name, m.generation
                )));
            }
            lines.push(ratatui::text::Line::from(""));
        }
        lines.push(ratatui::text::Line::from(" [Shift+A] Export full DOT tree"));
        f.render_widget(Paragraph::new(lines).block(tree_block), area);
    }

    fn render_archeology(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        _snapshot: &crate::model::state::snapshot::WorldSnapshot,
    ) {
        let arch_block = Block::default()
            .title(" 🏛️ Archeology ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(205, 133, 63)));
        let mut lines = Vec::new();
        if self.archeology_snapshots.is_empty() {
            lines.push(ratatui::text::Line::from(" No history snapshots found. "));
        } else {
            let (tick, stats) = &self.archeology_snapshots[self.archeology_index];
            lines.push(ratatui::text::Line::from(format!(
                " Timeline: Tick {} ({}/{})",
                tick,
                self.archeology_index + 1,
                self.archeology_snapshots.len()
            )));
            lines.push(ratatui::text::Line::from(format!(
                "  Pop: {} | Species: {}",
                stats.population, stats.species_count
            )));
        }
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(" 🦴 Fossil Record "));
        if self.world.fossil_registry.fossils.is_empty() {
            lines.push(ratatui::text::Line::from("  No fossils excavated yet."));
        } else {
            for (i, fossil) in self
                .world
                .fossil_registry
                .fossils
                .iter()
                .enumerate()
                .take(15)
            {
                let style = if i == self.selected_fossil_index {
                    Style::default().bg(Color::Rgb(80, 80, 80)).fg(Color::White)
                } else {
                    Style::default().fg(Color::Rgb(
                        fossil.color_rgb.0,
                        fossil.color_rgb.1,
                        fossil.color_rgb.2,
                    ))
                };
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::raw(if i == self.selected_fossil_index {
                        " > "
                    } else {
                        "   "
                    }),
                    ratatui::text::Span::styled(&fossil.name, style),
                    ratatui::text::Span::raw(format!(" (Gen: {})", fossil.max_generation)),
                ]));
            }
        }
        f.render_widget(Paragraph::new(lines).block(arch_block), area);
    }

    fn render_hall_of_fame_and_brain(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        snapshot: &crate::model::state::snapshot::WorldSnapshot,
    ) -> u16 {
        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        let hof_block = Block::default()
            .title(" 🏆 Hall of Fame ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let mut hof_lines = Vec::new();
        for (score, e) in &snapshot.hall_of_fame.top_living {
            let age = snapshot.tick - e.metabolism.birth_tick;
            hof_lines.push(ratatui::text::Line::from(format!(
                "Fit {:.0}: {}",
                score,
                e.name()
            )));
            hof_lines.push(ratatui::text::Line::from(format!(
                "  Age: {} | Kids: {}",
                age, e.metabolism.offspring_count
            )));
        }
        f.render_widget(
            Paragraph::new(hof_lines).block(hof_block),
            sidebar_layout[0],
        );

        let mut offset = 0;
        if let Some(id) = self.selected_entity {
            if let Some(entity) = snapshot.entities.iter().find(|e| e.id == id) {
                let brain_block = Block::default()
                    .title(format!(" 🧬 {} ", entity.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(entity.r, entity.g, entity.b)));

                let mut lines = Vec::new();
                lines.push(ratatui::text::Line::from(format!(
                    " Energy: {:.0}/{:.0}",
                    entity.energy, entity.max_energy
                )));
                lines.push(ratatui::text::Line::from(format!(
                    " Gen: {} | Age: {}",
                    entity.generation, entity.age
                )));
                lines.push(ratatui::text::Line::from(format!(
                    " Rank: {:.2}",
                    entity.rank
                )));

                offset = lines.len() as u16;

                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(" Brain Activity:"));
                let mut out_spans = vec![ratatui::text::Span::raw(" Out: ")];
                for i in 22..33 {
                    let val = *entity.last_activations.get(&{ i }).unwrap_or(&0.0);
                    out_spans.push(ratatui::text::Span::styled(
                        format!("{:.1} ", val),
                        Style::default().fg(if val > 0.0 { Color::Green } else { Color::Red }),
                    ));
                }
                lines.push(ratatui::text::Line::from(out_spans));

                if let Some(ref hex) = entity.genotype_hex {
                    let dna_short = if hex.len() > 16 { &hex[..16] } else { hex };
                    lines.push(ratatui::text::Line::from(format!(" DNA: {}...", dna_short)));
                }

                f.render_widget(Paragraph::new(lines).block(brain_block), sidebar_layout[1]);
            }
        }
        offset
    }

    fn render_market(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let market_block = Block::default()
            .title(" 💹 Multiverse Market ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let mut lines = Vec::new();
        if self.network_state.trade_offers.is_empty() {
            lines.push(ratatui::text::Line::from(" No active trade offers. "));
        } else {
            for (i, offer) in self.network_state.trade_offers.iter().enumerate() {
                lines.push(ratatui::text::Line::from(format!(
                    " #{} Offer: {:.0} {:?}",
                    i, offer.offer_amount, offer.offer_resource
                )));
                lines.push(ratatui::text::Line::from(format!(
                    "      Request: {:.0} {:?}",
                    offer.request_amount, offer.request_resource
                )));
            }
        }
        f.render_widget(Paragraph::new(lines).block(market_block), area);
    }

    fn render_research(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        snapshot: &crate::model::state::snapshot::WorldSnapshot,
    ) {
        let research_block = Block::default()
            .title(" 🔬 Neural Research (Plasticity) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        if let Some(id) = self.selected_entity {
            if let Some(entity) = snapshot.entities.iter().find(|e| e.id == id) {
                let mut lines = vec![
                    ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            " Subject: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        ratatui::text::Span::raw(entity.name.clone()),
                    ]),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                        " Synaptic Plasticity Heatmap (Δw magnitude):",
                        Style::default().add_modifier(Modifier::BOLD),
                    )]),
                ];

                let mut deltas: Vec<_> = entity.weight_deltas.iter().collect();
                deltas.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

                for (innovation, delta) in deltas.iter().take(20) {
                    let d_abs = delta.abs();
                    if d_abs < 1e-5 {
                        continue;
                    }

                    let color = if d_abs > 0.1 {
                        Color::Yellow
                    } else if d_abs > 0.01 {
                        Color::Cyan
                    } else {
                        Color::Blue
                    };

                    let bar_len = (d_abs * 100.0).min(15.0) as usize;
                    lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::styled(
                            format!("  #{} ", innovation),
                            Style::default().fg(Color::DarkGray),
                        ),
                        ratatui::text::Span::raw(format!("Δw: {:.5} ", d_abs)),
                        ratatui::text::Span::styled(
                            "█".repeat(bar_len),
                            Style::default().fg(color),
                        ),
                    ]));
                }

                if lines.len() <= 3 {
                    lines.push(ratatui::text::Line::from(
                        " No significant synaptic changes detected. ",
                    ));
                }

                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(
                    " [0] Clear Deltas  [C] Export DNA ",
                ));

                f.render_widget(Paragraph::new(lines).block(research_block), area);
                return;
            }
        }
        f.render_widget(
            Paragraph::new(" Select an entity to analyze neural plasticity. ")
                .block(research_block),
            area,
        );
    }

    fn render_civilization(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        _snapshot: &crate::model::state::snapshot::WorldSnapshot,
    ) {
        let civ_block = Block::default()
            .title(" 🏛️ Civilization Dashboard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let mut lines = Vec::new();
        let top_lineages = self.world.lineage_registry.get_top_lineages(5);

        if top_lineages.is_empty() {
            lines.push(ratatui::text::Line::from(
                " No dominant civilizations detected. ",
            ));
        } else {
            for (_id, record) in top_lineages {
                let color = Color::Cyan;
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        format!(" {} ", record.name),
                        Style::default()
                            .bg(color)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(" Level: {}", record.civilization_level)),
                ]));

                lines.push(ratatui::text::Line::from(format!(
                    "  Pop: {} | Energy: {:.0}",
                    record.current_population, record.total_energy_consumed
                )));

                let mut goals = vec![ratatui::text::Span::raw("  Goals: ")];
                if record.completed_goals.is_empty() {
                    goals.push(ratatui::text::Span::styled(
                        "None",
                        Style::default().fg(Color::DarkGray),
                    ));
                } else {
                    for goal in &record.completed_goals {
                        goals.push(ratatui::text::Span::styled(
                            format!("{:?} ", goal),
                            Style::default().fg(Color::Green),
                        ));
                    }
                }
                lines.push(ratatui::text::Line::from(goals));

                let mut traits = vec![ratatui::text::Span::raw("  Traits: ")];
                if record.ancestral_traits.is_empty() {
                    traits.push(ratatui::text::Span::styled(
                        "None",
                        Style::default().fg(Color::DarkGray),
                    ));
                } else {
                    for t in &record.ancestral_traits {
                        traits.push(ratatui::text::Span::styled(
                            format!("{:?} ", t),
                            Style::default().fg(Color::Yellow),
                        ));
                    }
                }
                lines.push(ratatui::text::Line::from(traits));

                if let Ok(mem) = record.collective_memory.read() {
                    if !mem.is_empty() {
                        let mut mem_line = vec![ratatui::text::Span::raw("  Memory: ")];
                        for (k, v) in mem.iter().take(3) {
                            mem_line.push(ratatui::text::Span::styled(
                                format!("{}:{:.1} ", k, v),
                                Style::default().fg(Color::Magenta),
                            ));
                        }
                        lines.push(ratatui::text::Line::from(mem_line));
                    }
                }

                lines.push(ratatui::text::Line::from(""));
            }
        }

        f.render_widget(Paragraph::new(lines).block(civ_block), area);
    }
}
