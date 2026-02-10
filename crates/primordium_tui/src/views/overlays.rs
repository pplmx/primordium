use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

pub struct LegendWidget;

impl Widget for LegendWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let legend_width = 40.min(area.width - 4);
        let legend_height = 10.min(area.height - 4);
        let legend_area = Rect::new(
            (area.width - legend_width) / 2,
            (area.height - legend_height) / 2,
            legend_width,
            legend_height,
        );

        Clear.render(legend_area, buf);
        let legend_text = vec![
            ratatui::text::Line::from(" Divine Legend "),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(" [M] Mutate  - Force mutation"),
            ratatui::text::Line::from(" [K] Smite   - Destroy entity"),
            ratatui::text::Line::from(" [P] Reincarnate - Reset brain"),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(" [Space] Pause  [H] Help "),
        ];

        Paragraph::new(legend_text)
            .block(Block::default().borders(Borders::ALL))
            .render(legend_area, buf);
    }
}

pub struct CinematicOverlayWidget {
    pub tick: u64,
    pub carbon_level: f64,
}

impl Widget for CinematicOverlayWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let header_area = Rect::new(area.x + 2, area.y + 1, area.width - 4, 3);
        let footer_area = Rect::new(
            area.x + 2,
            area.bottom().saturating_sub(2),
            area.width - 4,
            1,
        );

        let header_block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray));

        let climate_text = if self.carbon_level > 800.0 {
            ("Scorching", Color::Red)
        } else if self.carbon_level > 500.0 {
            ("Warming", Color::Yellow)
        } else {
            ("Balanced", Color::Green)
        };

        let header_text = vec![ratatui::text::Line::from(vec![
            ratatui::text::Span::styled(
                " PRIMORDIUM ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            ratatui::text::Span::raw(format!(" | Tick: {} | Climate: ", self.tick)),
            ratatui::text::Span::styled(climate_text.0, Style::default().fg(climate_text.1)),
        ])];

        Paragraph::new(header_text)
            .block(header_block)
            .render(header_area, buf);

        let footer_text = ratatui::text::Span::styled(
            " [z] Exit Cinematic Mode ",
            Style::default().fg(Color::DarkGray),
        );
        Paragraph::new(footer_text).render(footer_area, buf);
    }
}
