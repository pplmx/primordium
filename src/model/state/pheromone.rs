//! Pheromone system for inter-entity chemical communication

use serde::{Deserialize, Serialize};

/// Types of pheromones entities can deposit
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,    // "I found food here"
    Danger,  // "Predator detected"
    SignalA, // Semantic channel A
    SignalB, // Semantic channel B
}

/// A request to deposit pheromones at a location
#[derive(Debug, Clone, Copy)]
pub struct PheromoneDeposit {
    pub x: f64,
    pub y: f64,
    pub ptype: PheromoneType,
    pub amount: f32,
}

/// A single cell in the pheromone grid
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PheromoneCell {
    pub food_strength: f32,   // 0.0 - 1.0
    pub danger_strength: f32, // 0.0 - 1.0
    pub sig_a_strength: f32,  // 0.0 - 1.0
    pub sig_b_strength: f32,  // 0.0 - 1.0
}

impl PheromoneCell {
    pub fn decay(&mut self, rate: f32) {
        self.food_strength *= rate;
        self.danger_strength *= rate;
        self.sig_a_strength *= rate;
        self.sig_b_strength *= rate;

        // Clean up very small values
        let threshold = 0.01;
        if self.food_strength < threshold {
            self.food_strength = 0.0;
        }
        if self.danger_strength < threshold {
            self.danger_strength = 0.0;
        }
        if self.sig_a_strength < threshold {
            self.sig_a_strength = 0.0;
        }
        if self.sig_b_strength < threshold {
            self.sig_b_strength = 0.0;
        }
    }

    pub fn deposit(&mut self, ptype: PheromoneType, amount: f32) {
        match ptype {
            PheromoneType::Food => {
                self.food_strength = (self.food_strength + amount).min(1.0);
            }
            PheromoneType::Danger => {
                self.danger_strength = (self.danger_strength + amount).min(1.0);
            }
            PheromoneType::SignalA => {
                self.sig_a_strength = (self.sig_a_strength + amount).min(1.0);
            }
            PheromoneType::SignalB => {
                self.sig_b_strength = (self.sig_b_strength + amount).min(1.0);
            }
        }
    }
}

/// Grid-based pheromone map for the world
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PheromoneGrid {
    pub cells: Vec<PheromoneCell>,
    #[serde(skip)]
    pub back_buffer: Vec<PheromoneCell>,
    pub width: u16,
    pub height: u16,
    pub decay_rate: f32, // Per-tick decay multiplier
    #[serde(skip)]
    pub is_dirty: bool,
}

impl PheromoneGrid {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![PheromoneCell::default(); width as usize * height as usize];
        let back_buffer = cells.clone();
        Self {
            cells,
            back_buffer,
            width,
            height,
            decay_rate: 0.995,
            is_dirty: true,
        }
    }

    #[inline(always)]
    fn index(&self, x: u16, y: u16) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    pub fn deposit(&mut self, x: f64, y: f64, ptype: PheromoneType, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        self.cells[idx].deposit(ptype, amount);
        self.is_dirty = true;
    }

    /// Sense average pheromone strengths in a radius.
    /// Returns (Food, Danger, SignalA, SignalB)
    pub fn sense_all(&self, x: f64, y: f64, radius: f64) -> (f32, f32, f32, f32) {
        let cx = x as i32;
        let cy = y as i32;
        let r = radius as i32;

        let mut food_sum = 0.0f32;
        let mut danger_sum = 0.0f32;
        let mut sig_a_sum = 0.0f32;
        let mut sig_b_sum = 0.0f32;
        let mut count = 0;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let idx = self.index(nx as u16, ny as u16);
                    let cell = &self.back_buffer[idx];
                    food_sum += cell.food_strength;
                    danger_sum += cell.danger_strength;
                    sig_a_sum += cell.sig_a_strength;
                    sig_b_sum += cell.sig_b_strength;
                    count += 1;
                }
            }
        }

        if count > 0 {
            (
                food_sum / count as f32,
                danger_sum / count as f32,
                sig_a_sum / count as f32,
                sig_b_sum / count as f32,
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Legacy sense for compatibility
    pub fn sense(&self, x: f64, y: f64, radius: f64) -> (f32, f32) {
        let (f, d, _, _) = self.sense_all(x, y, radius);
        (f, d)
    }

    pub fn update(&mut self) {
        self.is_dirty = true;
        for cell in &mut self.cells {
            cell.decay(self.decay_rate);
        }
        for i in 0..self.cells.len() {
            self.back_buffer[i] = self.cells[i];
        }
    }

    pub fn get_cell(&self, x: u16, y: u16) -> &PheromoneCell {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
        &self.cells[self.index(ix, iy)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pheromone_deposit_signals() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::SignalA, 0.5);
        grid.deposit(5.0, 5.0, PheromoneType::SignalB, 0.7);

        let cell = grid.get_cell(5, 5);
        assert_eq!(cell.sig_a_strength, 0.5);
        assert_eq!(cell.sig_b_strength, 0.7);
    }
}
