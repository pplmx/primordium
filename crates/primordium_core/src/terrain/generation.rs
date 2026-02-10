use super::{TerrainCell, TerrainGrid, TerrainType};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashSet;

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
            outpost_buffer: vec![false; width as usize * height as usize],
        }
    }

    pub(crate) fn value_noise(x: f32, y: f32, seed: u64) -> f32 {
        let scale1 = 0.1;
        let scale2 = 0.05;
        let scale3 = 0.02;
        let noise1 = Self::hash_noise(x * scale1, y * scale1, seed) * 0.5;
        let noise2 = Self::hash_noise(x * scale2, y * scale2, seed.wrapping_add(1)) * 0.3;
        let noise3 = Self::hash_noise(x * scale3, y * scale3, seed.wrapping_add(2)) * 0.2;
        (noise1 + noise2 + noise3).clamp(0.0, 1.0)
    }

    pub(crate) fn hash_noise(x: f32, y: f32, seed: u64) -> f32 {
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

    pub(crate) fn hash(x: i32, y: i32, seed: u64) -> f32 {
        let n = (x.wrapping_mul(127) ^ y.wrapping_mul(311)) as u64 ^ seed;
        let n = n.wrapping_mul(0x517cc1b727220a95);
        let n = n ^ (n >> 32);
        (n & 0xFFFFFF) as f32 / 0xFFFFFF as f32
    }
}
