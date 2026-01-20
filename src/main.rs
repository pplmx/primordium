use primordium::ui::tui::Tui;
use primordium::app::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut tui = Tui::new()?;
    tui.init()?;

    let mut app = App::new()?;
    let res = app.run(&mut tui).await;

    tui.exit()?;
    
    if let Err(e) = res {
        eprintln!("Application error: {e}");
    } else {
        println!("Exited clean.");
    }
    
    Ok(())
}
