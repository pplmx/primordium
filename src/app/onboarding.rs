use crate::app::state::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

impl App {
    pub fn render_onboarding(&self, f: &mut Frame) {
        let step = match self.onboarding_step {
            Some(s) => s,
            None => return,
        };

        let area = f.area();
        let modal_width = 55.min(area.width - 4);
        let modal_height = 16.min(area.height - 4);
        let modal_area = Rect::new(
            (area.width - modal_width) / 2,
            (area.height - modal_height) / 2,
            modal_width,
            modal_height,
        );
        f.render_widget(Clear, modal_area);

        let (title, content): (&str, Vec<&str>) = match step {
            0 => (
                " 🌱 Welcome to Primordium! (1/3) ",
                vec![
                    "",
                    " This is a living ecosystem simulation",
                    " where digital organisms EVOLVE!",
                    "",
                    " 🔗 YOUR HARDWARE = THEIR WORLD",
                    " ─────────────────────────────────",
                    " • High CPU load → Hot climate",
                    "   (organisms burn energy faster)",
                    "",
                    " • High RAM usage → Food scarcity",
                    "   (less food spawns)",
                    "",
                    "",
                    " Press [Enter] to continue...",
                ],
            ),
            1 => (
                " 🧬 Understanding Life (2/3) ",
                vec![
                    "",
                    " Each organism has a NEURAL NETWORK",
                    " brain that evolves through survival!",
                    "",
                    " 📊 VISUAL LANGUAGE",
                    " ─────────────────────────────────",
                    " ●  Adult Worker (Green)     ·  Larva",
                    " ▲  Adult Soldier (Red)      △  Larva",
                    " ◈  Adult Engineer (Cyan)    ◇  Larva",
                    " ◎  Adult Provider (Yellow)  ○  Larva",
                    " †  Starving (Urgent Red)",
                    "",
                    " Press [Enter] to continue...",
                ],
            ),
            _ => (
                " 🎮 Controls (3/3) ",
                vec![
                    "",
                    " KEYBOARD",
                    " ─────────────────────────────────",
                    " [Space]  Pause / Resume",
                    " [B]      Show brain panel",
                    " [H]      Open full help guide",
                    " [X]      Trigger genetic surge",
                    "",
                    " MOUSE",
                    " ─────────────────────────────────",
                    " Left Click   Select organism",
                    " Right Click  Spawn food",
                    "",
                    " Press [Enter] to START!",
                ],
            ),
        };

        let lines: Vec<ratatui::text::Line> = content
            .iter()
            .map(|s| ratatui::text::Line::from(*s))
            .collect();

        f.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            ),
            modal_area,
        );
    }
}
