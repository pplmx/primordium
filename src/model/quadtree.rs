use crate::model::entity::Entity;

pub struct SpatialHash {
    pub cell_size: f64,
    pub cells: std::collections::HashMap<(i32, i32), Vec<usize>>,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, x: f64, y: f64, index: usize) {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        self.cells.entry((cx, cy)).or_default().push(index);
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
        e.x >= self.x && e.x < self.x + self.w && e.y >= self.y && e.y < self.y + self.h
    }
}
