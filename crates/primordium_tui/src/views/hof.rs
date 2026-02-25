use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw_hall_of_fame(f: &mut Frame, has_storage: bool, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new("Hall of Fame - Galactic Federation")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if has_storage {
        let status = Paragraph::new("No legendary entities recorded yet")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Status"));
        f.render_widget(status, chunks[1]);
    } else {
        let error = Paragraph::new("No SQLite storage available.")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(error, chunks[1]);
    }
}
