pub use primordium_data::{OutpostSpecialization, TerrainType};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod disasters;
pub mod generation;
pub mod succession;

pub trait TerrainLogic {
    fn movement_modifier(&self) -> f64;
    fn food_spawn_modifier(&self) -> f64;
    fn symbol(&self) -> char;
}

impl TerrainLogic for TerrainType {
    fn movement_modifier(&self) -> f64 {
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
            TerrainType::Outpost => 0.6,
        }
    }

    fn food_spawn_modifier(&self) -> f64 {
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
            TerrainType::Outpost => 0.2,
        }
    }

    fn symbol(&self) -> char {
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
            TerrainType::Outpost => 'Ψ',
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct TerrainCell {
    pub terrain_type: TerrainType,
    pub original_type: TerrainType,
    pub elevation: f32,
    pub fertility: f32,
    pub stability: f32,
    pub biomass_accumulation: f32,
    pub plant_biomass: f32,
    pub owner_id: Option<uuid::Uuid>,
    pub energy_store: f32,
    pub outpost_spec: OutpostSpecialization,
    pub local_moisture: f32,
    pub local_cooling: f32,
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
            owner_id: None,
            energy_store: 0.0,
            outpost_spec: OutpostSpecialization::Standard,
            local_moisture: 0.5,
            local_cooling: 0.0,
        }
    }
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Default, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct TerrainGrid {
    pub cells: Vec<TerrainCell>,
    pub width: u16,
    pub height: u16,
    pub dust_bowl_timer: u32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub is_dirty: bool,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub outpost_indices: HashSet<usize>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub(crate) type_buffer: Vec<TerrainType>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub(crate) hydration_buffer: Vec<bool>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub(crate) moisture_buffer: Vec<f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub(crate) cooling_buffer: Vec<f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub(crate) outpost_buffer: Vec<bool>,
}

impl TerrainGrid {
    #[inline(always)]
    pub fn index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn get(&self, x: f64, y: f64) -> &TerrainCell {
        let ix = x.max(0.0).min(self.width as f64 - 1.0) as u16;
        let iy = y.max(0.0).min(self.height as f64 - 1.0) as u16;
        &self.cells[self.index(ix, iy)]
    }

    pub fn movement_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.movement_modifier()
    }

    pub fn food_spawn_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.food_spawn_modifier()
    }

    pub fn get_cell(&self, x: u16, y: u16) -> &TerrainCell {
        let ix = x.min(self.width.wrapping_sub(1));
        let iy = y.min(self.height.wrapping_sub(1));
        &self.cells[self.index(ix, iy)]
    }

    pub fn sense_wall(&self, x: f64, y: f64, range: f64) -> f32 {
        let mut min_dist = range;
        let ix = x as i32;
        let iy = y as i32;
        let r = range as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = ix + dx;
                let ny = iy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let cell = &self.cells[ny as usize * self.width as usize + nx as usize];
                    if cell.terrain_type == TerrainType::Wall {
                        let dist = ((dx * dx + dy * dy) as f64).sqrt();
                        if dist < min_dist {
                            min_dist = dist;
                        }
                    }
                }
            }
        }
        (1.0 - (min_dist / range)).clamp(0.0, 1.0) as f32
    }

    pub fn set_cell_type(&mut self, x: u16, y: u16, t: TerrainType) {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
        let idx = self.index(ix, iy);

        if self.cells[idx].terrain_type == TerrainType::Outpost {
            self.outpost_indices.remove(&idx);
            self.cells[idx].energy_store = 0.0;
        }
        if t == TerrainType::Outpost {
            self.outpost_indices.insert(idx);
        }

        self.cells[idx].terrain_type = t;
        self.is_dirty = true;
    }

    pub fn set_fertility(&mut self, x: u16, y: u16, f: f32) {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx].fertility = f.clamp(0.0, 1.0);
        self.is_dirty = true;
    }

    pub fn average_fertility(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.cells.iter().map(|c| c.fertility).sum();
        sum / self.cells.len() as f32
    }

    pub fn add_global_fertility(&mut self, amount: f32) {
        if self.cells.is_empty() {
            return;
        }
        let per_cell = amount / self.cells.len() as f32;
        for cell in &mut self.cells {
            cell.fertility = (cell.fertility + per_cell).clamp(0.0, 1.0);
        }
        self.is_dirty = true;
    }

    pub fn deplete(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx].fertility = (self.cells[idx].fertility - amount).max(0.0);
        self.is_dirty = true;
    }

    pub fn fertilize(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx].fertility = (self.cells[idx].fertility + amount).min(1.0);
        self.is_dirty = true;
    }

    pub fn add_biomass(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx].biomass_accumulation += amount;
        self.is_dirty = true;
    }

    pub fn has_neighbor_type(&self, x: u16, y: u16, t: TerrainType) -> bool {
        let ix = x as i32;
        let iy = y as i32;
        let w = self.width as i32;
        let h = self.height as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = ix + dx;
                let ny = iy + dy;
                if nx >= 0
                    && nx < w
                    && ny >= 0
                    && ny < h
                    && self.cells[(ny as usize * self.width as usize) + nx as usize].terrain_type
                        == t
                {
                    return true;
                }
            }
        }
        false
    }
}
