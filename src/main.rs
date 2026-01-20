use anyhow::Result;
use clap::Parser;
use primordium::app::App;
use primordium::ui::tui::Tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mode to run the simulation in
    #[arg(short, long, value_enum, default_value = "standard")]
    mode: Mode,

    /// Custom config file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Game rules mode (Standard, Cooperative, BattleRoyale)
    #[arg(long, default_value = "standard")]
    gamemode: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Standard,
    Screensaver,
    Headless,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.mode {
        Mode::Headless => {
            println!("Running in HEADLESS mode...");
            let mut app = App::new()?;
            // No TUI, just loop
            while app.running {
                // Background tasks might need a small sleep to not peg CPU at 100% if sim is fast
                // But for experiments, we want it fast.
                if let Err(e) = app.world.update(&app.env) {
                    eprintln!("Sim error: {e}");
                    break;
                }
                // Periodic system poll (mocked or reduced frequency in headless)
                // ... logic to handle headless termination etc.
                if app.world.entities.is_empty() {
                    break;
                }
            }
            println!("Headless simulation finished.");
        }
        _ => {
            let mut tui = Tui::new()?;
            tui.init()?;

            let mut app = App::new()?;

            // Override game mode from CLI
            match args.gamemode.to_lowercase().as_str() {
                "coop" | "cooperative" => app.world.config.game_mode = primordium::model::config::GameMode::Cooperative,
                "battle" | "battleroyale" => app.world.config.game_mode = primordium::model::config::GameMode::BattleRoyale,
                _ => {},
            }
            if matches!(args.mode, Mode::Screensaver) {
                app.screensaver = true;
            }

            let res = app.run(&mut tui).await;

            tui.exit()?;

            if let Err(e) = res {
                eprintln!("Application error: {e}");
            } else {
                println!("Exited clean.");
            }
        }
    }

    Ok(())
}
