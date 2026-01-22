use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub value: f64,
    pub symbol: char,
    pub color: Color,
}

impl Food {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            value: 50.0,
            symbol: '*',
            color: Color::Green,
        }
    }
}
