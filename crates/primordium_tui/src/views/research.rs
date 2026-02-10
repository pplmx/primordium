use primordium_core::snapshot::WorldSnapshot;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use uuid::Uuid;

pub struct ResearchWidget<'a> {
    pub snapshot: &'a WorldSnapshot,
    pub selected_entity: Option<Uuid>,
}

impl<'a> Widget for ResearchWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let research_block = Block::default()
            .title(" 🔬 Neural Research (Plasticity) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        if let Some(id) = self.selected_entity {
            if let Some(entity) = self.snapshot.entities.iter().find(|e| e.id == id) {
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
                deltas.sort_by(|a, b| {
                    b.1.abs()
                        .partial_cmp(&a.1.abs())
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

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

                Paragraph::new(lines)
                    .block(research_block)
                    .render(area, buf);
                return;
            }
        }
        Paragraph::new(" Select an entity to analyze neural plasticity. ")
            .block(research_block)
            .render(area, buf);
    }
}
