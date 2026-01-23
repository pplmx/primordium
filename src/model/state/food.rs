use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub value: f64,
    pub symbol: char,
    pub color_rgb: (u8, u8, u8),
    /// NEW: Metabolic niche identifier (0.0 to 1.0)
    pub nutrient_type: f32,
}

impl Food {
    pub fn new(x: u16, y: u16, nutrient_type: f32) -> Self {
        let color = if nutrient_type < 0.5 {
            (0, 255, 0) // Green
        } else {
            (0, 100, 255) // Blue
        };

        Self {
            x,
            y,
            value: 50.0,
            symbol: '*',
            color_rgb: color,
            nutrient_type,
        }
    }
}
