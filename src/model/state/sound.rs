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
#[derive(Serialize, Deserialize, Clone)]
pub struct SoundGrid {
    pub cells: Vec<Vec<f32>>,
    pub width: u16,
    pub height: u16,
}

impl SoundGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            cells: vec![vec![0.0; width as usize]; height as usize],
            width,
            height,
        }
    }

    pub fn deposit(&mut self, x: f64, y: f64, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix] = (self.cells[iy][ix] + amount).min(2.0);
    }

    pub fn update(&mut self) {
        // 1. Diffusion & Decay (Simulating ripples)
        let old_cells = self.cells.clone();

        self.cells.par_iter_mut().enumerate().for_each(|(y, row)| {
            for (x, cell) in row.iter_mut().enumerate() {
                let mut neighbors_sum = 0.0;
                let mut count = 0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                            neighbors_sum += old_cells[ny as usize][nx as usize];
                            count += 1;
                        }
                    }
                }

                // Diffusion factor (0.1) and rapid decay (0.7)
                let diffused = if count > 0 {
                    neighbors_sum / count as f32
                } else {
                    0.0
                };
                *cell = (old_cells[y][x] * 0.4 + diffused * 0.6) * 0.7;

                if *cell < 0.01 {
                    *cell = 0.0;
                }
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
                    sum += self.cells[ny as usize][nx as usize];
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
