pub use primordium_data::{OutpostSpecialization, TerrainType};
use rand::Rng;
use rayon::prelude::*;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

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
    /// NEW: Resistance to type change (0.0 to 1.0)
    pub stability: f32,
    /// NEW: Accumulated biomass from entities (triggers Forest transition)
    pub biomass_accumulation: f32,
    /// NEW: Local plant biomass density (0.0 to 100.0)
    pub plant_biomass: f32,
    /// NEW: Owner of the cell (for Nests and Outposts)
    pub owner_id: Option<uuid::Uuid>,
    /// NEW: Energy stored in the cell (for Outposts)
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
    type_buffer: Vec<TerrainType>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    hydration_buffer: Vec<bool>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    moisture_buffer: Vec<f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    cooling_buffer: Vec<f32>,
}

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

impl TerrainGrid {
    pub fn generate(width: u16, height: u16, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut cells = vec![TerrainCell::default(); width as usize * height as usize];

        let w = width as usize;
        for (idx, cell) in cells.iter_mut().enumerate() {
            let x = (idx % w) as f32;
            let y = (idx / w) as f32;
            let noise = Self::value_noise(x, y, seed);
            cell.elevation = noise;
        }

        let mountain_threshold = 0.7;
        let river_threshold = 0.25;

        for cell in &mut cells {
            if cell.elevation > mountain_threshold {
                cell.terrain_type = TerrainType::Mountain;
                cell.original_type = TerrainType::Mountain;
            } else if cell.elevation < river_threshold {
                cell.terrain_type = TerrainType::River;
                cell.original_type = TerrainType::River;
            }
        }

        let oasis_count = ((width as usize * height as usize) / 200).max(3);
        let rock_count = ((width as usize * height as usize) / 150).max(5);

        let mut placed = 0;
        let mut attempts = 0;
        while placed < oasis_count && attempts < oasis_count * 10 {
            let x = rng.gen_range(0..width as usize);
            let y = rng.gen_range(0..height as usize);
            let idx = (y * width as usize) + x;
            if cells[idx].terrain_type == TerrainType::Plains {
                cells[idx].terrain_type = TerrainType::Oasis;
                cells[idx].original_type = TerrainType::Oasis;
                placed += 1;
            }
            attempts += 1;
        }

        placed = 0;
        attempts = 0;
        while placed < rock_count && attempts < rock_count * 10 {
            let x = rng.gen_range(0..width as usize);
            let y = rng.gen_range(0..height as usize);
            let idx = (y * width as usize) + x;
            if cells[idx].terrain_type == TerrainType::Plains {
                cells[idx].terrain_type = TerrainType::Wall;
                cells[idx].original_type = TerrainType::Wall;
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
            outpost_indices: HashSet::new(),
            type_buffer: vec![TerrainType::Plains; width as usize * height as usize],
            hydration_buffer: vec![false; width as usize * height as usize],
            moisture_buffer: vec![0.5; width as usize * height as usize],
            cooling_buffer: vec![0.0; width as usize * height as usize],
        }
    }

    #[inline(always)]
    pub fn index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn update(&mut self, herbivore_biomass: f64, tick: u64, world_seed: u64) -> (f64, f64) {
        if self.is_dirty {
            self.outpost_indices.clear();
            for (i, c) in self.cells.iter().enumerate() {
                if matches!(c.terrain_type, TerrainType::Outpost) {
                    self.outpost_indices.insert(i);
                }
            }
            self.is_dirty = false;
        }

        if self.dust_bowl_timer > 0 {
            self.dust_bowl_timer -= 1;
        }

        let pressure = (herbivore_biomass / 5000.0) as f32;
        let global_recovery_rate = (0.001 - pressure).max(-0.01);
        let is_dust_bowl = self.dust_bowl_timer > 0;

        let w = self.width;
        let h = self.height;

        if self.type_buffer.len() != w as usize * h as usize {
            self.type_buffer = vec![TerrainType::Plains; w as usize * h as usize];
            self.hydration_buffer = vec![false; w as usize * h as usize];
            self.moisture_buffer = vec![0.5; w as usize * h as usize];
            self.cooling_buffer = vec![0.0; w as usize * h as usize];
        }

        let mut outpost_map = vec![false; self.cells.len()];
        for (i, cell) in self.cells.iter().enumerate() {
            self.type_buffer[i] = cell.terrain_type;
            self.hydration_buffer[i] = false;

            match cell.terrain_type {
                TerrainType::River => {
                    self.moisture_buffer[i] = 1.0;
                    self.cooling_buffer[i] = 0.5;
                }
                TerrainType::Forest => {
                    self.moisture_buffer[i] = (self.moisture_buffer[i] + 0.1).min(1.0);
                    self.cooling_buffer[i] = 1.0;
                }
                TerrainType::Oasis => {
                    self.moisture_buffer[i] = 1.0;
                    self.cooling_buffer[i] = 0.8;
                }
                TerrainType::Desert => {
                    self.moisture_buffer[i] *= 0.95;
                    self.cooling_buffer[i] *= 0.9;
                }
                _ => {
                    self.moisture_buffer[i] *= 0.99;
                    self.cooling_buffer[i] *= 0.99;
                }
            }

            if cell.terrain_type == TerrainType::Outpost {
                outpost_map[i] = true;
            }
        }

        let mut next_moisture = self.moisture_buffer.clone();
        let mut next_cooling = self.cooling_buffer.clone();

        for y in 1..(h as usize - 1) {
            for x in 1..(w as usize - 1) {
                let idx = y * w as usize + x;
                let avg_m = (self.moisture_buffer[idx - 1]
                    + self.moisture_buffer[idx + 1]
                    + self.moisture_buffer[idx - w as usize]
                    + self.moisture_buffer[idx + w as usize])
                    * 0.25;
                next_moisture[idx] =
                    (self.moisture_buffer[idx] * 0.9 + avg_m * 0.1).clamp(0.0, 1.0);

                let avg_c = (self.cooling_buffer[idx - 1]
                    + self.cooling_buffer[idx + 1]
                    + self.cooling_buffer[idx - w as usize]
                    + self.cooling_buffer[idx + w as usize])
                    * 0.25;
                next_cooling[idx] = (self.cooling_buffer[idx] * 0.9 + avg_c * 0.1).clamp(0.0, 1.0);
            }
        }
        self.moisture_buffer = next_moisture;
        self.cooling_buffer = next_cooling;

        // Hydration map: true if cell is within radius 2 of a river
        for y in 0..h {
            for x in 0..w {
                if self.type_buffer[self.index(x, y)] == TerrainType::River {
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                                let nidx = (ny as usize * w as usize) + nx as usize;
                                self.hydration_buffer[nidx] = true;
                            }
                        }
                    }
                }
            }
        }

        let type_grid = &self.type_buffer;
        let hydration_map = &self.hydration_buffer;
        let moisture_map = &self.moisture_buffer;
        let cooling_map = &self.cooling_buffer;
        let outposts = &outpost_map;

        type TransitionVec = Vec<Vec<(u16, u16, TerrainType)>>;
        let (stats, transitions): (Vec<(f64, f64)>, TransitionVec) = self
            .cells
            .par_chunks_mut(w as usize)
            .enumerate()
            .map(|(y, row)| {
                let mut row_biomass = 0.0;
                let mut row_sequestration = 0.0;
                let mut row_transitions = Vec::new();
                let mut rng = ChaCha8Rng::seed_from_u64(world_seed ^ tick ^ (y as u64));

                for (x, cell) in row.iter_mut().enumerate() {
                    let idx = y * w as usize + x;
                    cell.local_moisture = moisture_map[idx];
                    cell.local_cooling = cooling_map[idx];

                    let x_u16 = x as u16;
                    let y_u16 = y as u16;
                    row_biomass += cell.plant_biomass as f64;

                    let is_near_outpost = if cell.terrain_type == TerrainType::Forest {
                        let mut found = false;
                        for dy in -2..=2 {
                            for dx in -2..=2 {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0
                                    && nx < w as i32
                                    && ny >= 0
                                    && ny < h as i32
                                    && outposts[(ny as usize * w as usize) + nx as usize]
                                {
                                    found = true;
                                    break;
                                }
                            }

                            if found {
                                break;
                            }
                        }
                        found
                    } else {
                        false
                    };

                    let seq_mult = if is_near_outpost { 2.5 } else { 1.0 };
                    if cell.terrain_type == TerrainType::Forest {
                        row_sequestration += cell.plant_biomass as f64
                            * seq_mult
                            * (1.0 + cell.local_moisture as f64);
                    }

                    let mut fertility_gain =
                        (global_recovery_rate + (cell.plant_biomass * 0.0001)).max(-0.05);

                    fertility_gain += cell.local_moisture * 0.01;

                    if hydration_map[idx] {
                        fertility_gain += 0.005;
                    }

                    cell.fertility = (cell.fertility + fertility_gain).clamp(0.0, 1.0);

                    let _r = match cell.terrain_type {
                        TerrainType::Forest => 0.05 * (1.0 + cell.local_moisture),
                        TerrainType::Plains => 0.02,
                        TerrainType::Oasis => 0.08,
                        TerrainType::Desert => 0.005,
                        _ => 0.0,
                    };

                    let seq_mult = if is_near_outpost { 2.5 } else { 1.0 };
                    if cell.terrain_type == TerrainType::Forest {
                        row_sequestration += cell.plant_biomass as f64 * seq_mult;
                    }

                    let mut fertility_gain =
                        (global_recovery_rate + (cell.plant_biomass * 0.0001)).max(-0.05);

                    if hydration_map[(y * w as usize) + x] {
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

                    cell.biomass_accumulation *= 0.999;
                    if is_dust_bowl && cell.terrain_type == TerrainType::Plains {
                        cell.fertility = (cell.fertility - 0.05).max(0.0);
                    }

                    if cell.energy_store > 1000.0 {
                        cell.fertility *= 0.99;
                    }

                    let mut forest_neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0
                                && nx < w as i32
                                && ny >= 0
                                && ny < h as i32
                                && type_grid[(ny as usize * w as usize) + nx as usize]
                                    == TerrainType::Forest
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
                            let mut river_neighbors = 0;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 {
                                        continue;
                                    }
                                    let nx = x as i32 + dx;
                                    let ny = y as i32 + dy;
                                    if nx >= 0
                                        && nx < w as i32
                                        && ny >= 0
                                        && ny < h as i32
                                        && type_grid[(ny as usize * w as usize) + nx as usize]
                                            == TerrainType::River
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
                ((row_biomass, row_sequestration), row_transitions)
            })
            .unzip();

        for row_list in transitions {
            for (x, y, t) in row_list {
                self.set_cell_type(x, y, t);
            }
        }
        let total_biomass = stats.iter().map(|s| s.0).sum();
        let total_sequestration = stats.iter().map(|s| s.1).sum();
        (total_biomass, total_sequestration)
    }

    pub fn trigger_dust_bowl(&mut self, duration: u32) {
        self.dust_bowl_timer = duration;
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
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        &self.cells[self.index(ix, iy)]
    }

    pub fn movement_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.movement_modifier()
    }
    pub fn food_spawn_modifier(&self, x: f64, y: f64) -> f64 {
        self.get(x, y).terrain_type.food_spawn_modifier()
    }
    pub fn get_cell(&self, x: u16, y: u16) -> &TerrainCell {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
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

    /// Manually set cell type (useful for testing and disasters)
    pub fn set_cell_type(&mut self, x: u16, y: u16, t: TerrainType) {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
        let idx = self.index(ix, iy);

        if self.cells[idx].terrain_type == TerrainType::Outpost {
            self.outpost_indices.remove(&idx);
        }
        if t == TerrainType::Outpost {
            self.outpost_indices.insert(idx);
        }

        self.cells[idx].terrain_type = t;
        self.is_dirty = true;
    }

    /// Manually set cell fertility (useful for testing)
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
        terrain.update(0.0, 0, 42);
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
            terrain.update(0.0, 0, 42);
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
}
