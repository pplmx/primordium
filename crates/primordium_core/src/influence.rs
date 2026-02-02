use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(
    Serialize, Deserialize, Clone, Default, Debug, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct InfluenceCell {
    pub dominant_lineage: Option<Uuid>,
    pub intensity: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct InfluenceGrid {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<InfluenceCell>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct InfluenceSource {
    pub x: f64,
    pub y: f64,
    pub rank: f32,
    pub lineage_id: Uuid,
}

impl From<&crate::snapshot::InternalEntitySnapshot> for InfluenceSource {
    fn from(snap: &crate::snapshot::InternalEntitySnapshot) -> Self {
        Self {
            x: snap.x,
            y: snap.y,
            rank: snap.rank,
            lineage_id: snap.lineage_id,
        }
    }
}

impl Default for InfluenceGrid {
    fn default() -> Self {
        Self::new(100, 100)
    }
}

impl InfluenceGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![InfluenceCell::default(); width as usize * height as usize],
        }
    }

    pub fn update(&mut self, entities: &[crate::snapshot::InternalEntitySnapshot]) {
        for cell in &mut self.cells {
            cell.intensity *= 0.95;
            if cell.intensity < 0.01 {
                cell.dominant_lineage = None;
                cell.intensity = 0.0;
            }
        }

        let mut lineage_presence: HashMap<(usize, usize), HashMap<Uuid, f32>> = HashMap::new();

        for e in entities {
            let ex = e.x as usize;
            let ey = e.y as usize;
            if ex < self.width as usize && ey < self.height as usize {
                let entry = lineage_presence.entry((ex, ey)).or_default();
                let power = 0.1 + (e.rank * 0.5);
                *entry.entry(e.lineage_id).or_default() += power;
            }
        }

        for ((x, y), presence) in lineage_presence {
            let idx = y * self.width as usize + x;
            let mut strongest_l = None;
            let mut max_p = 0.0;
            for (lid, p) in presence {
                if p > max_p {
                    max_p = p;
                    strongest_l = Some(lid);
                }
            }

            if let Some(lid) = strongest_l {
                let cell = &mut self.cells[idx];
                if cell.dominant_lineage == Some(lid) {
                    cell.intensity = (cell.intensity + max_p).min(5.0);
                } else if max_p > cell.intensity {
                    cell.dominant_lineage = Some(lid);
                    cell.intensity = max_p;
                } else {
                    cell.intensity -= max_p;
                }
            }
        }
    }

    pub fn get_influence(&self, x: f64, y: f64) -> (Option<Uuid>, f32) {
        let ix = (x as usize).min(self.width as usize - 1);
        let iy = (y as usize).min(self.height as usize - 1);
        let cell = &self.cells[iy * self.width as usize + ix];
        (cell.dominant_lineage, cell.intensity)
    }
}
