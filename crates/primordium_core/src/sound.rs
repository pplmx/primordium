use rayon::prelude::*;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct SoundDeposit {
    pub x: f64,
    pub y: f64,
    pub amount: f32,
}

#[derive(Serialize, Deserialize, Debug, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct SoundGrid {
    pub cells: Vec<f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub back_buffer: Vec<f32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    atomic_deposits: Vec<AtomicU32>,
    pub width: u16,
    pub height: u16,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub is_dirty: bool,
}

impl Clone for SoundGrid {
    fn clone(&self) -> Self {
        let size = self.width as usize * self.height as usize;
        Self {
            cells: self.cells.clone(),
            back_buffer: self.back_buffer.clone(),
            atomic_deposits: (0..size).map(|_| AtomicU32::new(0)).collect(),
            width: self.width,
            height: self.height,
            is_dirty: self.is_dirty,
        }
    }
}

impl Default for SoundGrid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl SoundGrid {
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            cells: vec![0.0; size],
            back_buffer: vec![0.0; size],
            atomic_deposits: (0..size).map(|_| AtomicU32::new(0)).collect(),
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

    pub fn deposit_parallel(&self, x: f64, y: f64, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        let target = &self.atomic_deposits[idx];

        let mut current = target.load(Ordering::Relaxed);
        loop {
            let f = f32::from_bits(current);
            let next = (f + amount).min(2.0).to_bits();
            match target.compare_exchange_weak(current, next, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn update(&mut self) {
        self.is_dirty = true;
        let size = self.cells.len();
        if self.atomic_deposits.len() != size {
            self.atomic_deposits = (0..size).map(|_| AtomicU32::new(0)).collect();
        }

        std::mem::swap(&mut self.cells, &mut self.back_buffer);
        let old_cells = &self.back_buffer;
        let width = self.width;
        let height = self.height;

        let atomics = &self.atomic_deposits;

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
                let dep = f32::from_bits(atomics[idx].swap(0, Ordering::SeqCst));
                *cell = (old_cells[idx] * 0.4 + diffused * 0.6 + dep) * 0.7;
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

    pub fn get_cell(&self, x: u16, y: u16) -> f32 {
        if x < self.width && y < self.height {
            self.cells[self.index(x, y)]
        } else {
            0.0
        }
    }
}
