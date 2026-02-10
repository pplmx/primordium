use primordium_core::lineage_registry::LineageRegistry;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct CivilizationWidget<'a> {
    pub registry: &'a LineageRegistry,
}

impl<'a> Widget for CivilizationWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let civ_block = Block::default()
            .title(" 🏛️ Civilization Dashboard ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let mut lines = Vec::new();
        let top_lineages = self.registry.get_top_lineages(5);

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
        Paragraph::new(lines).block(civ_block).render(area, buf);
    }
}
