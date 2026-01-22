use rand::Rng;
use ratatui::style::Color;

/// Terrain types with distinct environmental effects
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TerrainType {
    #[default]
    Plains,
    Mountain,
    River,
    Oasis,
    Barren,
    Wall,
}

impl TerrainType {
    pub fn movement_modifier(&self) -> f64 {
        match self {
            TerrainType::Plains => 1.0,
            TerrainType::Mountain => 0.5,
            TerrainType::River => 1.5,
            TerrainType::Oasis => 1.0,
            TerrainType::Barren => 0.7,
            TerrainType::Wall => 0.0,
        }
    }

    pub fn food_spawn_modifier(&self) -> f64 {
        match self {
            TerrainType::Plains => 1.0,
            TerrainType::Mountain => 0.0,
            TerrainType::River => 0.8,
            TerrainType::Oasis => 3.0,
            TerrainType::Barren => 0.1,
            TerrainType::Wall => 0.0,
        }
    }

    pub fn symbol(&self) -> char {
        match self {
            TerrainType::Plains => ' ',
            TerrainType::Mountain => '▲',
            TerrainType::River => '≈',
            TerrainType::Oasis => '◊',
            TerrainType::Barren => '░',
            TerrainType::Wall => '█',
        }
    }

    pub fn color(&self) -> Color {
        match self {
            TerrainType::Plains => Color::Reset,
            TerrainType::Mountain => Color::Rgb(100, 100, 100),
            TerrainType::River => Color::Rgb(70, 130, 180),
            TerrainType::Oasis => Color::Rgb(50, 205, 50),
            TerrainType::Barren => Color::Rgb(139, 69, 19),
            TerrainType::Wall => Color::Rgb(60, 60, 60),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TerrainCell {
    pub terrain_type: TerrainType,
    pub original_type: TerrainType,
    pub elevation: f32,
    pub fertility: f32,
}

impl Default for TerrainCell {
    fn default() -> Self {
        Self {
            terrain_type: TerrainType::Plains,
            original_type: TerrainType::Plains,
            elevation: 0.5,
            fertility: 1.0,
        }
    }
}

pub struct TerrainGrid {
    cells: Vec<Vec<TerrainCell>>,
    pub width: u16,
    pub height: u16,
    pub dust_bowl_timer: u32,
}

impl TerrainGrid {
    pub fn generate(width: u16, height: u16, seed: u64) -> Self {
        let mut rng = rand::thread_rng();
        let mut cells = vec![vec![TerrainCell::default(); width as usize]; height as usize];

        for (y, row) in cells.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let noise = Self::value_noise(x as f32, y as f32, seed);
                cell.elevation = noise;
            }
        }

        let mountain_threshold = 0.7;
        let river_threshold = 0.25;

        for row in &mut cells {
            for cell in row {
                if cell.elevation > mountain_threshold {
                    cell.terrain_type = TerrainType::Mountain;
                    cell.original_type = TerrainType::Mountain;
                } else if cell.elevation < river_threshold {
                    cell.terrain_type = TerrainType::River;
                    cell.original_type = TerrainType::River;
                }
            }
        }

        let oasis_count = ((width as usize * height as usize) / 200).max(3);
        let rock_count = ((width as usize * height as usize) / 150).max(5);

        let mut placed = 0;
        let mut attempts = 0;
        while placed < oasis_count && attempts < oasis_count * 10 {
            let x = rng.gen_range(0..width as usize);
            let y = rng.gen_range(0..height as usize);
            if cells[y][x].terrain_type == TerrainType::Plains {
                cells[y][x].terrain_type = TerrainType::Oasis;
                cells[y][x].original_type = TerrainType::Oasis;
                placed += 1;
            }
            attempts += 1;
        }

        placed = 0;
        attempts = 0;
        while placed < rock_count && attempts < rock_count * 10 {
            let x = rng.gen_range(0..width as usize);
            let y = rng.gen_range(0..height as usize);
            if cells[y][x].terrain_type == TerrainType::Plains {
                cells[y][x].terrain_type = TerrainType::Wall;
                cells[y][x].original_type = TerrainType::Wall;
                placed += 1;
            }
            attempts += 1;
        }

        Self {
            cells,
            width,
            height,
            dust_bowl_timer: 0,
        }
    }

    pub fn update(&mut self) {
        if self.dust_bowl_timer > 0 {
            self.dust_bowl_timer -= 1;
        }
        for row in &mut self.cells {
            for cell in row {
                cell.fertility = (cell.fertility + 0.001).min(1.0);
                if self.dust_bowl_timer > 0 && cell.terrain_type == TerrainType::Plains {
                    cell.terrain_type = TerrainType::Barren;
                    cell.fertility = (cell.fertility - 0.05).max(0.0);
                }
                if cell.terrain_type != TerrainType::Mountain
                    && cell.terrain_type != TerrainType::River
                    && cell.terrain_type != TerrainType::Wall
                {
                    if cell.fertility < 0.15 {
                        cell.terrain_type = TerrainType::Barren;
                    } else if cell.terrain_type == TerrainType::Barren && cell.fertility > 0.4 {
                        cell.terrain_type = cell.original_type;
                    }
                }
            }
        }
    }

    pub fn trigger_dust_bowl(&mut self, duration: u32) {
        self.dust_bowl_timer = duration;
    }

    pub fn deplete(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].fertility = (self.cells[iy][ix].fertility - amount).max(0.0);
    }

    fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
        let scale1 = 0.1;
        let scale2 = 0.05;
        let scale3 = 0.02;
        let noise1 = Self::hash_noise(x * scale1, y * scale1, seed) * 0.5;
        let noise2 = Self::hash_noise(x * scale2, y * scale2, seed.wrapping_add(1)) * 0.3;
        let noise3 = Self::hash_noise(x * scale3, y * scale3, seed.wrapping_add(2)) * 0.2;
        (noise1 + noise2 + noise3).clamp(0.0, 1.0)
    }

    fn hash_noise(x: f32, y: f32, seed: u64) -> f32 {
        let ix = x.floor() as i32;
        let iy = y.floor() as i32;
        let fx = x - x.floor();
        let fy = y - y.floor();
        let ux = fx * fx * (3.0 - 2.0 * fx);
        let uy = fy * fy * (3.0 - 2.0 * fy);
        let v00 = Self::hash(ix, iy, seed);
        let v10 = Self::hash(ix + 1, iy, seed);
        let v01 = Self::hash(ix, iy + 1, seed);
        let v11 = Self::hash(ix + 1, iy + 1, seed);
        let v0 = v00 + ux * (v10 - v00);
        let v1 = v01 + ux * (v11 - v01);
        v0 + uy * (v1 - v0)
    }

    fn hash(x: i32, y: i32, seed: u64) -> f32 {
        let n = (x.wrapping_mul(127) ^ y.wrapping_mul(311)) as u64 ^ seed;
        let n = n.wrapping_mul(0x517cc1b727220a95);
        let n = n ^ (n >> 32);
        (n & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }

    pub fn get(&self, x: f64, y: f64) -> &TerrainCell {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        &self.cells[iy][ix]
    }

    pub fn movement_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.movement_modifier()
    }
    pub fn food_spawn_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.food_spawn_modifier()
    }
    pub fn get_cell(&self, x: u16, y: u16) -> &TerrainCell {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        &self.cells[iy][ix]
    }

    /// Manually set cell type (useful for testing and disasters)
    pub fn set_cell_type(&mut self, x: u16, y: u16, t: TerrainType) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].terrain_type = t;
    }
}
