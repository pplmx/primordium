use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize, Deserialize)]
pub struct SpatialHash {
    pub cell_size: f64,
    pub width: u16,
    pub height: u16,
    pub cols: usize,
    pub rows: usize,
    #[serde(skip, default = "Vec::new")]
    pub cell_offsets: Vec<usize>,
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
            cell_offsets: vec![0; cols * rows + 1],
            entity_indices: Vec::new(),
            lineage_centroids: HashMap::new(),
        }
    }

    pub fn new_empty() -> Self {
        Self::new(10.0, 1, 1)
    }

    pub fn clear(&mut self) {
        self.cell_offsets.fill(0);
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
        self.cols = (width as f64 / self.cell_size).ceil() as usize;
        self.rows = (height as f64 / self.cell_size).ceil() as usize;

        let cell_count = self.cols * self.rows;
        let entity_count = data.len();

        let counts: Vec<usize> = data
            .par_iter()
            .fold(
                || vec![0; cell_count],
                |mut acc, &(x, y, _)| {
                    if let Some(idx) = self.get_cell_idx(x, y) {
                        acc[idx] += 1;
                    }
                    acc
                },
            )
            .reduce(
                || vec![0; cell_count],
                |mut a, b| {
                    for i in 0..cell_count {
                        a[i] += b[i];
                    }
                    a
                },
            );

        self.cell_offsets.resize(cell_count + 1, 0);
        let mut total = 0;
        for (i, &count) in counts.iter().enumerate().take(cell_count) {
            self.cell_offsets[i] = total;
            total += count;
        }
        self.cell_offsets[cell_count] = total;

        self.entity_indices.resize(entity_count, 0);

        let current_positions: Vec<AtomicUsize> = self
            .cell_offsets
            .iter()
            .take(cell_count)
            .map(|&start| AtomicUsize::new(start))
            .collect();

        data.par_iter()
            .enumerate()
            .for_each(|(entity_idx, &(x, y, _))| {
                if let Some(cell_idx) = self.get_cell_idx(x, y) {
                    let write_idx = current_positions[cell_idx].fetch_add(1, Ordering::Relaxed);
                    unsafe {
                        let ptr = self.entity_indices.as_ptr() as *mut usize;
                        *ptr.add(write_idx) = entity_idx;
                    }
                }
            });

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

    pub fn add_centroid_data(&mut self, data: &[(f64, f64, uuid::Uuid)]) {
        let extra_centroids = data
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

        for (lid, (sx, sy, count)) in extra_centroids {
            let entry = self.lineage_centroids.entry(lid).or_insert((0.0, 0.0, 0));
            entry.0 += sx;
            entry.1 += sy;
            entry.2 += count;
        }
    }

    pub fn sense_kin(&self, x: f64, y: f64, range: f64, lid: uuid::Uuid) -> (f64, f64) {
        if let Some((cx, cy)) = self.get_lineage_centroid(&lid) {
            let dx = (cx - x) / range;
            let dy = (cy - y) / range;
            (dx.clamp(-1.0, 1.0), dy.clamp(-1.0, 1.0))
        } else {
            (0.0, 0.0)
        }
    }

    pub fn query(&self, x: f64, y: f64, radius: f64) -> Vec<usize> {
        let mut result = Vec::new();
        self.query_into(x, y, radius, &mut result);
        result
    }

    #[inline]
    pub fn query_callback<F>(&self, x: f64, y: f64, radius: f64, mut f: F)
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

                for &idx in &self.entity_indices[start..end] {
                    f(idx);
                }
            }
        }
    }

    #[inline]
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
mod tests {
    use super::*;

    #[test]
    fn test_spatial_hash_insert_and_query_same_cell() {
        let mut hash = SpatialHash::new(10.0, 100, 100);
        let data = vec![(5.0, 5.0, uuid::Uuid::nil()), (7.0, 8.0, uuid::Uuid::nil())];
        hash.build_with_lineage(&data, 100, 100);

        let results = hash.query(6.0, 6.0, 5.0);

        assert!(results.contains(&0), "Should find entity 0");
        assert!(results.contains(&1), "Should find entity 1");
    }

    #[test]
    fn test_spatial_hash_query_finds_nearby() {
        let mut hash = SpatialHash::new(5.0, 200, 200);
        let data = vec![
            (10.0, 10.0, uuid::Uuid::nil()),
            (12.0, 10.0, uuid::Uuid::nil()),
            (100.0, 100.0, uuid::Uuid::nil()),
        ];
        hash.build_with_lineage(&data, 200, 200);

        let results = hash.query(11.0, 10.0, 5.0);

        assert!(results.contains(&0), "Should find nearby entity 0");
        assert!(results.contains(&1), "Should find nearby entity 1");
        assert!(!results.contains(&2), "Should NOT find distant entity 2");
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut hash = SpatialHash::new(10.0, 100, 100);
        let data = vec![
            (5.0, 5.0, uuid::Uuid::nil()),
            (15.0, 15.0, uuid::Uuid::nil()),
        ];
        hash.build_with_lineage(&data, 100, 100);

        assert!(hash.entity_indices.len() == 2);
        hash.clear();
        assert!(hash.entity_indices.is_empty());
    }
}
