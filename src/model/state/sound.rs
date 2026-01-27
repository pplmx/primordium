//! Sound system for real-time acoustic communication
//! Unlike pheromones, sound propagates as waves and decays rapidly.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// A request to deposit sound at a location
#[derive(Debug, Clone, Copy)]
pub struct SoundDeposit {
    pub x: f64,
    pub y: f64,
    pub amount: f32,
}

/// Grid-based sound map for the world
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SoundGrid {
    pub cells: Vec<f32>,
    #[serde(skip)]
    pub back_buffer: Vec<f32>,
    pub width: u16,
    pub height: u16,
    #[serde(skip)]
    pub is_dirty: bool,
}

impl SoundGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            cells: vec![0.0; width as usize * height as usize],
            back_buffer: vec![0.0; width as usize * height as usize],
            width,
            height,
            is_dirty: true,
        }
    }

    #[inline(always)]
    fn index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn deposit(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx] = (self.cells[idx] + amount).min(2.0);
        self.is_dirty = true;
    }

    pub fn update(&mut self) {
        self.is_dirty = true;

        std::mem::swap(&mut self.cells, &mut self.back_buffer);

        let old_cells = &self.back_buffer;
        let width = self.width;
        let height = self.height;

        self.cells
            .par_iter_mut()
            .enumerate()
            .for_each(|(idx, cell)| {
                let x = (idx % width as usize) as i32;
                let y = (idx / width as usize) as i32;

                let mut neighbors_sum = 0.0;
                let mut count = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            neighbors_sum +=
                                old_cells[(ny as usize * width as usize) + nx as usize];
                            count += 1;
                        }
                    }
                }

                let diffused = if count > 0 {
                    neighbors_sum / count as f32
                } else {
                    0.0
                };
                *cell = (old_cells[idx] * 0.4 + diffused * 0.6) * 0.7;

                if *cell < 0.01 {
                    *cell = 0.0;
                }
            });
    }

    pub fn sense(&self, x: f64, y: f64, radius: f64) -> f32 {
        let cx = x as i32;
        let cy = y as i32;
        let r = radius as i32;
        let mut sum = 0.0;
        let mut count = 0;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    sum += self.cells[(ny as usize * self.width as usize) + nx as usize];
                    count += 1;
                }
            }
        }

        if count > 0 {
            sum / count as f32
        } else {
            0.0
        }
    }
}
