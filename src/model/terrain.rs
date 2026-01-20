use rand::Rng;
use ratatui::style::Color;

/// Terrain types with distinct environmental effects
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TerrainType {
    #[default]
    Plains,   // Default terrain, no modifiers
    Mountain, // Slows movement by 50%, no food spawns
    River,    // Speeds movement by 50%, normal food
    Oasis,    // Normal movement, food spawn ×3
}

impl TerrainType {
    /// Movement speed multiplier for this terrain type
    pub fn movement_modifier(&self) -> f64 {
        match self {
            TerrainType::Plains => 1.0,
            TerrainType::Mountain => 0.5,
            TerrainType::River => 1.5,
            TerrainType::Oasis => 1.0,
        }
    }

    /// Food spawn probability multiplier
    pub fn food_spawn_modifier(&self) -> f64 {
        match self {
            TerrainType::Plains => 1.0,
            TerrainType::Mountain => 0.0, // No food on mountains
            TerrainType::River => 0.8,
            TerrainType::Oasis => 3.0,    // 3x food in oases
        }
    }

    /// Symbol for rendering in TUI
    pub fn symbol(&self) -> char {
        match self {
            TerrainType::Plains => ' ',
            TerrainType::Mountain => '▲',
            TerrainType::River => '≈',
            TerrainType::Oasis => '◊',
        }
    }

    /// Color for rendering in TUI
    pub fn color(&self) -> Color {
        match self {
            TerrainType::Plains => Color::Reset,
            TerrainType::Mountain => Color::Rgb(100, 100, 100),  // Gray
            TerrainType::River => Color::Rgb(70, 130, 180),      // Steel Blue
            TerrainType::Oasis => Color::Rgb(50, 205, 50),       // Lime Green
        }
    }
}

/// A single cell in the terrain grid
#[derive(Debug, Clone, Copy)]
pub struct TerrainCell {
    pub terrain_type: TerrainType,
    pub elevation: f32, // 0.0 - 1.0, used for generation
}

impl Default for TerrainCell {
    fn default() -> Self {
        Self {
            terrain_type: TerrainType::Plains,
            elevation: 0.5,
        }
    }
}

/// Grid-based terrain map for the world
pub struct TerrainGrid {
    cells: Vec<Vec<TerrainCell>>,
    pub width: u16,
    pub height: u16,
}

impl TerrainGrid {
    /// Generate a new terrain grid with procedural terrain
    pub fn generate(width: u16, height: u16, seed: u64) -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = vec![vec![TerrainCell::default(); width as usize]; height as usize];

        // Simple noise-based terrain generation
        // Phase 1: Generate elevation map using value noise
        for y in 0..height as usize {
            for x in 0..width as usize {
                // Simple pseudo-random elevation based on position and seed
                let noise = Self::value_noise(x as f32, y as f32, seed);
                cells[y][x].elevation = noise;
            }
        }

        // Phase 2: Assign terrain types based on elevation
        let mountain_threshold = 0.7;
        let river_threshold = 0.25;

        for y in 0..height as usize {
            for x in 0..width as usize {
                let elevation = cells[y][x].elevation;

                if elevation > mountain_threshold {
                    cells[y][x].terrain_type = TerrainType::Mountain;
                } else if elevation < river_threshold {
                    cells[y][x].terrain_type = TerrainType::River;
                }
            }
        }

        // Phase 3: Scatter oases in plains areas
        let oasis_count = ((width as usize * height as usize) / 200).max(3);
        let mut placed = 0;
        let mut attempts = 0;

        while placed < oasis_count && attempts < oasis_count * 10 {
            let x = rng.gen_range(0..width as usize);
            let y = rng.gen_range(0..height as usize);

            if cells[y][x].terrain_type == TerrainType::Plains {
                cells[y][x].terrain_type = TerrainType::Oasis;
                placed += 1;
            }
            attempts += 1;
        }

        Self { cells, width, height }
    }

    /// Simple value noise function for terrain generation
    fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
        // Use multiple octaves of noise for more natural terrain
        let scale1 = 0.1;
        let scale2 = 0.05;
        let scale3 = 0.02;

        let noise1 = Self::hash_noise(x * scale1, y * scale1, seed) * 0.5;
        let noise2 = Self::hash_noise(x * scale2, y * scale2, seed.wrapping_add(1)) * 0.3;
        let noise3 = Self::hash_noise(x * scale3, y * scale3, seed.wrapping_add(2)) * 0.2;

        (noise1 + noise2 + noise3).clamp(0.0, 1.0)
    }

    /// Hash-based noise for deterministic pseudo-random values
    fn hash_noise(x: f32, y: f32, seed: u64) -> f32 {
        let ix = x.floor() as i32;
        let iy = y.floor() as i32;
        let fx = x - x.floor();
        let fy = y - y.floor();

        // Smooth interpolation
        let ux = fx * fx * (3.0 - 2.0 * fx);
        let uy = fy * fy * (3.0 - 2.0 * fy);

        // Get corner values
        let v00 = Self::hash(ix, iy, seed);
        let v10 = Self::hash(ix + 1, iy, seed);
        let v01 = Self::hash(ix, iy + 1, seed);
        let v11 = Self::hash(ix + 1, iy + 1, seed);

        // Bilinear interpolation
        let v0 = v00 + ux * (v10 - v00);
        let v1 = v01 + ux * (v11 - v01);

        v0 + uy * (v1 - v0)
    }

    /// Simple hash function to generate pseudo-random values
    fn hash(x: i32, y: i32, seed: u64) -> f32 {
        let n = (x.wrapping_mul(127) ^ y.wrapping_mul(311)) as u64 ^ seed;
        let n = n.wrapping_mul(0x517cc1b727220a95);
        let n = n ^ (n >> 32);
        (n & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }

    /// Get terrain at a specific world position
    pub fn get(&self, x: f64, y: f64) -> &TerrainCell {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        &self.cells[iy][ix]
    }

    /// Get movement modifier at position
    pub fn movement_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.movement_modifier()
    }

    /// Get food spawn modifier at position
    pub fn food_spawn_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.food_spawn_modifier()
    }

    /// Get terrain cell for rendering (by integer coordinates)
    pub fn get_cell(&self, x: u16, y: u16) -> &TerrainCell {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        &self.cells[iy][ix]
    }
}
