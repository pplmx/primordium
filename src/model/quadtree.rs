use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct SpatialHash {
    pub cell_size: f64,
    pub width: u16,
    pub height: u16,
    pub cols: usize,
    pub rows: usize,
    #[serde(skip, default = "Vec::new")]
    pub heads: Vec<i32>,
    #[serde(skip, default = "Vec::new")]
    pub next: Vec<i32>,
    #[serde(skip, default = "Vec::new")]
    pub entity_indices: Vec<usize>,
    #[serde(skip)]
    pub lineage_centroids: HashMap<uuid::Uuid, (f64, f64, usize)>,
}

impl SpatialHash {
    pub fn new(cell_size: f64, width: u16, height: u16) -> Self {
        let cols = (width as f64 / cell_size).ceil() as usize;
        let rows = (height as f64 / cell_size).ceil() as usize;
        Self {
            cell_size,
            width,
            height,
            cols,
            rows,
            heads: vec![-1; cols * rows],
            next: Vec::new(),
            entity_indices: Vec::new(),
            lineage_centroids: HashMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(10.0, 1, 1)
    }

    pub fn clear(&mut self) {
        if self.heads.len() != self.cols * self.rows {
            self.heads = vec![-1; self.cols * self.rows];
        } else {
            self.heads.fill(-1);
        }
        self.next.clear();
        self.entity_indices.clear();
        self.lineage_centroids.clear();
    }

    #[inline(always)]
    fn get_cell_idx(&self, x: f64, y: f64) -> Option<usize> {
        let cx = (x / self.cell_size).floor() as i32;
        let cy = (y / self.cell_size).floor() as i32;
        if cx >= 0 && cx < self.cols as i32 && cy >= 0 && cy < self.rows as i32 {
            Some((cy as usize * self.cols) + cx as usize)
        } else {
            None
        }
    }

    pub fn insert(&mut self, x: f64, y: f64, index: usize) {
        if let Some(cell_idx) = self.get_cell_idx(x, y) {
            let entry_idx = self.entity_indices.len() as i32;
            self.entity_indices.push(index);
            self.next.push(self.heads[cell_idx]);
            self.heads[cell_idx] = entry_idx;
        }
    }

    pub fn build_parallel(&mut self, positions: &[(f64, f64)], width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cols = (width as f64 / self.cell_size).ceil() as usize;
        self.rows = (height as f64 / self.cell_size).ceil() as usize;
        self.clear();
        for (idx, &(x, y)) in positions.iter().enumerate() {
            self.insert(x, y, idx);
        }
    }

    pub fn build_with_lineage(&mut self, data: &[(f64, f64, uuid::Uuid)], width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cols = (width as f64 / self.cell_size).ceil() as usize;
        self.rows = (height as f64 / self.cell_size).ceil() as usize;
        self.clear();

        for (idx, &(x, y, _)) in data.iter().enumerate() {
            self.insert(x, y, idx);
        }

        self.lineage_centroids = data
            .par_iter()
            .fold(
                HashMap::new,
                |mut acc: HashMap<uuid::Uuid, (f64, f64, usize)>, &(x, y, lid)| {
                    let entry = acc.entry(lid).or_insert((0.0, 0.0, 0));
                    entry.0 += x;
                    entry.1 += y;
                    entry.2 += 1;
                    acc
                },
            )
            .reduce(HashMap::new, |mut a, b| {
                for (lid, (sx, sy, count)) in b {
                    let entry = a.entry(lid).or_insert((0.0, 0.0, 0));
                    entry.0 += sx;
                    entry.1 += sy;
                    entry.2 += count;
                }
                a
            });
    }

    pub fn get_lineage_centroid(&self, lid: &uuid::Uuid) -> Option<(f64, f64)> {
        self.lineage_centroids.get(lid).map(|&(sx, sy, c)| {
            if c > 0 {
                (sx / c as f64, sy / c as f64)
            } else {
                (0.0, 0.0)
            }
        })
    }

    pub fn query(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut result = Vec::new();
        let min_cx = ((x - radius) / self.cell_size).floor() as i32;
        let max_cx = ((x + radius) / self.cell_size).floor() as i32;
        let min_cy = ((y - radius) / self.cell_size).floor() as i32;
        let max_cy = ((y + radius) / self.cell_size).floor() as i32;

        for cy in min_cy..=max_cy {
            if cy < 0 || cy >= self.rows as i32 {
                continue;
            }
            for cx in min_cx..=max_cx {
                if cx < 0 || cx >= self.cols as i32 {
                    continue;
                }

                let cell_idx = (cy as usize * self.cols) + cx as usize;
                let mut head = self.heads[cell_idx];
                while head != -1 {
                    result.push(self.entity_indices[head as usize]);
                    head = self.next[head as usize];
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insert_and_query_same_cell() {
        let mut hash = SpatialHash::new(10.0, 100, 100);
        hash.insert(5.0, 5.0, 0);
        hash.insert(7.0, 8.0, 1);

        let results = hash.query(6.0, 6.0, 5.0);

        assert!(results.contains(&0), "Should find entity 0");
        assert!(results.contains(&1), "Should find entity 1");
    }

    #[test]
    fn test_spatial_hash_query_finds_nearby() {
        let mut hash = SpatialHash::new(5.0, 200, 200);
        hash.insert(10.0, 10.0, 0);
        hash.insert(12.0, 10.0, 1);
        hash.insert(100.0, 100.0, 2);

        let results = hash.query(11.0, 10.0, 5.0);

        assert!(results.contains(&0), "Should find nearby entity 0");
        assert!(results.contains(&1), "Should find nearby entity 1");
        assert!(!results.contains(&2), "Should NOT find distant entity 2");
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut hash = SpatialHash::new(10.0, 100, 100);
        hash.insert(5.0, 5.0, 0);
        hash.insert(15.0, 15.0, 1);

        assert!(hash.entity_indices.len() == 2);
        hash.clear();
        assert!(hash.entity_indices.is_empty());
    }
}
