use super::{TerrainGrid, TerrainType};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

impl TerrainGrid {
    pub fn update(&mut self, herbivore_biomass: f64, tick: u64, world_seed: u64) -> (f64, f64) {
        if self.is_dirty {
            self.outpost_indices.clear();
            for (i, c) in self.cells.iter().enumerate() {
                if matches!(c.terrain_type, TerrainType::Outpost) {
                    self.outpost_indices.insert(i);
                }
            }
            self.is_dirty = false;
        }

        if self.dust_bowl_timer > 0 {
            self.dust_bowl_timer -= 1;
        }

        let pressure = (herbivore_biomass / 5000.0) as f32;
        let global_recovery_rate = (0.001 - pressure).max(-0.01);
        let is_dust_bowl = self.dust_bowl_timer > 0;

        let w = self.width;
        let h = self.height;

        if self.type_buffer.len() != w as usize * h as usize {
            self.type_buffer = vec![TerrainType::Plains; w as usize * h as usize];
            self.hydration_buffer = vec![false; w as usize * h as usize];
            self.moisture_buffer = vec![0.5; w as usize * h as usize];
            self.cooling_buffer = vec![0.0; w as usize * h as usize];
            self.outpost_buffer = vec![false; w as usize * h as usize];
        }

        for (i, cell) in self.cells.iter().enumerate() {
            self.type_buffer[i] = cell.terrain_type;
            self.hydration_buffer[i] = false;
            self.outpost_buffer[i] = false;

            match cell.terrain_type {
                TerrainType::River => {
                    self.moisture_buffer[i] = 1.0;
                    self.cooling_buffer[i] = 0.5;
                }
                TerrainType::Forest => {
                    self.moisture_buffer[i] = (self.moisture_buffer[i] + 0.1).min(1.0);
                    self.cooling_buffer[i] = 1.0;
                }
                TerrainType::Oasis => {
                    self.moisture_buffer[i] = 1.0;
                    self.cooling_buffer[i] = 0.8;
                }
                TerrainType::Desert => {
                    self.moisture_buffer[i] *= 0.95;
                    self.cooling_buffer[i] *= 0.9;
                }
                _ => {
                    self.moisture_buffer[i] *= 0.99;
                    self.cooling_buffer[i] *= 0.99;
                }
            }

            if cell.terrain_type == TerrainType::Outpost {
                self.outpost_buffer[i] = true;
            }
        }

        let mut next_moisture = self.moisture_buffer.clone();
        let mut next_cooling = self.cooling_buffer.clone();

        for y in 1..(h as usize - 1) {
            for x in 1..(w as usize - 1) {
                let idx = y * w as usize + x;
                let avg_m = (self.moisture_buffer[idx - 1]
                    + self.moisture_buffer[idx + 1]
                    + self.moisture_buffer[idx - w as usize]
                    + self.moisture_buffer[idx + w as usize])
                    * 0.25;
                next_moisture[idx] =
                    (self.moisture_buffer[idx] * 0.9 + avg_m * 0.1).clamp(0.0, 1.0);

                let avg_c = (self.cooling_buffer[idx - 1]
                    + self.cooling_buffer[idx + 1]
                    + self.cooling_buffer[idx - w as usize]
                    + self.cooling_buffer[idx + w as usize])
                    * 0.25;
                next_cooling[idx] = (self.cooling_buffer[idx] * 0.9 + avg_c * 0.1).clamp(0.0, 1.0);
            }
        }
        self.moisture_buffer = next_moisture;
        self.cooling_buffer = next_cooling;

        for y in 0..h {
            for x in 0..w {
                if self.type_buffer[self.index(x, y)] == TerrainType::River {
                    for dy in -2..=2 {
                        for dx in -2..=2 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                                let nidx = (ny as usize * w as usize) + nx as usize;
                                self.hydration_buffer[nidx] = true;
                            }
                        }
                    }
                }
            }
        }

        let type_grid = &self.type_buffer;
        let hydration_map = &self.hydration_buffer;
        let moisture_map = &self.moisture_buffer;
        let cooling_map = &self.cooling_buffer;
        let outposts = &self.outpost_buffer;

        type TransitionVec = Vec<Vec<(u16, u16, TerrainType)>>;
        let (stats, transitions): (Vec<(f64, f64)>, TransitionVec) = self
            .cells
            .par_chunks_mut(w as usize)
            .enumerate()
            .map(|(y, row)| {
                let mut row_biomass = 0.0;
                let mut row_sequestration = 0.0;
                let mut row_transitions = Vec::new();
                let mut rng = ChaCha8Rng::seed_from_u64(world_seed ^ tick ^ (y as u64));

                for (x, cell) in row.iter_mut().enumerate() {
                    let idx = y * w as usize + x;
                    cell.local_moisture = moisture_map[idx];
                    cell.local_cooling = cooling_map[idx];

                    let x_u16 = x as u16;
                    let y_u16 = y as u16;
                    row_biomass += cell.plant_biomass as f64;

                    let is_near_outpost = if cell.terrain_type == TerrainType::Forest {
                        let mut found = false;
                        for dy in -2..=2 {
                            for dx in -2..=2 {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0
                                    && nx < w as i32
                                    && ny >= 0
                                    && ny < h as i32
                                    && outposts[(ny as usize * w as usize) + nx as usize]
                                {
                                    found = true;
                                    break;
                                }
                            }

                            if found {
                                break;
                            }
                        }
                        found
                    } else {
                        false
                    };

                    let seq_mult = if is_near_outpost { 2.5 } else { 1.0 };
                    if cell.terrain_type == TerrainType::Forest {
                        row_sequestration += cell.plant_biomass as f64
                            * seq_mult
                            * (1.0 + cell.local_moisture as f64);
                    }

                    let mut fertility_gain =
                        (global_recovery_rate + (cell.plant_biomass * 0.0001)).max(-0.05);

                    fertility_gain += cell.local_moisture * 0.01;

                    if hydration_map[idx] {
                        fertility_gain += 0.005;
                    }

                    let plant_loss = if cell.terrain_type != TerrainType::Barren
                        && cell.terrain_type != TerrainType::Desert
                    {
                        cell.plant_biomass * 0.00005
                    } else {
                        0.0
                    };

                    cell.fertility = (cell.fertility + fertility_gain - plant_loss).clamp(0.0, 1.0);

                    cell.biomass_accumulation *= 0.999;
                    if is_dust_bowl && cell.terrain_type == TerrainType::Plains {
                        cell.fertility = (cell.fertility - 0.05).max(0.0);
                    }

                    if cell.energy_store > 1000.0 {
                        cell.fertility *= 0.99;
                    }

                    let mut forest_neighbors = 0;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if nx >= 0
                                && nx < w as i32
                                && ny >= 0
                                && ny < h as i32
                                && type_grid[(ny as usize * w as usize) + nx as usize]
                                    == TerrainType::Forest
                            {
                                forest_neighbors += 1;
                            }
                        }
                    }

                    match cell.terrain_type {
                        TerrainType::Plains => {
                            let chance = 0.001 + (forest_neighbors as f64 * 0.01);
                            if cell.plant_biomass > 60.0
                                && cell.fertility > 0.6
                                && rng.gen_bool(chance.min(1.0))
                            {
                                row_transitions.push((x_u16, y_u16, TerrainType::Forest));
                            } else if cell.fertility < 0.05 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Desert));
                            } else if cell.fertility < 0.15 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Barren));
                            }
                        }
                        TerrainType::Forest => {
                            if cell.fertility < 0.3 || cell.plant_biomass < 20.0 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::River => {
                            let mut river_neighbors = 0;
                            for dy in -1..=1 {
                                for dx in -1..=1 {
                                    if dx == 0 && dy == 0 {
                                        continue;
                                    }
                                    let nx = x as i32 + dx;
                                    let ny = y as i32 + dy;
                                    if nx >= 0
                                        && nx < w as i32
                                        && ny >= 0
                                        && ny < h as i32
                                        && type_grid[(ny as usize * w as usize) + nx as usize]
                                            == TerrainType::River
                                    {
                                        river_neighbors += 1;
                                    }
                                }
                            }
                            if river_neighbors == 0 && cell.fertility < 0.2 && rng.gen_bool(0.01) {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::Desert => {
                            if cell.fertility > 0.3 {
                                row_transitions.push((x_u16, y_u16, TerrainType::Plains));
                            }
                        }
                        TerrainType::Barren => {
                            if cell.fertility > 0.4 {
                                row_transitions.push((x_u16, y_u16, cell.original_type));
                            }
                        }
                        _ => {}
                    }
                }
                ((row_biomass, row_sequestration), row_transitions)
            })
            .unzip();

        for row_list in transitions {
            for (x, y, t) in row_list {
                self.set_cell_type(x, y, t);
            }
        }
        let total_biomass = stats.iter().map(|s| s.0).sum();
        let total_sequestration = stats.iter().map(|s| s.1).sum();
        (total_biomass, total_sequestration)
    }
}
