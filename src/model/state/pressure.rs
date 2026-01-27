use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PressureType {
    BuildDemand,
    DigDemand,
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
    pub build_demand: f32,
    pub dig_demand: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PressureGrid {
    pub cells: Vec<PressureCell>,
    pub width: u16,
    pub height: u16,
    pub decay_rate: f32,
    #[serde(skip)]
    pub is_dirty: bool,
}

impl PressureGrid {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![PressureCell::default(); width as usize * height as usize];
        Self {
            cells,
            width,
            height,
            decay_rate: 0.99,
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
            PressureType::BuildDemand => {
                self.cells[idx].build_demand = (self.cells[idx].build_demand + amount).min(5.0);
            }
            PressureType::DigDemand => {
                self.cells[idx].dig_demand = (self.cells[idx].dig_demand + amount).min(5.0);
            }
        }
        self.is_dirty = true;
    }

    pub fn update(&mut self) {
        self.is_dirty = true;
        self.cells.par_iter_mut().for_each(|cell| {
            cell.build_demand *= self.decay_rate;
            cell.dig_demand *= self.decay_rate;
            if cell.build_demand < 0.01 {
                cell.build_demand = 0.0;
            }
            if cell.dig_demand < 0.01 {
                cell.dig_demand = 0.0;
            }
        });
    }

    pub fn sense(&self, x: f64, y: f64, radius: f64) -> (f32, f32) {
        let cx = x as i32;
        let cy = y as i32;
        let r = radius as i32;
        let mut b_sum = 0.0;
        let mut d_sum = 0.0;
        let mut count = 0;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let idx = self.index(nx as u16, ny as u16);
                    b_sum += self.cells[idx].build_demand;
                    d_sum += self.cells[idx].dig_demand;
                    count += 1;
                }
            }
        }

        if count > 0 {
            (b_sum / count as f32, d_sum / count as f32)
        } else {
            (0.0, 0.0)
        }
    }
}
