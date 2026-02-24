# Agent Project Memory: primordium_tui

> TUI rendering crate for terminal-based simulation visualization.

## OVERVIEW

Terminal user interface layer using ratatui for real-time simulation rendering and user interaction.

## STRUCTURE

```
src/
├── lib.rs              # Tui struct (init/exit, terminal lifecycle)
├── renderer.rs         # WorldWidget (main world rendering, terrain, entities, bonds)
└── views/
    ├── mod.rs          # Widget exports
    ├── status.rs       # StatusWidget (CPU/RAM gauges, era, stats)
    ├── brain.rs        # BrainWidget (selected entity neural activity)
    ├── ancestry.rs     # AncestryWidget (tree of life visualization)
    ├── archeology.rs   # ArcheologyWidget (fossil record, time travel)
    ├── overlays.rs     # CinematicOverlayWidget, LegendWidget
    ├── sparklines.rs   # SparklinesWidget (population trends)
    ├── hof.rs          # HallOfFameWidget (leaderboard)
    ├── market.rs       # MarketWidget (P2P trading)
    ├── research.rs     # ResearchWidget (lineage traits)
    ├── civilization.rs # CivilizationWidget (outpost networks)
    ├── chronicle.rs    # ChronicleWidget (event log)
    └── help.rs         # HelpWidget (controls reference)
```

## WHERE TO LOOK

- **Main rendering**: `renderer.rs::WorldWidget` - single-pass entity rendering with bond line optimization
- **Widget pattern**: All views implement `ratatui::Widget` trait with `render(self, area, buf)` method
- **Screen/world coords**: `renderer.rs::world_to_screen()` and `screen_to_world()` for coordinate mapping
- **Entity visualization**: `symbol_for_status()` and `color_for_status()` map entity state to glyphs/colors
- **View modes**: `view_mode` parameter (0-7) switches between Normal, Fertility, Social, Rank, Vocal, Market, Research, Civilization overlays

## CONVENTIONS

- **Widget naming**: `{Feature}Widget` struct with `snapshot: &'a WorldSnapshot` field
- **Lifetime parameters**: All widgets use `'a` lifetime for snapshot borrowing
- **Layout**: Use `ratatui::layout::Layout` with `Constraint` for responsive sizing
- **Styling**: `ratatui::style::{Color, Style, Modifier}` for consistent theming
- **Text**: `ratatui::text::{Line, Span}` for rich text formatting
- **Blocks**: `Block::default().title().borders(Borders::ALL)` for widget containers

## ANTI-PATTERNS

- **Direct buffer mutation**: Always use `ratatui` widgets (Paragraph, Gauge, Block) instead of raw buffer writes
- **Hardcoded colors**: Use `Color::Rgb()` for entity-specific colors, predefined colors for UI elements
- **Ignoring screensaver mode**: Check `screensaver` flag to skip borders in fullscreen mode
- **Redundant allocations**: Pre-allocate `Vec`/`HashMap` with capacity when size is known (see bond rendering optimization)
- **Blocking render**: Keep `render()` methods fast - defer heavy computation to snapshot generation in core crate
