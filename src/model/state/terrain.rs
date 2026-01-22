use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Terrain types with distinct environmental effects
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_generate_has_correct_dimensions() {
        let terrain = TerrainGrid::generate(50, 30, 42);
        assert_eq!(terrain.width, 50);
        assert_eq!(terrain.height, 30);
    }

    #[test]
    fn test_terrain_type_movement_modifiers() {
        assert_eq!(TerrainType::Plains.movement_modifier(), 1.0);
        assert_eq!(TerrainType::Mountain.movement_modifier(), 0.5);
        assert_eq!(TerrainType::River.movement_modifier(), 1.5);
        assert_eq!(TerrainType::Wall.movement_modifier(), 0.0);
    }

    #[test]
    fn test_terrain_type_food_spawn_modifiers() {
        assert_eq!(TerrainType::Plains.food_spawn_modifier(), 1.0);
        assert_eq!(TerrainType::Oasis.food_spawn_modifier(), 3.0);
        assert_eq!(TerrainType::Mountain.food_spawn_modifier(), 0.0);
        assert_eq!(TerrainType::Wall.food_spawn_modifier(), 0.0);
    }

    #[test]
    fn test_terrain_dust_bowl_trigger() {
        let mut terrain = TerrainGrid::generate(10, 10, 42);
        assert_eq!(terrain.dust_bowl_timer, 0);

        terrain.trigger_dust_bowl(500);
        assert_eq!(terrain.dust_bowl_timer, 500);
    }

    #[test]
    fn test_terrain_deplete_and_recover() {
        let mut terrain = TerrainGrid::generate(10, 10, 42);
        let initial_fertility = terrain.get_cell(5, 5).fertility;

        // Deplete fertility
        terrain.deplete(5.0, 5.0, 0.5);
        let depleted_fertility = terrain.get_cell(5, 5).fertility;
        assert!(
            depleted_fertility < initial_fertility,
            "Fertility should decrease after depletion"
        );

        // Update to recover (slowly)
        for _ in 0..100 {
            terrain.update();
        }
        let recovered_fertility = terrain.get_cell(5, 5).fertility;
        assert!(
            recovered_fertility > depleted_fertility,
            "Fertility should recover over time"
        );
    }

    #[test]
    fn test_terrain_get_boundary_safety() {
        let terrain = TerrainGrid::generate(10, 10, 42);

        // Should not panic on out-of-bounds access
        let _ = terrain.get(100.0, 100.0);
        let _ = terrain.get(-5.0, -5.0);
        let _ = terrain.get_cell(100, 100);
    }

    #[test]
    fn test_terrain_set_cell_type() {
        let mut terrain = TerrainGrid::generate(10, 10, 42);
        terrain.set_cell_type(5, 5, TerrainType::Wall);
        assert_eq!(terrain.get_cell(5, 5).terrain_type, TerrainType::Wall);
    }
}
