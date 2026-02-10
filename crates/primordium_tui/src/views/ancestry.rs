use primordium_core::snapshot::WorldSnapshot;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct AncestryWidget<'a> {
    pub snapshot: &'a WorldSnapshot,
}

impl<'a> Widget for AncestryWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let tree_block = Block::default()
            .title(" 🌳 Tree of Life (Top Lineages) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));

        let mut lines = Vec::new();
        let mut top_lineages: Vec<_> = self.snapshot.stats.lineage_counts.iter().collect();
        top_lineages.sort_by(|a, b| b.1.cmp(a.1));

        for (id, count) in top_lineages.iter().take(5) {
            lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(
                    format!(" Dynasty #{} ", &id.to_string()[..4]),
                    Style::default().bg(Color::Blue).fg(Color::White),
                ),
                ratatui::text::Span::raw(format!(" ({} alive)", count)),
            ]));

            let members: Vec<_> = self
                .snapshot
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
        Paragraph::new(lines).block(tree_block).render(area, buf);
    }
}
