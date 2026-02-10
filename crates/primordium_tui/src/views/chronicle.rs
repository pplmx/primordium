use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub struct ChronicleWidget<'a> {
    pub events: &'a [(String, ratatui::style::Color)],
}

impl<'a> Widget for ChronicleWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let events: Vec<ratatui::text::Line> = self
            .events
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
        chronicle.render(area, buf);
    }
}
