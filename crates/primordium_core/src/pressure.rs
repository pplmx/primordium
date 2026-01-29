use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PressureType {
    DigDemand,
    BuildDemand,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PressureDeposit {
    pub x: f64,
    pub y: f64,
    pub ptype: PressureType,
    pub amount: f32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PressureCell {
    pub dig_demand: f32,
    pub build_demand: f32,
}

impl PressureCell {
    pub fn decay(&mut self, rate: f32) {
        self.dig_demand *= rate;
        self.build_demand *= rate;
        if self.dig_demand < 0.01 {
            self.dig_demand = 0.0;
        }
        if self.build_demand < 0.01 {
            self.build_demand = 0.0;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PressureGrid {
    pub cells: Vec<PressureCell>,
    #[serde(skip)]
    pub back_buffer: Vec<PressureCell>,
    #[serde(skip)]
    atomic_dig: Vec<AtomicU32>,
    #[serde(skip)]
    atomic_build: Vec<AtomicU32>,
    pub width: u16,
    pub height: u16,
    pub decay_rate: f32,
    #[serde(skip)]
    pub is_dirty: bool,
}

impl Clone for PressureGrid {
    fn clone(&self) -> Self {
        let size = self.width as usize * self.height as usize;
        Self {
            cells: self.cells.clone(),
            back_buffer: self.back_buffer.clone(),
            atomic_dig: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_build: (0..size).map(|_| AtomicU32::new(0)).collect(),
            width: self.width,
            height: self.height,
            decay_rate: self.decay_rate,
            is_dirty: self.is_dirty,
        }
    }
}

impl Default for PressureGrid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl PressureGrid {
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        let cells = vec![PressureCell::default(); size];
        Self {
            cells,
            back_buffer: vec![PressureCell::default(); size],
            atomic_dig: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_build: (0..size).map(|_| AtomicU32::new(0)).collect(),
            width,
            height,
            decay_rate: 0.95,
            is_dirty: true,
        }
    }

    #[inline(always)]
    fn index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn deposit(&mut self, x: f64, y: f64, ptype: PressureType, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        match ptype {
            PressureType::DigDemand => {
                self.cells[idx].dig_demand = (self.cells[idx].dig_demand + amount).min(5.0)
            }
            PressureType::BuildDemand => {
                self.cells[idx].build_demand = (self.cells[idx].build_demand + amount).min(5.0)
            }
        }
        self.is_dirty = true;
    }

    pub fn deposit_parallel(&self, x: f64, y: f64, ptype: PressureType, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        let target = match ptype {
            PressureType::DigDemand => &self.atomic_dig[idx],
            PressureType::BuildDemand => &self.atomic_build[idx],
        };

        let mut current = target.load(Ordering::Relaxed);
        loop {
            let f = f32::from_bits(current);
            let next = (f + amount).min(5.0).to_bits();
            match target.compare_exchange_weak(current, next, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn update(&mut self) {
        self.is_dirty = true;
        let size = self.cells.len();
        if self.atomic_dig.len() != size {
            self.atomic_dig = (0..size).map(|_| AtomicU32::new(0)).collect();
            self.atomic_build = (0..size).map(|_| AtomicU32::new(0)).collect();
        }

        let rate = self.decay_rate;
        self.cells.par_iter_mut().enumerate().for_each(|(i, cell)| {
            let d = f32::from_bits(self.atomic_dig[i].swap(0, Ordering::SeqCst));
            let b = f32::from_bits(self.atomic_build[i].swap(0, Ordering::SeqCst));
            cell.dig_demand = (cell.dig_demand * rate + d).min(1.0);
            cell.build_demand = (cell.build_demand * rate + b).min(1.0);
            if cell.dig_demand < 0.01 {
                cell.dig_demand = 0.0;
            }
            if cell.build_demand < 0.01 {
                cell.build_demand = 0.0;
            }
        });
    }

    pub fn sense(&self, x: f64, y: f64, radius: f64) -> (f32, f32) {
        let cx = x as i32;
        let cy = y as i32;
        let r = radius as i32;
        let mut dig_sum = 0.0;
        let mut build_sum = 0.0;
        let mut count = 0;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let idx = self.index(nx as u16, ny as u16);
                    dig_sum += self.cells[idx].dig_demand;
                    build_sum += self.cells[idx].build_demand;
                    count += 1;
                }
            }
        }
        if count > 0 {
            (dig_sum / count as f32, build_sum / count as f32)
        } else {
            (0.0, 0.0)
        }
    }
}
