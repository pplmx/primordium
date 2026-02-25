use primordium_core::environment::Era;
use primordium_core::snapshot::WorldSnapshot;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Gauge, Paragraph, Widget};

pub struct StatusWidget<'a> {
    pub snapshot: &'a WorldSnapshot,
    pub cpu_usage: f64,
    pub ram_usage_percent: f32,
    pub app_memory_usage_mb: f64,
    pub current_era: Era,
    pub oxygen_level: f64,
    pub view_mode: u8,
    pub peer_count: usize,
    pub migrations_received: u64,
    pub migrations_sent: u64,
    pub is_online: bool,
    pub resource_icon: String,
    pub available_energy: f64,
}

impl<'a> Widget for StatusWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let status_lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        let line1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(status_lines[0]);

        let cpu_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(self.cpu_usage as u16)
            .label(format!("CPU: {:.1}%", self.cpu_usage));
        cpu_gauge.render(line1[0], buf);

        let era_info = match self.current_era {
            Era::Primordial => ("Primordial", Color::Green),
            Era::DawnOfLife => ("Dawn of Life", Color::Cyan),
            Era::Flourishing => ("Flourishing", Color::Yellow),
            Era::DominanceWar => ("Dominance War", Color::Red),
            Era::ApexEra => ("Apex Era", Color::Magenta),
        };

        Paragraph::new(format!(
            " | Era: {} | Tick: {}",
            era_info.0, self.snapshot.tick
        ))
        .style(Style::default().fg(era_info.1))
        .render(line1[1], buf);

        let line2 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(status_lines[1]);

        let ram_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(self.ram_usage_percent as u16)
            .label(format!(
                "RAM: {:.1}% (App: {:.0}MB)",
                self.ram_usage_percent, self.app_memory_usage_mb
            ));
        ram_gauge.render(line2[0], buf);

        Paragraph::new(format!(" | Resources: {}", self.resource_icon)).render(line2[1], buf);

        let total_biomass = self.snapshot.stats.biomass_h + self.snapshot.stats.biomass_c;
        let h_percent = if total_biomass > 0.0 {
            (self.snapshot.stats.biomass_h / total_biomass * 10.0) as usize
        } else {
            5
        };
        let mut biomass_bar = String::from("[");
        for i in 0..10 {
            if i < h_percent {
                biomass_bar.push('H');
            } else {
                biomass_bar.push('C');
            }
        }
        biomass_bar.push(']');

        let view_str = match self.view_mode {
            1 => " [Fertility] ",
            2 => " [Social] ",
            3 => " [Rank] ",
            4 => " [Vocal] ",
            5 => " [Market] ",
            6 => " [Research] ",
            7 => " [Civilization] ",
            _ => " [Normal] ",
        };

        let world_stats = vec![
            ratatui::text::Span::raw(format!("Pop: {} | ", self.snapshot.entities.len())),
            ratatui::text::Span::styled(view_str, Style::default().fg(Color::Cyan)),
            ratatui::text::Span::styled("Biomass: ", Style::default().add_modifier(Modifier::BOLD)),
            ratatui::text::Span::styled(biomass_bar, Style::default().fg(Color::Yellow)),
            ratatui::text::Span::raw(format!(
                " | Species: {} | Gen: {} | AvgLife: {:.0} | CO2: {:.0} | O2: {:.1}% | Soil: {:.2}",
                self.snapshot.stats.species_count,
                self.snapshot.stats.max_generation,
                self.snapshot.stats.avg_lifespan,
                self.snapshot.stats.carbon_level,
                self.oxygen_level,
                self.snapshot.stats.global_fertility,
            )),
        ];

        Paragraph::new(ratatui::text::Line::from(world_stats))
            .style(Style::default().fg(Color::DarkGray))
            .render(status_lines[2], buf);

        let hive_stats = vec![
            ratatui::text::Span::styled(" 🕸  Hive: ", Style::default().fg(Color::Cyan)),
            ratatui::text::Span::raw(format!(
                "{} Peers | In: {} | Out: {} | Status: ",
                self.peer_count, self.migrations_received, self.migrations_sent
            )),
            if self.is_online {
                ratatui::text::Span::styled("Online", Style::default().fg(Color::Green))
            } else {
                ratatui::text::Span::styled("Offline", Style::default().fg(Color::Red))
            },
        ];

        Paragraph::new(ratatui::text::Line::from(hive_stats)).render(status_lines[4], buf);

        let energy_info = vec![
            ratatui::text::Span::styled("⚡ Energy: ", Style::default().fg(Color::Yellow)),
            ratatui::text::Span::raw(format!("{:.0}", self.available_energy)),
        ];
        Paragraph::new(ratatui::text::Line::from(energy_info))
            .style(Style::default().fg(Color::DarkGray))
            .render(status_lines[3], buf);
        let legend = " [Space] Pause | [z] Cinematic | [M] Mutate | [K] Smite | [P] Reincarnate | [A] Ancestry | [Y] Archeology | [H] Help ";
        Paragraph::new(legend)
            .style(Style::default().fg(Color::DarkGray))
            .render(status_lines[5], buf);
    }
}
