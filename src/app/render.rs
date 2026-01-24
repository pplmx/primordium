use crate::app::state::App;
use crate::model::state::environment::Era;
use crate::ui::renderer::WorldWidget;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Sparkline};
use ratatui::Frame;

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
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

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Status (4 lines: CPU, RAM, Stats, Legend)
                Constraint::Length(4), // Sparklines
                Constraint::Min(0),    // World
                Constraint::Length(7), // Chronicle
            ])
            .split(main_layout[0]);

        self.last_world_rect = left_layout[2];

        if self.screensaver {
            let world_widget = WorldWidget::new(&self.world, true);
            f.render_widget(world_widget, f.size());
        } else {
            // STATUS BAR
            let status_lines = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // CPU + Era
                    Constraint::Length(1), // RAM + Resources
                    Constraint::Length(1), // World Stats
                    Constraint::Length(1), // Legend
                ])
                .split(left_layout[0]);

            let line1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(status_lines[0]);
            let era_icon = match self.env.current_era {
                Era::Primordial => "🌀",
                Era::DawnOfLife => "🌱",
                Era::Flourishing => "🌸",
                Era::DominanceWar => "⚔️",
                Era::ApexEra => "👑",
            };
            let cpu_gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(self.env.cpu_usage as u16)
                .label(format!(
                    "CPU: {:.1}% | {} {:?}",
                    self.env.cpu_usage, era_icon, self.env.current_era
                ));
            f.render_widget(cpu_gauge, line1[0]);
            f.render_widget(
                Paragraph::new(format!(
                    " | Climate: {} | Time: {}",
                    self.env.climate().icon(),
                    self.env.time_of_day().icon()
                )),
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

            let max_gen = self
                .world
                .entities
                .iter()
                .map(|e| e.metabolism.generation)
                .max()
                .unwrap_or(0);
            let total_biomass = self.world.pop_stats.biomass_h + self.world.pop_stats.biomass_c;
            let h_percent = if total_biomass > 0.0 {
                (self.world.pop_stats.biomass_h / total_biomass * 10.0) as usize
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

            let pressure = (self.env.metabolism_multiplier() - 1.0) * 100.0;
            let world_stats = vec![
                ratatui::text::Span::raw(format!("Pop: {} | ", self.world.entities.len())),
                ratatui::text::Span::styled(
                    "Biomass: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(biomass_bar, Style::default().fg(Color::Yellow)),
                ratatui::text::Span::raw(format!(
                    " | Species: {} | Gen: {} | AvgLife: {:.0} | CO2: {:.0} | Soil: {:.2}",
                    self.world.pop_stats.species_count,
                    max_gen,
                    self.world.pop_stats.avg_lifespan,
                    self.world.pop_stats.carbon_level,
                    self.world.pop_stats.global_fertility,
                )),
                ratatui::text::Span::styled(
                    format!(" | Mut: {:.2}x", self.world.pop_stats.mutation_scale),
                    Style::default().fg(Color::DarkGray),
                ),
                ratatui::text::Span::styled(
                    format!(" | Vel: {:.2}", self.world.pop_stats.evolutionary_velocity),
                    Style::default().fg(Color::Magenta),
                ),
                ratatui::text::Span::styled(
                    format!(" | Pressure: {:.0}%", pressure),
                    Style::default().fg(if pressure > 50.0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ];

            f.render_widget(
                Paragraph::new(ratatui::text::Line::from(world_stats))
                    .style(Style::default().fg(Color::DarkGray)),
                status_lines[2],
            );

            // LEGEND BAR
            let legend_spans = vec![
                ratatui::text::Span::styled("● ", Style::default().fg(Color::White)),
                ratatui::text::Span::raw("Forage "),
                ratatui::text::Span::styled("♦ ", Style::default().fg(Color::Rgb(255, 69, 0))),
                ratatui::text::Span::raw("Hunt "),
                ratatui::text::Span::styled("♥ ", Style::default().fg(Color::Rgb(255, 105, 180))),
                ratatui::text::Span::raw("Mate "),
                ratatui::text::Span::styled("† ", Style::default().fg(Color::Rgb(150, 50, 50))),
                ratatui::text::Span::raw("Starve "),
                ratatui::text::Span::styled("☣ ", Style::default().fg(Color::Rgb(154, 205, 50))),
                ratatui::text::Span::raw("Infect "),
                ratatui::text::Span::styled("◦ ", Style::default().fg(Color::Rgb(200, 200, 255))),
                ratatui::text::Span::raw("Juv "),
                ratatui::text::Span::styled("♣ ", Style::default().fg(Color::Rgb(100, 200, 100))),
                ratatui::text::Span::raw("Share"),
                ratatui::text::Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                ratatui::text::Span::styled("▲", Style::default().fg(Color::Rgb(100, 100, 100))),
                ratatui::text::Span::raw("Mt "),
                ratatui::text::Span::styled("≈", Style::default().fg(Color::Rgb(70, 130, 180))),
                ratatui::text::Span::raw("River "),
                ratatui::text::Span::styled("◊", Style::default().fg(Color::Rgb(50, 205, 50))),
                ratatui::text::Span::raw("Oasis "),
                ratatui::text::Span::styled("*", Style::default().fg(Color::Green)),
                ratatui::text::Span::raw("Food"),
                ratatui::text::Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                ratatui::text::Span::styled("[H]", Style::default().fg(Color::Cyan)),
                ratatui::text::Span::raw(" Help"),
                ratatui::text::Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                ratatui::text::Span::styled(
                    format!("Brush: {:?}", self.brush_type),
                    Style::default().fg(Color::Yellow),
                ),
            ];
            f.render_widget(
                Paragraph::new(ratatui::text::Line::from(legend_spans)),
                status_lines[3],
            );

            // SPARKLINE PANES
            let spark_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(left_layout[1]);

            let cpu_history: Vec<u64> = self.cpu_history.iter().cloned().collect();
            let cpu_spark = Sparkline::default()
                .block(
                    Block::default()
                        .title("CPU Load (60s)")
                        .borders(Borders::LEFT),
                )
                .data(&cpu_history)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(cpu_spark, spark_layout[0]);

            let pop_history: Vec<u64> = self.pop_history.iter().cloned().collect();
            let pop_spark = Sparkline::default()
                .block(
                    Block::default()
                        .title("Population Health (60s)")
                        .borders(Borders::LEFT),
                )
                .data(&pop_history)
                .style(Style::default().fg(Color::Magenta));
            f.render_widget(pop_spark, spark_layout[1]);

            let world_widget = WorldWidget::new(&self.world, false);
            f.render_widget(world_widget, left_layout[2]);

            let chronicle_block = Block::default()
                .title(" 📜 Live Chronicle ")
                .borders(Borders::ALL);
            let chronicle_lines: Vec<_> = self
                .event_log
                .iter()
                .map(|(msg, color)| {
                    ratatui::text::Line::from(ratatui::text::Span::styled(
                        msg,
                        Style::default().fg(*color),
                    ))
                })
                .collect();
            f.render_widget(
                Paragraph::new(chronicle_lines).block(chronicle_block),
                left_layout[3],
            );

            if self.show_brain {
                self.render_hall_of_fame_and_brain(f, main_layout[1]);
            } else if self.show_ancestry {
                self.render_ancestry_tree(f, main_layout[1]);
            } else if self.show_archeology {
                self.render_archeology(f, main_layout[1]);
            }

            self.render_help(f);
            self.render_onboarding(f);
        }
    }

    fn render_ancestry_tree(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let tree_block = Block::default()
            .title(" 🌳 Tree of Life (Top Lineages) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let mut lines = Vec::new();

        // Use registry to find top living lineages
        let mut top_lineages: Vec<_> = self.world.pop_stats.lineage_counts.iter().collect();
        top_lineages.sort_by(|a, b| b.1.cmp(a.1));

        for (id, count) in top_lineages.iter().take(5) {
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" Dynasty #{} ", &id.to_string()[..4]),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ),
                ratatui::text::Span::raw(format!(" ({} alive)", count)),
            ]));

            // Show a few representatives of this lineage
            let members: Vec<_> = self
                .world
                .entities
                .iter()
                .filter(|e| e.metabolism.lineage_id == **id)
                .take(3)
                .collect();

            for m in members {
                let tp = m.metabolism.trophic_potential;
                let role_icon = if tp < 0.3 {
                    "🌿"
                } else if tp > 0.7 {
                    "🥩"
                } else {
                    "🍪"
                };
                lines.push(ratatui::text::Line::from(format!(
                    "   └── {} {} (Gen {})",
                    role_icon,
                    m.name(),
                    m.metabolism.generation
                )));
            }
            lines.push(ratatui::text::Line::from(""));
        }

        lines.push(ratatui::text::Line::from(" [Shift+A] Export full DOT tree"));

        f.render_widget(Paragraph::new(lines).block(tree_block), area);
    }

    fn render_archeology(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let arch_block = Block::default()
            .title(" 🏛️ Archeology Tool (Deep History) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(205, 133, 63)));

        let mut lines = Vec::new();

        if self.archeology_snapshots.is_empty() {
            lines.push(ratatui::text::Line::from(" No history snapshots found. "));
            lines.push(ratatui::text::Line::from(
                " Run simulation longer to collect data. ",
            ));
        } else {
            let (tick, stats) = &self.archeology_snapshots[self.archeology_index];
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" Timeline: Tick {} ", tick),
                    Style::default()
                        .bg(Color::Rgb(139, 69, 19))
                        .fg(Color::White),
                ),
                ratatui::text::Span::raw(format!(
                    " ({}/{}) ",
                    self.archeology_index + 1,
                    self.archeology_snapshots.len()
                )),
            ]));
            lines.push(ratatui::text::Line::from(format!(
                "  Pop: {} | Species: {}",
                stats.population, stats.species_count
            )));
            lines.push(ratatui::text::Line::from(format!(
                "  CO2: {:.0} ppm | Hotspots: {}",
                stats.carbon_level, stats.biodiversity_hotspots
            )));
            lines.push(ratatui::text::Line::from(
                "  Navigation: [ and ] to travel through time",
            ));
        }

        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                " 🦴 Fossil Record (Extinct Icons) ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));

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
                    ratatui::text::Span::styled(
                        if i == self.selected_fossil_index {
                            " > "
                        } else {
                            "   "
                        },
                        Style::default().fg(Color::Yellow),
                    ),
                    ratatui::text::Span::styled(&fossil.name, style),
                    ratatui::text::Span::raw(format!(
                        " (Gen: {}, Kids: {})",
                        fossil.max_generation, fossil.total_offspring
                    )),
                ]));
            }
            lines.push(ratatui::text::Line::from(""));
            lines.push(ratatui::text::Line::from(
                "  [↑/↓] Select Fossil  [G] Resurrect ",
            ));
        }

        f.render_widget(Paragraph::new(lines).block(arch_block), area);
    }

    fn render_hall_of_fame_and_brain(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let mut ho_lines = Vec::new();
        ho_lines.push(ratatui::text::Line::from(" 🏆 Hall of Fame (Living)"));
        ho_lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
            " Score = Age×0.5 + Kids×10 + Peak×0.2",
            Style::default().fg(Color::DarkGray),
        )));
        for (score, e) in &self.world.hall_of_fame.top_living {
            let age = self.world.tick - e.metabolism.birth_tick;
            let style = if Some(e.id) == self.selected_entity {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(e.color())
            };
            ho_lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" {:.0} ", score),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(e.name(), style),
            ]));
            ho_lines.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                format!(
                    "   Age:{} Kids:{} Peak:{:.0}",
                    age, e.metabolism.offspring_count, e.metabolism.peak_energy
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }
        ho_lines.push(ratatui::text::Line::from(""));
        ho_lines.push(ratatui::text::Line::from(" 👑 Dominant Lineages (Pop)"));
        let mut lineage_pop: Vec<_> = self.world.pop_stats.lineage_counts.iter().collect();
        lineage_pop.sort_by(|a, b| b.1.cmp(a.1));
        for (id, count) in lineage_pop.iter().take(3) {
            ho_lines.push(ratatui::text::Line::from(format!(
                "   #{} : {} entities",
                &id.to_string()[..8],
                count
            )));
        }

        ho_lines.push(ratatui::text::Line::from(""));
        ho_lines.push(ratatui::text::Line::from(
            " 🏆 All-Time Success (Total Pop)",
        ));
        let top_historical = self.world.lineage_registry.get_top_lineages(3);
        for (id, record) in top_historical {
            ho_lines.push(ratatui::text::Line::from(format!(
                "   #{} : {} total",
                &id.to_string()[..8],
                record.total_entities_produced
            )));
        }
        ho_lines.push(ratatui::text::Line::from(""));

        if let Some(id) = self.selected_entity {
            if let Some(entity) = self.world.entities.iter().find(|e| e.id == id) {
                let brain_block = Block::default()
                    .title(format!(" 🧬 {} ", entity.name()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(entity.color()));
                let mut lines = ho_lines;
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Energy: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "{:.1}/{:.1}",
                        entity.metabolism.energy, entity.metabolism.max_energy
                    )),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Age:    ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "{} ticks",
                        self.world.tick - entity.metabolism.birth_tick
                    )),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Trophic: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "{:.1} (Herb:{:.0}%)",
                        entity.metabolism.trophic_potential,
                        (1.0 - entity.metabolism.trophic_potential) * 100.0
                    )),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Offspring: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!("{}", entity.metabolism.offspring_count)),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Lineage:   ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "#{}",
                        &entity.metabolism.lineage_id.to_string()[..8]
                    )),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Sensing: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!("{:.1}", entity.physics.sensing_range)),
                ]));

                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Speed:   ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!("{:.1}", entity.physics.max_speed)),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Repro Invest: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "{:.0}%",
                        entity.intel.genotype.reproductive_investment * 100.0
                    )),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Maturity Gene: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!(
                        "{:.1}x",
                        entity.intel.genotype.maturity_gene
                    )),
                ]));
                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Brain Complexity:",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(ratatui::text::Line::from(format!(
                    "  Nodes:       {}",
                    entity.intel.genotype.brain.nodes.len()
                )));
                lines.push(ratatui::text::Line::from(format!(
                    "  Connections: {}",
                    entity
                        .intel
                        .genotype
                        .brain
                        .connections
                        .iter()
                        .filter(|c| c.enabled)
                        .count()
                )));
                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(" Strongest Connections:"));

                // Sort and show top 12 connections
                let mut conns = entity.intel.genotype.brain.connections.clone();
                conns.sort_by(|a, b| b.weight.abs().partial_cmp(&a.weight.abs()).unwrap());

                for c in conns.iter().filter(|c| c.enabled).take(12) {
                    let from_label = match c.from {
                        0..=17 => format!(
                            "In-{}",
                            match c.from {
                                0 => "FX",
                                1 => "FY",
                                2 => "EN",
                                3 => "NB",
                                4 => "PH",
                                5 => "TR",
                                6 => "KX",
                                7 => "KY",
                                8 => "SA",
                                9 => "SB",
                                10 => "WL",
                                11 => "AG",
                                _ => "Mem",
                            }
                        ),
                        _ => format!("Hid-{}", c.from),
                    };
                    let to_label = match c.to {
                        18..=25 => format!(
                            "Out-{}",
                            match c.to {
                                18 => "MovX",
                                19 => "MovY",
                                20 => "Spd",
                                21 => "Agg",
                                22 => "Shr",
                                23 => "Clr",
                                24 => "EmA",
                                25 => "EmB",
                                _ => "?",
                            }
                        ),
                        _ => format!("Hid-{}", c.to),
                    };

                    lines.push(ratatui::text::Line::from(vec![
                        ratatui::text::Span::raw(format!("  {} → {} ", from_label, to_label)),
                        ratatui::text::Span::styled(
                            format!("{:.2}", c.weight),
                            Style::default().fg(if c.weight > 0.0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                        ),
                    ]));
                }

                lines.push(ratatui::text::Line::from(""));
                let dna_short = &entity.intel.genotype.to_hex()[..16];

                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " [C] Export DNA ",
                        Style::default().bg(Color::Blue).fg(Color::White),
                    ),
                ]));
                lines.push(ratatui::text::Line::from(format!(" DNA: {}...", dna_short)));
                f.render_widget(Paragraph::new(lines).block(brain_block), area);
            }
        }
    }
}
