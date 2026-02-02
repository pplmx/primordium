use crate::app::state::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

impl App {
    pub fn render_help(&self, f: &mut Frame) {
        if !self.show_help {
            return;
        }

        let area = f.area();
        let help_width = 60.min(area.width - 4);
        let help_height = 20.min(area.height - 4);
        let help_area = Rect::new(
            (area.width - help_width) / 2,
            (area.height - help_height) / 2,
            help_width,
            help_height,
        );
        f.render_widget(Clear, help_area);

        // Tab titles
        let tab_titles = [
            "[1]Controls",
            "[2]Symbols",
            "[3]Concepts",
            "[4]Eras",
            "[5]Visuals",
            "[6]Research",
            "[7]Civ",
        ];
        let mut tab_spans = Vec::new();
        for (i, title) in tab_titles.iter().enumerate() {
            if i == self.help_tab as usize {
                tab_spans.push(ratatui::text::Span::styled(
                    format!(" {} ", title),
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                ));
            } else {
                tab_spans.push(ratatui::text::Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Content based on tab
        let help_content: Vec<&str> = match self.help_tab {
            0 => vec![
                "",
                " ⌨️  KEYBOARD CONTROLS",
                " ─────────────────────────────────",
                " [Q]       Quit simulation",
                " [Space]   Pause / Resume",
                " [z]       Toggle Cinematic Mode",
                " [B]       Toggle Brain panel",
                " [A]       Toggle Ancestry Tree",
                " [Y]       Toggle Archeology Tool",
                " [+/-]     Speed up / Slow down (or edit gene)",
                " [[/]]     Archeology Seek (Time)",
                " [↑/↓]     Fossil Select (in Archeology)",
                " [G]       Resurrect Fossil (Cloning)",
                " [1-8]     Switch View modes",
                " [J]       Toggle Social Brush (Peace/War)",
                " [H]       Toggle this Help",
                " [X]       Genetic Surge (mutate all)",
                " [C]       Export selected DNA",
                " [V]       Import DNA from file",
                "",
                " 🧬 GENETIC ENGINEERING",
                " ─────────────────────────────────",
                " [Click] Gene label in sidebar to focus",
                " [+/-]   Increment/Decrement focused gene",
                "",
                " ⚡ DIVINE INTERVENTION (Targeted)",
                " ─────────────────────────────────",
                " [M] Mutate  [K] Smite  [P] Reincarnate",
                "",
                " 🛠️  DIVINE EDITOR (Brush)",
                " ─────────────────────────────────",
                " [!] Plains  [@] Mountain  [#] River",
                " [$] Oasis   [%] Wall      [^] Barren",
                "",
                " 🖱️  MOUSE CONTROLS",
                " ─────────────────────────────────",
                " Left Click   Select entity",
                " Left Drag    Paint Terrain",
                " Right Click  Spawn food cluster",
            ],

            1 => vec![
                "",
                " 🧬 ENTITY STATUS SYMBOLS",
                " ─────────────────────────────────",
                " ●  Foraging  - Normal behavior",
                " ♦  Hunting   - Attacking others",
                " ♥  Mating    - Ready to reproduce",
                " †  Starving  - Energy < 20%",
                " ◦  Juvenile  - Too young to mate",
                " ☣  Infected  - Carrying pathogen",
                " ♣  Sharing   - Giving energy",
                " ⚭  Bonded    - Symbiotic pairing",
                "",
                " 🗺️  TERRAIN TYPES",
                " ─────────────────────────────────",
                " ▲  Mountain  - Slow movement",
                " ≈  River     - Fast movement",
                " ♠  Forest    - High food, CO2 sink",
                " ▒  Desert    - Low food, Heat stress",
                " ◊  Oasis     - 3x food spawn",
                " ░  Barren    - No food growth",
                " █  Wall      - Impassable barrier",
                " *  Food      - Energy source",
            ],
            2 => vec![
                "",
                " 🔗 HARDWARE COUPLING",
                " ─────────────────────────────────",
                " Your CPU load = World climate",
                "   Low CPU  → Temperate (×1.0)",
                "   High CPU → Scorching (×3.0)",
                "",
                " Your RAM usage = Resource scarcity",
                "   Low RAM  → Abundant food",
                "   High RAM → Famine conditions",
                "",
                " 🧠 NEURAL EVOLUTION",
                " ─────────────────────────────────",
                " Each entity has a neural network",
                " that evolves through reproduction.",
                " Fittest organisms survive longer!",
            ],
            3 => vec![
                "",
                " 📜 WORLD ERAS",
                " ─────────────────────────────────",
                " 🌀 Primordial  - Chaos adaptation",
                "",
                " 🌱 DawnOfLife  - Stability or",
                "    High Herbivore Biomass",
                "",
                " 🌸 Flourishing - Biodiversity",
                "    hotspots and healthy pop",
                "",
                " ⚔️  DominanceWar - High Carbon",
                "    or Predator dominance",
                "",
                " 👑 ApexEra     - Fitness > 8000",
            ],
            4 => vec![
                "",
                " 👁️  VISUALIZATION MODES [1-8]",
                " ─────────────────────────────────",
                " [1] Normal      - Default view",
                " [2] Fertility   - Soil health (G:Healthy, R:Depleted)",
                " [3] Social      - Peace (B) and War (R) zones",
                " [4] Rank        - Social hierarchy (Purple: High Rank)",
                " [5] Vocal       - Vocal signal propagation (Yellow)",
                " [6] Market      - Multiverse trade offers",
                " [7] Research    - Neural plasticity heatmap",
                " [8] Civilization- Global Dynasty dashboard",
                "",
                " 🪖  SPECIAL INDICATORS",
                " ─────────────────────────────────",
                " ⚔️  Soldier      - High rank + aggressive",
                " Alpha Highlight - Golden aura in Social/Rank views",
                " Soldier Aura    - Dark red aura in Social/Rank views",
            ],
            5 => vec![
                "",
                " 🧪 NEURAL RESEARCH TOOLS",
                " ─────────────────────────────────",
                " Research mode [7] allows you to see the",
                " REAL-TIME synaptic changes in a brain.",
                "",
                " Blue → Cyan → Yellow intensity scale",
                " represents the magnitude of weight delta",
                " (Δw) during Hebbian reinforcement.",
                "",
                " This reveals the learning pathways that",
                " the organism is currently reinforcing.",
            ],
            6 => vec![
                "",
                " 🏛️  CIVILIZATION & DYNASTIES",
                " ─────────────────────────────────",
                " View mode [8] tracks the macro-progress of",
                " the most successful lineages (Dynasties).",
                "",
                " 🏆 Levels: Ownership of outposts and high",
                "    population grants civilization buffs.",
                "",
                " 🧠 Collective Memory: Lineages share a",
                "    shared memory pool for goals & threats.",
                "",
                " 🛡️ Ancestral Traits: Long-lived lineages",
                "    evolve persistent epigenetic bonuses.",
            ],
            _ => vec![""],
        };

        let mut lines: Vec<ratatui::text::Line> = Vec::new();
        lines.push(ratatui::text::Line::from(tab_spans));
        for line in help_content {
            lines.push(ratatui::text::Line::from(line));
        }

        f.render_widget(
            Paragraph::new(lines).block(Block::default().title(" 📖 Help ").borders(Borders::ALL)),
            help_area,
        );
    }
}
