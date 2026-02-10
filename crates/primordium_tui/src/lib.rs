pub mod views;

use anyhow::Result;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self { terminal })
    }

    pub fn init(&mut self) -> Result<()> {
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        enable_raw_mode()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        );
    }
}
pub mod renderer;
