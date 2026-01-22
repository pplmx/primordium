use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub value: f64,
    pub symbol: char,
    pub color_rgb: (u8, u8, u8),
}

impl Food {
    pub fn new(x: u16, y: u16) -> Self {
        Self {
            x,
            y,
            value: 50.0,
            symbol: '*',
            color_rgb: (0, 255, 0), // Green
        }
    }
}
