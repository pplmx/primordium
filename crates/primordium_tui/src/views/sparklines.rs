use primordium_core::environment::Era;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Sparkline, Widget};

pub struct SparklinesWidget<'a> {
    pub pop_data: &'a [u64],
    pub cpu_data: &'a [u64],
    pub current_era: Era,
}

impl<'a> Widget for SparklinesWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let spark_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let era_color = match self.current_era {
            Era::Primordial => Color::Green,
            Era::DawnOfLife => Color::Cyan,
            Era::Flourishing => Color::Yellow,
            Era::DominanceWar => Color::Red,
            Era::ApexEra => Color::Magenta,
        };

        Sparkline::default()
            .block(Block::default().title(" Population "))
            .data(self.pop_data)
            .style(Style::default().fg(era_color))
            .render(spark_layout[0], buf);

        Sparkline::default()
            .block(Block::default().title(" CPU Stress "))
            .data(self.cpu_data)
            .style(Style::default().fg(Color::Yellow))
            .render(spark_layout[1], buf);
    }
}
