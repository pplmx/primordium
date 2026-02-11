use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

/// Terrain type for world cells.
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
    /// Default grassy plains.
    #[default]
    Plains,
    /// Slow-moving mountain terrain.
    Mountain,
    /// Fast-moving river with hydration.
    River,
    /// Food-rich oasis.
    Oasis,
    /// Barren wasteland.
    Barren,
    /// Impassable wall.
    Wall,
    /// Forest with carbon sequestration.
    Forest,
    /// Hot desert terrain.
    Desert,
    /// Protective nest structure.
    Nest,
    /// Advanced outpost structure.
    Outpost,
}

/// Specialization type for outpost structures.
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
    /// Standard outpost.
    #[default]
    Standard,
    /// Energy storage silo.
    Silo,
    /// Offspring nursery.
    Nursery,
}

/// Food resource in the world.
#[derive(Clone, Debug, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct Food {
    /// X grid coordinate.
    pub x: u16,
    /// Y grid coordinate.
    pub y: u16,
    /// Energy value when consumed.
    pub value: f64,
    /// Display symbol.
    pub symbol: char,
    /// RGB color tuple.
    pub color_rgb: (u8, u8, u8),
    /// Nutrient type (0.0=green, 1.0=blue).
    pub nutrient_type: f32,
}

impl Food {
    /// Create new food at position with nutrient type.
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
