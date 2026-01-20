use anyhow::Result;
use primordium::app::App;
use primordium::ui::tui::Tui;

fn main() -> Result<()> {
    let mut tui = Tui::new()?;
    tui.init()?;

    let mut app = App::new()?;
    let res = app.run(&mut tui);

    tui.exit()?;

    if let Err(e) = res {
        eprintln!("Application error: {e}");
    } else {
        println!("Exited clean.");
    }

    Ok(())
}
