use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

#[derive(Clone, Default)]
pub struct SpatialHash {
    pub cell_size: f64,
    pub width: u16,
    pub height: u16,
    pub cols: usize,
    pub rows: usize,
    pub cell_offsets: Vec<usize>,
    pub entity_indices: Vec<usize>,
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
            cell_offsets: vec![0; cols * rows + 1],
            entity_indices: Vec::new(),
            lineage_centroids: HashMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(5.0, 100, 100)
    }

    #[inline]
    pub fn get_cell_idx(&self, x: f64, y: f64) -> Option<usize> {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let cx = (x / self.cell_size) as i32;
        let cy = (y / self.cell_size) as i32;
        if cx < 0 || cx >= self.cols as i32 || cy < 0 || cy >= self.rows as i32 {
            None
        } else {
            Some((cy as usize * self.cols) + cx as usize)
        }
    }

    pub fn build_parallel(&mut self, positions: &[(f64, f64)], width: u16, height: u16) {
        let data_with_dummy_lineage: Vec<(f64, f64, uuid::Uuid)> = positions
            .iter()
            .map(|&(x, y)| (x, y, uuid::Uuid::nil()))
            .collect();
        self.build_with_lineage(&data_with_dummy_lineage, width, height);
    }

    pub fn build_with_lineage(&mut self, data: &[(f64, f64, uuid::Uuid)], width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.cols = (f64::from(width) / self.cell_size).ceil() as usize;
        self.rows = (f64::from(height) / self.cell_size).ceil() as usize;

        let cell_count = self.cols * self.rows;
        let entity_count = data.len();

        let atomic_counts: Vec<AtomicUsize> =
            (0..cell_count).map(|_| AtomicUsize::new(0)).collect();
        data.par_iter().for_each(|&(x, y, _)| {
            if let Some(idx) = self.get_cell_idx(x, y) {
                atomic_counts[idx].fetch_add(1, AtomicOrdering::Relaxed);
            }
        });
        let counts: Vec<usize> = atomic_counts.into_iter().map(|a| a.into_inner()).collect();

        self.cell_offsets.resize(cell_count + 1, 0);
        let mut total = 0;
        for (i, &count) in counts.iter().enumerate().take(cell_count) {
            self.cell_offsets[i] = total;
            total += count;
        }
        self.cell_offsets[cell_count] = total;

        self.entity_indices.resize(entity_count, 0);

        let mut current_offsets = self.cell_offsets[..cell_count].to_vec();

        for (entity_idx, &(x, y, _)) in data.iter().enumerate() {
            if let Some(cell_idx) = self.get_cell_idx(x, y) {
                let write_idx = current_offsets[cell_idx];
                self.entity_indices[write_idx] = entity_idx;
                current_offsets[cell_idx] += 1;
            }
        }

        // Deterministic sequential centroid calculation
        let mut centroids = HashMap::new();
        for &(x, y, lid) in data {
            let entry = centroids.entry(lid).or_insert((0.0, 0.0, 0));
            entry.0 += x;
            entry.1 += y;
            entry.2 += 1;
        }
        self.lineage_centroids = centroids;
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

    pub fn sense_kin(&self, x: f64, y: f64, radius: f64, lid: uuid::Uuid) -> (f64, f64) {
        if let Some((cx, cy)) = self.get_lineage_centroid(&lid) {
            let dx = cx - x;
            let dy = cy - y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            if dist < radius {
                (dx / dist, dy / dist)
            } else {
                (0.0, 0.0)
            }
        } else {
            (0.0, 0.0)
        }
    }

    pub fn query_callback<F>(&self, x: f64, y: f64, radius: f64, mut callback: F)
    where
        F: FnMut(usize),
    {
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
                let start = self.cell_offsets[cell_idx];
                let end = self.cell_offsets[cell_idx + 1];

                for &entity_idx in &self.entity_indices[start..end] {
                    callback(entity_idx);
                }
            }
        }
    }

    pub fn count_nearby(&self, x: f64, y: f64, radius: f64) -> usize {
        let mut count = 0;
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
                count += self.cell_offsets[cell_idx + 1] - self.cell_offsets[cell_idx];
            }
        }
        count
    }

    pub fn count_nearby_kin(
        &self,
        x: f64,
        y: f64,
        radius: f64,
        lineage_id: uuid::Uuid,
        original_data: &[(f64, f64, uuid::Uuid)],
    ) -> usize {
        let mut count = 0;
        self.query_callback(x, y, radius, |idx| {
            if original_data[idx].2 == lineage_id {
                count += 1;
            }
        });
        count
    }

    #[inline]
    pub fn query_into(&self, x: f64, y: f64, radius: f64, result: &mut Vec<usize>) {
        result.clear();
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
                let start = self.cell_offsets[cell_idx];
                let end = self.cell_offsets[cell_idx + 1];

                result.extend_from_slice(&self.entity_indices[start..end]);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_query_finds_nearby() {
        let mut sh = SpatialHash::new(5.0, 20, 20);
        let data = vec![
            (1.0, 1.0, uuid::Uuid::new_v4()),
            (2.0, 2.0, uuid::Uuid::new_v4()),
            (10.0, 10.0, uuid::Uuid::new_v4()),
        ];
        sh.build_with_lineage(&data, 20, 20);

        let mut count = 0;
        sh.query_callback(1.5, 1.5, 2.0, |_| count += 1);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_spatial_hash_insert_and_query_same_cell() {
        let mut sh = SpatialHash::new(5.0, 20, 20);
        let data = vec![(1.0, 1.0, uuid::Uuid::new_v4())];
        sh.build_with_lineage(&data, 20, 20);
        let mut count = 0;
        sh.query_callback(1.0, 1.0, 1.0, |_| count += 1);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut sh = SpatialHash::new(5.0, 20, 20);
        let data = vec![(1.0, 1.0, uuid::Uuid::new_v4())];
        sh.build_with_lineage(&data, 20, 20);
        sh.build_with_lineage(&[], 20, 20);
        let mut count = 0;
        sh.query_callback(1.0, 1.0, 10.0, |_| count += 1);
        assert_eq!(count, 0);
    }
}
