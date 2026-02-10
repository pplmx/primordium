use primordium_core::history::PopulationStats;
use primordium_data::Fossil;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct ArcheologyWidget<'a> {
    pub snapshots: &'a [(u64, PopulationStats)],
    pub index: usize,
    pub fossils: &'a [Fossil],
    pub selected_fossil_index: usize,
}

impl<'a> Widget for ArcheologyWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let arch_block = Block::default()
            .title(" 🏛️ Archeology ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(205, 133, 63)));
        let mut lines = Vec::new();
        if self.snapshots.is_empty() {
            lines.push(ratatui::text::Line::from(" No history snapshots found. "));
        } else {
            let (tick, stats) = &self.snapshots[self.index];
            lines.push(ratatui::text::Line::from(format!(
                " Timeline: Tick {} ({}/{})",
                tick,
                self.index + 1,
                self.snapshots.len()
            )));
            lines.push(ratatui::text::Line::from(format!(
                "  Pop: {} | Species: {}",
                stats.population, stats.species_count
            )));
        }
        lines.push(ratatui::text::Line::from(""));
        lines.push(ratatui::text::Line::from(" 🦴 Fossil Record "));
        if self.fossils.is_empty() {
            lines.push(ratatui::text::Line::from("  No fossils excavated yet."));
        } else {
            for (i, fossil) in self.fossils.iter().enumerate().take(15) {
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
        Paragraph::new(lines).block(arch_block).render(area, buf);
    }
}
