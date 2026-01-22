//! Pheromone system for inter-entity chemical communication

use serde::{Deserialize, Serialize};

/// Types of pheromones entities can deposit
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PheromoneType {
    Food,   // "I found food here"
    Danger, // "Predator detected"
}

/// A single cell in the pheromone grid
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PheromoneCell {
    pub food_strength: f32,   // 0.0 - 1.0
    pub danger_strength: f32, // 0.0 - 1.0
}

impl PheromoneCell {
    pub fn decay(&mut self, rate: f32) {
        self.food_strength *= rate;
        self.danger_strength *= rate;
        // Clean up very small values
        if self.food_strength < 0.01 {
            self.food_strength = 0.0;
        }
        if self.danger_strength < 0.01 {
            self.danger_strength = 0.0;
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
        }
    }
}

/// Grid-based pheromone map for the world
#[derive(Serialize, Deserialize)]
pub struct PheromoneGrid {
    cells: Vec<Vec<PheromoneCell>>,
    pub width: u16,
    pub height: u16,
    decay_rate: f32, // Per-tick decay multiplier (e.physics.g., 0.995)
}

impl PheromoneGrid {
    /// Create a new empty pheromone grid
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![PheromoneCell::default(); width as usize]; height as usize];
        Self {
            cells,
            width,
            height,
            decay_rate: 0.995, // Slow decay
        }
    }

    /// Deposit pheromone at a location
    pub fn deposit(&mut self, x: f64, y: f64, ptype: PheromoneType, amount: f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        self.cells[iy][ix].deposit(ptype, amount);
    }

    /// Sense average pheromone strength in a radius
    pub fn sense(&self, x: f64, y: f64, radius: f64) -> (f32, f32) {
        let cx = x as i32;
        let cy = y as i32;
        let r = radius as i32;

        let mut food_sum = 0.0f32;
        let mut danger_sum = 0.0f32;
        let mut count = 0;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    let cell = &self.cells[ny as usize][nx as usize];
                    food_sum += cell.food_strength;
                    danger_sum += cell.danger_strength;
                    count += 1;
                }
            }
        }

        if count > 0 {
            (food_sum / count as f32, danger_sum / count as f32)
        } else {
            (0.0, 0.0)
        }
    }

    /// Decay all pheromones (call once per tick)
    pub fn decay(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.decay(self.decay_rate);
            }
        }
    }

    /// Get cell at position for rendering
    pub fn get_cell(&self, x: u16, y: u16) -> &PheromoneCell {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        &self.cells[iy][ix]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pheromone_grid_new_dimensions() {
        let grid = PheromoneGrid::new(20, 10);
        assert_eq!(grid.width, 20);
        assert_eq!(grid.height, 10);
    }

    #[test]
    fn test_pheromone_deposit_food() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 0.5);

        let cell = grid.get_cell(5, 5);
        assert_eq!(cell.food_strength, 0.5);
        assert_eq!(cell.danger_strength, 0.0);
    }

    #[test]
    fn test_pheromone_deposit_danger() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Danger, 0.7);

        let cell = grid.get_cell(5, 5);
        assert_eq!(cell.food_strength, 0.0);
        assert_eq!(cell.danger_strength, 0.7);
    }

    #[test]
    fn test_pheromone_deposit_capped_at_one() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 0.8);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 0.5);

        let cell = grid.get_cell(5, 5);
        assert_eq!(cell.food_strength, 1.0, "Pheromone should be capped at 1.0");
    }

    #[test]
    fn test_pheromone_decay() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 1.0);

        // Decay multiple times
        for _ in 0..100 {
            grid.decay();
        }

        let cell = grid.get_cell(5, 5);
        assert!(
            cell.food_strength < 0.7,
            "Pheromone should decay significantly"
        );
    }

    #[test]
    fn test_pheromone_decay_cleanup_small_values() {
        let mut cell = PheromoneCell {
            food_strength: 0.005,
            danger_strength: 0.005,
        };

        cell.decay(0.9);

        assert_eq!(cell.food_strength, 0.0, "Small values should be cleaned up");
        assert_eq!(
            cell.danger_strength, 0.0,
            "Small values should be cleaned up"
        );
    }

    #[test]
    fn test_pheromone_sense_center() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 1.0);

        let (food, danger) = grid.sense(5.0, 5.0, 0.5);
        assert!(
            food > 0.0,
            "Should sense food pheromone at deposit location"
        );
        assert_eq!(danger, 0.0);
    }

    #[test]
    fn test_pheromone_sense_radius() {
        let mut grid = PheromoneGrid::new(10, 10);
        grid.deposit(5.0, 5.0, PheromoneType::Food, 1.0);

        // Sense from adjacent cell with radius 2
        let (food, _) = grid.sense(6.0, 5.0, 2.0);
        assert!(
            food > 0.0,
            "Should sense food pheromone from nearby location"
        );

        // Sense from far away
        let (food_far, _) = grid.sense(0.0, 0.0, 1.0);
        assert_eq!(food_far, 0.0, "Should not sense food from far away");
    }

    #[test]
    fn test_pheromone_boundary_safety() {
        let mut grid = PheromoneGrid::new(10, 10);

        // Should not panic on edge deposits
        grid.deposit(0.0, 0.0, PheromoneType::Food, 0.5);
        grid.deposit(9.0, 9.0, PheromoneType::Danger, 0.5);

        // Should not panic on out-of-bounds
        grid.deposit(100.0, 100.0, PheromoneType::Food, 0.5);
        let _ = grid.get_cell(100, 100);
        let _ = grid.sense(100.0, 100.0, 2.0);
    }
}
