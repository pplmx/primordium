use crate::app::state::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn draw_hall_of_fame(f: &mut Frame, app: &App, area: Rect) {
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

    if let Some(_storage) = app.world.logger.storage.as_ref() {
        let placeholder = Paragraph::new("Fetching data from SQLite...")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(placeholder, chunks[1]);
    } else {
        let error = Paragraph::new("No SQLite storage available.")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(error, chunks[1]);
    }
}
