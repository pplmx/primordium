use primordium_core::snapshot::WorldSnapshot;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

pub struct BrainWidget<'a> {
    pub snapshot: &'a WorldSnapshot,
    pub selected_entity: Option<Uuid>,
}

impl<'a> Widget for BrainWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(0)])
            .split(area);

        let hof_block = Block::default()
            .title(" 🏆 Hall of Fame ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let mut hof_lines = Vec::new();
        for (score, e) in &self.snapshot.hall_of_fame.top_living {
            let age = self.snapshot.tick - e.metabolism.birth_tick;
            hof_lines.push(ratatui::text::Line::from(format!(
                "Fit {:.0}: {}",
                score,
                &e.identity.id.to_string()[..4]
            )));
            hof_lines.push(ratatui::text::Line::from(format!(
                "  Age: {} | Kids: {}",
                age, e.metabolism.offspring_count
            )));
        }

        Paragraph::new(hof_lines)
            .block(hof_block)
            .render(sidebar_layout[0], buf);

        if let Some(id) = self.selected_entity {
            if let Some(entity) = self.snapshot.entities.iter().find(|e| e.id == id) {
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

                lines.push(ratatui::text::Line::from(" Brain Activity:"));
                let mut out_spans = vec![ratatui::text::Span::raw(" Out: ")];
                for i in 29..41 {
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

                Paragraph::new(lines)
                    .block(brain_block)
                    .render(sidebar_layout[1], buf);
            }
        }
    }
}
