use crate::model::state::entity::Entity;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
pub struct SpatialHash {
    pub cell_size: f64,
    pub cells: HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(10.0) // Default cell size
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, x: f64, y: f64, index: usize) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        self.cells.entry((cx, cy)).or_default().push(index);
    }

    /// NEW: Build the hash in parallel from a slice of positions.
    pub fn build_parallel(&mut self, positions: &[(f64, f64)]) {
        self.clear();

        // Use a Mutex-protected temporary map to collect entries in parallel.
        // For even higher performance with 10k+ entities, consider DashMap or a fixed-grid array.
        let temp_cells: Mutex<HashMap<(i32, i32), Vec<usize>>> = Mutex::new(HashMap::new());

        positions.par_iter().enumerate().for_each(|(idx, &(x, y))| {
            let cx = (x / self.cell_size).floor() as i32;
            let cy = (y / self.cell_size).floor() as i32;
            let mut map = temp_cells.lock().unwrap();
            map.entry((cx, cy)).or_default().push(idx);
        });

        self.cells = temp_cells.into_inner().unwrap();
    }

    pub fn query(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut result = Vec::new();
        let min_cx = ((x - radius) / self.cell_size).floor() as i32;
        let max_cx = ((x + radius) / self.cell_size).floor() as i32;
        let min_cy = ((y - radius) / self.cell_size).floor() as i32;
        let max_cy = ((y + radius) / self.cell_size).floor() as i32;

        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                if let Some(indices) = self.cells.get(&(cx, cy)) {
                    result.extend_from_slice(indices);
                }
            }
        }
        result
    }
}

// Keeping Rect as it might be useful for future features
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn contains_entity(&self, e: &Entity) -> bool {
        e.physics.x >= self.x
            && e.physics.x < self.x + self.w
            && e.physics.y >= self.y
            && e.physics.y < self.y + self.h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insert_and_query_same_cell() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(5.0, 5.0, 0);
        hash.insert(7.0, 8.0, 1);

        let results = hash.query(6.0, 6.0, 5.0);

        assert!(results.contains(&0), "Should find entity 0");
        assert!(results.contains(&1), "Should find entity 1");
    }

    #[test]
    fn test_spatial_hash_query_finds_nearby() {
        let mut hash = SpatialHash::new(5.0);
        hash.insert(10.0, 10.0, 0); // Cell (2, 2)
        hash.insert(12.0, 10.0, 1); // Cell (2, 2) - same cell
        hash.insert(100.0, 100.0, 2); // Cell (20, 20) - far away

        let results = hash.query(11.0, 10.0, 5.0);

        assert!(results.contains(&0), "Should find nearby entity 0");
        assert!(results.contains(&1), "Should find nearby entity 1");
        assert!(!results.contains(&2), "Should NOT find distant entity 2");
    }

    #[test]
    fn test_spatial_hash_query_empty() {
        let hash = SpatialHash::new(10.0);
        let results = hash.query(50.0, 50.0, 10.0);
        assert!(results.is_empty(), "Empty hash should return empty results");
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(5.0, 5.0, 0);
        hash.insert(15.0, 15.0, 1);

        assert!(!hash.cells.is_empty(), "Should have cells before clear");

        hash.clear();

        assert!(hash.cells.is_empty(), "Should be empty after clear");
    }

    #[test]
    fn test_spatial_hash_query_crosses_cell_boundary() {
        let mut hash = SpatialHash::new(10.0);
        // Entity at (9, 9) is in cell (0, 0)
        hash.insert(9.0, 9.0, 0);
        // Entity at (11, 11) is in cell (1, 1)
        hash.insert(11.0, 11.0, 1);

        // Query centered at (10, 10) with radius 5 should find both
        let results = hash.query(10.0, 10.0, 5.0);

        assert!(
            results.contains(&0),
            "Should find entity across cell boundary"
        );
        assert!(
            results.contains(&1),
            "Should find entity across cell boundary"
        );
    }

    #[test]
    fn test_spatial_hash_negative_coordinates() {
        let mut hash = SpatialHash::new(10.0);
        hash.insert(-5.0, -5.0, 0); // Cell (-1, -1)
        hash.insert(-15.0, -15.0, 1); // Cell (-2, -2)

        let results = hash.query(-5.0, -5.0, 5.0);

        assert!(
            results.contains(&0),
            "Should find entity at negative coords"
        );
        assert!(!results.contains(&1), "Should not find distant entity");
    }
}
