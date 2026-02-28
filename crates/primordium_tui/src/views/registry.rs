//! Registry Widget - Phase 70 Galactic Federation Marketplace UI
//!
//! Displays the Registry marketplace including:
//! - Hall of Fame (top lineages by civilization level)
//! - Genome marketplace (browse/submit genomes)
//! - Seed marketplace (browse/submit simulation configs)

use ratatui::prelude::*;
use ratatui::widgets::*;

/// A genome record from the marketplace.
#[derive(Debug, Clone)]
pub struct GenomeRecord {
    pub id: String,
    pub lineage_id: Option<String>,
    pub genotype: String,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub fitness_score: f64,
    pub offspring_count: u32,
    pub tick: u64,
    pub downloads: u32,
    pub created_at: String,
}

/// A seed (simulation config) record from the marketplace.
#[derive(Debug, Clone)]
pub struct SeedRecord {
    pub id: String,
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: String,
    pub config_json: String,
    pub avg_tick_time: f64,
    pub max_pop: u32,
    pub performance_summary: String,
    pub downloads: u32,
    pub created_at: String,
}

/// Hall of Fame entry.
#[derive(Debug, Clone)]
pub struct HallOfFameEntry {
    pub id: String,
    pub civilization_level: u32,
    pub is_extinct: bool,
}

/// Registry connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Registry data to display.
pub struct RegistryWidget<'a> {
    /// Current tab: 0 = Hall of Fame, 1 = Genomes, 2 = Seeds
    pub tab: u8,
    /// Hall of Fame entries
    pub hall_of_fame: &'a [HallOfFameEntry],
    /// Genome marketplace entries
    pub genomes: &'a [GenomeRecord],
    /// Seed marketplace entries
    pub seeds: &'a [SeedRecord],
    /// Selected index in current list
    pub selected_index: usize,
    /// Connection status
    pub status: &'a RegistryStatus,
}

impl<'a> Widget for RegistryWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let tab_names = ["🏆 Hall of Fame", "🧬 Genomes", "🌱 Seeds"];
        let mut tab_line = String::new();
        for (i, name) in tab_names.iter().enumerate() {
            if i > 0 {
                tab_line.push_str(" | ");
            }
            if i as u8 == self.tab {
                tab_line.push_str(&format!("[{}]", name));
            } else {
                tab_line.push_str(&format!(" {}", name));
            }
        }

        let title = Paragraph::new(tab_line)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🌌 Galactic Federation "),
            );
        title.render(chunks[0], buf);

        let content_area = chunks[1];
        match self.tab {
            0 => self.render_hall_of_fame(content_area, buf),
            1 => self.render_genomes(content_area, buf),
            2 => self.render_seeds(content_area, buf),
            _ => {}
        }
    }
}

impl<'a> RegistryWidget<'a> {
    fn render_hall_of_fame(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let inner = Block::default()
            .borders(Borders::ALL)
            .title(" Top Lineages ")
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = inner.inner(area);
        inner.render(area, buf);

        if self.hall_of_fame.is_empty() {
            let empty = Paragraph::new("No Hall of Fame data. Connect to a Registry server.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default());
            empty.render(inner_area, buf);
            return;
        }

        let mut lines = Vec::new();
        for (i, entry) in self.hall_of_fame.iter().enumerate().take(15) {
            let prefix = if i == self.selected_index {
                "► "
            } else {
                "  "
            };
            let status = if entry.is_extinct { "💀" } else { "👑" };
            let line = format!(
                "{}{} Level {} - {}",
                prefix,
                status,
                entry.civilization_level,
                &entry.id[..8]
            );
            let style = if i == self.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(line, style)));
        }

        let list = Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true });
        list.render(inner_area, buf);
    }

    fn render_genomes(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let inner = Block::default()
            .borders(Borders::ALL)
            .title(" Genome Marketplace ")
            .border_style(Style::default().fg(Color::Magenta));

        let inner_area = inner.inner(area);
        inner.render(area, buf);

        if self.genomes.is_empty() {
            let empty = Paragraph::new("No genomes available.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default());
            empty.render(inner_area, buf);
            return;
        }

        let mut lines = Vec::new();
        for (i, genome) in self.genomes.iter().enumerate().take(12) {
            let prefix = if i == self.selected_index {
                "► "
            } else {
                "  "
            };
            let line = format!(
                "{}{} - Fitness: {:.1} | By: {}",
                prefix, genome.name, genome.fitness_score, genome.author
            );
            let style = if i == self.selected_index {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(line, style)));
        }

        if self.selected_index < self.genomes.len() {
            let genome = &self.genomes[self.selected_index];
            lines.push(Line::from(""));
            let desc = if genome.description.len() > 50 {
                format!("{}...", &genome.description[..50])
            } else {
                genome.description.clone()
            };
            lines.push(Line::from(Span::styled(
                format!("Desc: {}", desc),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let list = Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true });
        list.render(inner_area, buf);
    }

    fn render_seeds(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let inner = Block::default()
            .borders(Borders::ALL)
            .title(" Seed Marketplace ")
            .border_style(Style::default().fg(Color::Green));

        let inner_area = inner.inner(area);
        inner.render(area, buf);

        if self.seeds.is_empty() {
            let empty = Paragraph::new("No seeds available.")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center)
                .block(Block::default());
            empty.render(inner_area, buf);
            return;
        }

        let mut lines = Vec::new();
        for (i, seed) in self.seeds.iter().enumerate().take(12) {
            let prefix = if i == self.selected_index {
                "► "
            } else {
                "  "
            };
            let line = format!(
                "{}{} - Max Pop: {} | By: {}",
                prefix, seed.name, seed.max_pop, seed.author
            );
            let style = if i == self.selected_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(line, style)));
        }

        if self.selected_index < self.seeds.len() {
            let seed = &self.seeds[self.selected_index];
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    "Perf: {} | Tick: {:.2}ms",
                    seed.performance_summary, seed.avg_tick_time
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let list = Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true });
        list.render(inner_area, buf);
    }
}

/// Registry view data bundle (to reduce function arguments).
pub struct RegistryViewData<'a> {
    pub tab: u8,
    pub hall_of_fame: &'a [HallOfFameEntry],
    pub genomes: &'a [GenomeRecord],
    pub seeds: &'a [SeedRecord],
    pub selected_index: usize,
    pub status: &'a RegistryStatus,
}

/// Draw the Registry panel in the sidebar.
pub fn draw_registry(f: &mut Frame, data: &RegistryViewData, area: Rect) {
    let widget = RegistryWidget {
        tab: data.tab,
        hall_of_fame: data.hall_of_fame,
        genomes: data.genomes,
        seeds: data.seeds,
        selected_index: data.selected_index,
        status: data.status,
    };
    f.render_widget(widget, area);
}
