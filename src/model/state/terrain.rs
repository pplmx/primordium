use rand::Rng;
use ratatui::style::Color;
use rayon::prelude::*;
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
    Forest,
    Desert,
    Nest,
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
            TerrainType::Forest => 0.7,
            TerrainType::Desert => 1.2,
            TerrainType::Nest => 0.8,
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
            TerrainType::Forest => 2.0,
            TerrainType::Desert => 0.3,
            TerrainType::Nest => 0.5,
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
            TerrainType::Forest => '♠',
            TerrainType::Desert => '▒',
            TerrainType::Nest => 'Ω',
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
            TerrainType::Forest => Color::Rgb(34, 139, 34),
            TerrainType::Desert => Color::Rgb(210, 180, 140),
            TerrainType::Nest => Color::Rgb(255, 215, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainCell {
    pub terrain_type: TerrainType,
    pub original_type: TerrainType,
    pub elevation: f32,
    pub fertility: f32,
    /// NEW: Resistance to type change (0.0 to 1.0)
    pub stability: f32,
    /// NEW: Accumulated biomass from entities (triggers Forest transition)
    pub biomass_accumulation: f32,
    /// NEW: Local plant biomass density (0.0 to 100.0)
    pub plant_biomass: f32,
}

impl Default for TerrainCell {
    fn default() -> Self {
        Self {
            terrain_type: TerrainType::Plains,
            original_type: TerrainType::Plains,
            elevation: 0.5,
            fertility: 1.0,
            stability: 1.0,
            biomass_accumulation: 0.0,
            plant_biomass: 10.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TerrainGrid {
    pub cells: Vec<Vec<TerrainCell>>,
    pub width: u16,
    pub height: u16,
    pub dust_bowl_timer: u32,
    #[serde(skip)]
    pub is_dirty: bool,
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
            is_dirty: true,
        }
    }

    pub fn update(&mut self, herbivore_biomass: f64) -> f64 {
        self.is_dirty = true;
        if self.dust_bowl_timer > 0 {
            self.dust_bowl_timer -= 1;
        }

        let pressure = (herbivore_biomass / 5000.0) as f32;
        let global_recovery_rate = (0.001 - pressure).max(-0.01);
        let is_dust_bowl = self.dust_bowl_timer > 0;

        // Create a lightweight type grid for neighbor checks in parallel
        let type_grid: Vec<Vec<TerrainType>> = self
            .cells
            .iter()
            .map(|row| row.iter().map(|c| c.terrain_type).collect())
            .collect();

        // Hydration map: true if cell is within radius 2 of a river
        let mut hydration_map = vec![vec![false; self.width as usize]; self.height as usize];
        for (y, row) in type_grid.iter().enumerate() {
            for (x, &t) in row.iter().enumerate() {
                if t == TerrainType::River {
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0
                                && nx < self.width as i32
                                && ny >= 0
                                && ny < self.height as i32
                            {
                                hydration_map[ny as usize][nx as usize] = true;
                            }
                        }
                    }
                }
            }
        }

        type TransitionVec = Vec<Vec<(u16, u16, TerrainType)>>;
        let (total_biomass_vec, transitions): (Vec<f64>, TransitionVec) = self
            .cells
            .par_iter_mut()
            .enumerate()
            .map(|(y, row)| {
                let mut row_biomass = 0.0;
                let mut row_transitions = Vec::new();
                let mut rng = rand::thread_rng();

                for (x, cell) in row.iter_mut().enumerate() {
                    let x_u16 = x as u16;
                    let y_u16 = y as u16;
                    row_biomass += cell.plant_biomass as f64;

                    // 1. Local Biological Feedback
                    let mut fertility_gain =
                        (global_recovery_rate + (cell.plant_biomass * 0.0001)).max(-0.05);

                    // Phase 52: Hydration Effect
                    if hydration_map[y][x] {
                        fertility_gain += 0.005;
                    }

                    let r = match cell.terrain_type {
                        TerrainType::Forest => 0.05,
                        TerrainType::Plains => 0.02,
                        TerrainType::Oasis => 0.08,
                        TerrainType::Desert => 0.005,
                        _ => 0.0,
                    };

                    let plant_loss = if r > 0.0 {
                        cell.plant_biomass * 0.00005
                    } else {
                        0.0
                    };

                    cell.fertility = (cell.fertility + fertility_gain - plant_loss).clamp(0.0, 1.0);

                    let _k = cell.fertility * 100.0;

                    cell.biomass_accumulation *= 0.999;
                    if is_dust_bowl && cell.terrain_type == TerrainType::Plains {
                        cell.fertility = (cell.fertility - 0.05).max(0.0);
                    }

                    // 2. Succession Logic (using type_grid snapshot)
                    let mut forest_neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0
                                && nx < type_grid[0].len() as i32
                                && ny >= 0
                                && ny < type_grid.len() as i32
                                && type_grid[ny as usize][nx as usize] == TerrainType::Forest
                            {
                                forest_neighbors += 1;
                            }
                        }
                    }

                    match cell.terrain_type {
                        TerrainType::Plains => {
                            let chance = 0.001 + (forest_neighbors as f64 * 0.01);
                            if cell.plant_biomass > 60.0
                                && cell.fertility > 0.6
                                && rng.gen_bool(chance.min(1.0))
                            {
                                row_transitions.push((x_u16, y_u16, TerrainType::Forest));
                            } else if cell.fertility < 0.05 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Desert));
                            } else if cell.fertility < 0.15 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Barren));
                            }
                        }
                        TerrainType::Forest => {
                            if cell.fertility < 0.3 || cell.plant_biomass < 20.0 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::River => {
                            // Phase 54 Tuning: River Evaporation
                            // Isolated rivers or rivers in low fertility zones can dry up
                            let mut river_neighbors = 0;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 {
                                        continue;
                                    }
                                    let nx = x as i32 + dx;
                                    let ny = y as i32 + dy;
                                    if nx >= 0
                                        && nx < type_grid[0].len() as i32
                                        && ny >= 0
                                        && ny < type_grid.len() as i32
                                        && type_grid[ny as usize][nx as usize] == TerrainType::River
                                    {
                                        river_neighbors += 1;
                                    }
                                }
                            }
                            if river_neighbors == 0 && cell.fertility < 0.2 && rng.gen_bool(0.01) {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::Desert => {
                            if cell.fertility > 0.3 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::Barren => {
                            if cell.fertility > 0.4 {
                                row_transitions.push((x_u16, y_u16, cell.original_type));
                            }
                        }
                        _ => {}
                    }
                }
                (row_biomass, row_transitions)
            })
            .unzip();

        // Apply transitions sequentially
        for row_list in transitions {
            for (x, y, t) in row_list {
                self.set_cell_type(x, y, t);
            }
        }
        total_biomass_vec.iter().sum()
    }

    pub fn trigger_dust_bowl(&mut self, duration: u32) {
        self.dust_bowl_timer = duration;
    }

    pub fn has_neighbor_type(&self, x: u16, y: u16, t: TerrainType) -> bool {
        let ix = x as i32;
        let iy = y as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = ix + dx;
                let ny = iy + dy;
                if nx >= 0
                    && nx < self.width as i32
                    && ny >= 0
                    && ny < self.height as i32
                    && self.cells[ny as usize][nx as usize].terrain_type == t
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn deplete(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].fertility = (self.cells[iy][ix].fertility - amount).max(0.0);
        self.is_dirty = true;
    }

    pub fn fertilize(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].fertility = (self.cells[iy][ix].fertility + amount).min(1.0);
        self.is_dirty = true;
    }

    pub fn add_biomass(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].biomass_accumulation += amount;
        self.is_dirty = true;
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
        self.is_dirty = true;
    }

    /// Manually set cell fertility (useful for testing)
    pub fn set_fertility(&mut self, x: u16, y: u16, f: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].fertility = f.clamp(0.0, 1.0);
        self.is_dirty = true;
    }

    pub fn average_fertility(&self) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;
        for row in &self.cells {
            for cell in row {
                sum += cell.fertility;
                count += 1;
            }
        }
        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }

    pub fn add_global_fertility(&mut self, amount: f32) {
        let per_cell = amount / (self.width as f32 * self.height as f32);
        for row in &mut self.cells {
            for cell in row {
                cell.fertility = (cell.fertility + per_cell).clamp(0.0, 1.0);
            }
        }
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
        terrain.update(0.0);
        assert_eq!(terrain.dust_bowl_timer, 499);
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
            terrain.update(0.0);
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
