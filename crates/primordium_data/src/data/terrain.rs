use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum TerrainType {
    #[default]
    Plains,
    Mountain,
    River,
    Oasis,
    Barren,
    Wall,
    Forest,
    Desert,
    Nest,
    Outpost,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum OutpostSpecialization {
    #[default]
    Standard,
    Silo,
    Nursery,
}

#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Food {
    pub x: u16,
    pub y: u16,
    pub value: f64,
    pub symbol: char,
    pub color_rgb: (u8, u8, u8),
    pub nutrient_type: f32,
}

impl Food {
    #[must_use]
    pub fn new(x: u16, y: u16, nutrient_type: f32) -> Self {
        let color = if nutrient_type < 0.5 {
            (0, 255, 0)
        } else {
            (0, 100, 255)
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
