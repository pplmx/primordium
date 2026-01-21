use crate::app::state::App;
use crate::model::environment::Era;
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
                if self.show_brain {
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
            let world_stats = format!(
                "Pop: {} | Species: {} | Gen: {} | AvgLife: {:.0} | Entropy: {:.2}",
                self.world.entities.len(),
                self.world.pop_stats.species_count,
                max_gen,
                self.world.pop_stats.avg_lifespan,
                self.world.pop_stats.avg_brain_entropy
            );
            f.render_widget(
                Paragraph::new(world_stats).style(Style::default().fg(Color::DarkGray)),
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
            }

            self.render_help(f);
            self.render_onboarding(f);
        }
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
                        " Role:   ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!("{:?}", entity.metabolism.role)),
                ]));
                lines.push(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        " Offspring: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    ratatui::text::Span::raw(format!("{}", entity.metabolism.offspring_count)),
                ]));
                lines.push(ratatui::text::Line::from(""));
                lines.push(ratatui::text::Line::from(" Neural Network Weights:"));

                // Neural Network weights visualization...
                for i in 0..6 {
                    let mut spans = Vec::new();
                    spans.push(ratatui::text::Span::raw(format!(
                        "  {} ",
                        match i {
                            0 => "FX",
                            1 => "FY",
                            2 => "EN",
                            3 => "NB",
                            4 => "PH",
                            5 => "TR",
                            _ => "??",
                        }
                    )));
                    for j in 0..6 {
                        let w = entity.intel.brain.weights_ih[i * 6 + j];
                        let symbol = if w > 0.5 {
                            "█"
                        } else if w > 0.0 {
                            "▓"
                        } else if w > -0.5 {
                            "▒"
                        } else {
                            "░"
                        };
                        spans.push(ratatui::text::Span::styled(
                            symbol,
                            Style::default().fg(if w > 0.0 { Color::Green } else { Color::Red }),
                        ));
                    }
                    lines.push(ratatui::text::Line::from(spans));
                }
                lines.push(ratatui::text::Line::from(""));
                for i in 0..6 {
                    let mut spans = Vec::new();
                    spans.push(ratatui::text::Span::raw("    "));
                    for j in 0..5 {
                        let w = entity.intel.brain.weights_ho[i * 5 + j];
                        let symbol = if w > 0.5 {
                            "█"
                        } else if w > 0.0 {
                            "▓"
                        } else if w > -0.5 {
                            "▒"
                        } else {
                            "░"
                        };
                        spans.push(ratatui::text::Span::styled(
                            symbol,
                            Style::default().fg(if w > 0.0 { Color::Green } else { Color::Red }),
                        ));
                    }
                    if i < 5 {
                        spans.push(ratatui::text::Span::raw(format!(
                            "  <- {}",
                            match i {
                                0 => "Move X",
                                1 => "Move Y",
                                2 => "Boost",
                                3 => "Aggro",
                                4 => "Share",
                                _ => "",
                            }
                        )));
                    }
                    lines.push(ratatui::text::Line::from(spans));
                }

                lines.push(ratatui::text::Line::from(""));
                let dna_short = &entity.intel.brain.to_hex()[..16];
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
