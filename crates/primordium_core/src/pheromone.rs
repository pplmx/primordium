use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub enum PheromoneType {
    Food,
    Danger,
    SignalA,
    SignalB,
}

#[derive(Debug, Clone, Copy, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct PheromoneDeposit {
    pub x: f64,
    pub y: f64,
    pub ptype: PheromoneType,
    pub amount: f32,
}

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize,
)]
#[archive(check_bytes)]
pub struct PheromoneCell {
    pub food_strength: f32,
    pub danger_strength: f32,
    pub sig_a_strength: f32,
    pub sig_b_strength: f32,
}

#[derive(Serialize, Deserialize, Debug, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct PheromoneGrid {
    pub cells: Vec<PheromoneCell>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub back_buffer: Vec<PheromoneCell>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    atomic_food: Vec<AtomicU32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    atomic_danger: Vec<AtomicU32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    atomic_sig_a: Vec<AtomicU32>,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    atomic_sig_b: Vec<AtomicU32>,
    pub width: u16,
    pub height: u16,
    pub decay_rate: f32,
    #[serde(skip)]
    #[with(rkyv::with::Skip)]
    pub is_dirty: bool,
}

impl Clone for PheromoneGrid {
    fn clone(&self) -> Self {
        let size = self.width as usize * self.height as usize;
        Self {
            cells: self.cells.clone(),
            back_buffer: self.back_buffer.clone(),
            atomic_food: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_danger: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_sig_a: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_sig_b: (0..size).map(|_| AtomicU32::new(0)).collect(),
            width: self.width,
            height: self.height,
            decay_rate: self.decay_rate,
            is_dirty: self.is_dirty,
        }
    }
}

impl Default for PheromoneGrid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

impl PheromoneGrid {
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        let cells = vec![PheromoneCell::default(); size];
        Self {
            cells,
            back_buffer: vec![PheromoneCell::default(); size],
            atomic_food: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_danger: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_sig_a: (0..size).map(|_| AtomicU32::new(0)).collect(),
            atomic_sig_b: (0..size).map(|_| AtomicU32::new(0)).collect(),
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
        let cell = &mut self.cells[idx];
        match ptype {
            PheromoneType::Food => cell.food_strength = (cell.food_strength + amount).min(1.0),
            PheromoneType::Danger => {
                cell.danger_strength = (cell.danger_strength + amount).min(1.0)
            }
            PheromoneType::SignalA => cell.sig_a_strength = (cell.sig_a_strength + amount).min(1.0),
            PheromoneType::SignalB => cell.sig_b_strength = (cell.sig_b_strength + amount).min(1.0),
        }
        self.is_dirty = true;
    }

    pub fn deposit_parallel(&self, x: f64, y: f64, ptype: PheromoneType, amount: f32) {
        let ix = (x as u16).min(self.width - 1);
        let iy = (y as u16).min(self.height - 1);
        let idx = self.index(ix, iy);
        let target = match ptype {
            PheromoneType::Food => &self.atomic_food[idx],
            PheromoneType::Danger => &self.atomic_danger[idx],
            PheromoneType::SignalA => &self.atomic_sig_a[idx],
            PheromoneType::SignalB => &self.atomic_sig_b[idx],
        };

        let mut current = target.load(Ordering::Relaxed);
        loop {
            let f = f32::from_bits(current);
            let next = (f + amount).min(1.0).to_bits();
            match target.compare_exchange_weak(current, next, Ordering::SeqCst, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

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

    pub fn sense(&self, x: f64, y: f64, radius: f64) -> (f32, f32) {
        let (f, d, _, _) = self.sense_all(x, y, radius);
        (f, d)
    }

    pub fn update(&mut self) {
        self.is_dirty = true;
        let size = self.cells.len();
        if self.atomic_food.len() != size {
            self.atomic_food = (0..size).map(|_| AtomicU32::new(0)).collect();
            self.atomic_danger = (0..size).map(|_| AtomicU32::new(0)).collect();
            self.atomic_sig_a = (0..size).map(|_| AtomicU32::new(0)).collect();
            self.atomic_sig_b = (0..size).map(|_| AtomicU32::new(0)).collect();
        }

        let rate = self.decay_rate;
        for i in 0..size {
            let f = f32::from_bits(self.atomic_food[i].swap(0, Ordering::SeqCst));
            let d = f32::from_bits(self.atomic_danger[i].swap(0, Ordering::SeqCst));
            let sa = f32::from_bits(self.atomic_sig_a[i].swap(0, Ordering::SeqCst));
            let sb = f32::from_bits(self.atomic_sig_b[i].swap(0, Ordering::SeqCst));

            let cell = &mut self.cells[i];
            cell.food_strength = (cell.food_strength * rate + f).min(1.0);
            cell.danger_strength = (cell.danger_strength * rate + d).min(1.0);
            cell.sig_a_strength = (cell.sig_a_strength * rate + sa).min(1.0);
            cell.sig_b_strength = (cell.sig_b_strength * rate + sb).min(1.0);

            if cell.food_strength < 0.01 {
                cell.food_strength = 0.0;
            }
            if cell.danger_strength < 0.01 {
                cell.danger_strength = 0.0;
            }
            if cell.sig_a_strength < 0.01 {
                cell.sig_a_strength = 0.0;
            }
            if cell.sig_b_strength < 0.01 {
                cell.sig_b_strength = 0.0;
            }
        }
        self.back_buffer.copy_from_slice(&self.cells);
    }

    pub fn get_cell(&self, x: u16, y: u16) -> &PheromoneCell {
        let ix = x.min(self.width - 1);
        let iy = y.min(self.height - 1);
        &self.cells[self.index(ix, iy)]
    }
}
