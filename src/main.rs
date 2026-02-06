use anyhow::Result;
use clap::Parser;
use primordium_lib::app::App;
use primordium_lib::ui::tui::Tui;

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

    /// Run in benchmark mode (headless, fixed ticks)
    #[arg(long)]
    benchmark: bool,

    #[arg(long)]
    relay: Option<String>,

    #[arg(long)]
    replay: Option<String>,
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

    if args.benchmark {
        println!("Running in BENCHMARK mode (500 ticks)...");
        let mut app = App::new()?;
        let start = std::time::Instant::now();
        for _ in 0..500 {
            if let Err(e) = app.world.update(&mut app.env) {
                eprintln!("Sim error: {e}");
                break;
            }
        }
        let dur = start.elapsed();
        println!(
            "Benchmark complete: 500 ticks in {:.2?} ({:.2} TPS)",
            dur,
            500.0 / dur.as_secs_f64()
        );
        return Ok(());
    }

    match args.mode {
        Mode::Headless => {
            println!("Running in HEADLESS mode...");
            let mut app = App::new()?;
            if let Some(url) = args.relay {
                println!("Connecting to relay: {}...", url);
                app.connect(&url);
            }
            while app.running {
                // Background tasks might need a small sleep to not peg CPU at 100% if sim is fast
                // But for experiments, we want it fast.
                if let Err(e) = app.world.update(&mut app.env) {
                    eprintln!("Sim error: {e}");
                    break;
                }
                // Periodic system poll (mocked or reduced frequency in headless)
                // ... logic to handle headless termination etc.
                if app.world.get_population_count() == 0 {
                    break;
                }
            }
            println!("Headless simulation finished.");
        }
        _ => {
            let mut tui = Tui::new()?;
            tui.init()?;

            let mut app = App::new()?;

            if let Some(url) = args.relay {
                app.connect(&url);
            }

            if let Some(path) = args.replay {
                if let Err(e) = app.load_replay(&path) {
                    eprintln!("Failed to load replay: {}", e);
                } else {
                    println!("Replay loaded from {}", path);
                }
            }

            // Override game mode from CLI
            match args.gamemode.to_lowercase().as_str() {
                "coop" | "cooperative" => {
                    app.world.config.game_mode =
                        primordium_lib::model::config::GameMode::Cooperative
                }
                "battle" | "battleroyale" => {
                    app.world.config.game_mode =
                        primordium_lib::model::config::GameMode::BattleRoyale
                }
                _ => {}
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
