use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::{OutpostSpecialization, TerrainGrid, TerrainType};
use primordium_data::{Entity, Metabolism};
use rayon::prelude::*;
use uuid::Uuid;

/// Phase 62: Outpost Power Grid (Civ Level 2)
/// Connected outposts (via canals/rivers) automatically balance and share energy stores.
pub fn resolve_power_grid(terrain: &mut TerrainGrid, width: u16, height: u16) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();
    if outpost_indices.len() < 2 {
        return;
    }

    let mut parent: Vec<usize> = (0..terrain.cells.len()).collect();
    fn find(i: usize, p: &mut [usize]) -> usize {
        if p[i] == i {
            i
        } else {
            p[i] = find(p[i], p);
            p[i]
        }
    }
    let union = |i: usize, j: usize, p: &mut [usize]| {
        let root_i = find(i, p);
        let root_j = find(j, p);
        if root_i != root_j {
            p[root_i] = root_j;
        }
    };

    for y in 0..height {
        for x in 0..width {
            let idx = (y as usize * width as usize) + x as usize;
            let cell = &terrain.cells[idx];
            if matches!(cell.terrain_type, TerrainType::Outpost | TerrainType::River) {
                for (dx, dy) in &[(1, 0), (0, 1), (1, 1), (1, -1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let nidx = (ny as usize * width as usize) + nx as usize;
                        let ncell = &terrain.cells[nidx];
                        if matches!(
                            ncell.terrain_type,
                            TerrainType::Outpost | TerrainType::River
                        ) {
                            union(idx, nidx, &mut parent);
                        }
                    }
                }
            }
        }
    }

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &idx in &outpost_indices {
        let root = find(idx, &mut parent);
        groups.entry(root).or_default().push(idx);
    }

    for group in groups.values() {
        if group.len() > 1 {
            let total_energy: f32 = group.iter().map(|&i| terrain.cells[i].energy_store).sum();
            let avg_energy = total_energy / group.len() as f32;
            for &i in group {
                let current = terrain.cells[i].energy_store;
                let flow = (avg_energy - current) * 0.1;
                terrain.cells[i].energy_store += flow;
            }
        }
    }
}

pub fn count_outposts_by_lineage(terrain: &TerrainGrid) -> std::collections::HashMap<Uuid, usize> {
    let mut counts = std::collections::HashMap::new();
    let outpost_indices = &terrain.outpost_indices;
    for &idx in outpost_indices {
        let cell = &terrain.cells[idx];
        if let Some(id) = cell.owner_id {
            *counts.entry(id).or_insert(0) += 1;
        }
    }
    counts
}

pub enum OutpostAction {
    TransferEnergy {
        entity_idx: usize,
        amount: f64,
        outpost_idx: usize,
    },
}

pub fn handle_outposts_ecs(
    terrain: &mut TerrainGrid,
    world: &mut hecs::World,
    entity_handles: &[hecs::Entity],
    spatial_hash: &SpatialHash,
    snapshots: &[crate::model::world::InternalEntitySnapshot],
    width: u16,
    silo_cap: f32,
    outpost_cap: f32,
) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    let actions: Vec<OutpostAction> = outpost_indices
        .par_iter()
        .fold(Vec::new, |mut acc, &idx| {
            let (ox, oy) = ((idx % width as usize) as f64, (idx / width as usize) as f64);
            let owner_id = terrain.cells[idx].owner_id;
            let stored = terrain.cells[idx].energy_store;
            let spec = terrain.cells[idx].outpost_spec;

            spatial_hash.query_callback(ox, oy, 3.0, |e_idx| {
                let snap = &snapshots[e_idx];
                if Some(snap.lineage_id) == owner_id {
                    match spec {
                        OutpostSpecialization::Silo => {
                            if snap.energy > 50.0 {
                                // Estimation of surplus
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: -snap.energy * 0.1,
                                    outpost_idx: idx,
                                });
                            }
                        }
                        OutpostSpecialization::Nursery => {
                            if snap.energy < 30.0 && stored > 20.0 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: 20.0,
                                    outpost_idx: idx,
                                });
                            }
                        }
                        _ => {
                            if snap.energy > 80.0 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: -snap.energy * 0.05,
                                    outpost_idx: idx,
                                });
                            } else if snap.energy < 20.0 && stored > 10.0 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: 10.0,
                                    outpost_idx: idx,
                                });
                            }
                        }
                    }
                }
            });
            acc
        })
        .reduce(Vec::new, |mut a, b| {
            a.extend(b);
            a
        });

    for action in actions {
        match action {
            OutpostAction::TransferEnergy {
                entity_idx,
                amount,
                outpost_idx,
            } => {
                let handle = entity_handles[entity_idx];
                if let Ok(mut met) = world.get::<&mut Metabolism>(handle) {
                    let actual_transfer = if amount > 0.0 {
                        amount.min(terrain.cells[outpost_idx].energy_store as f64)
                    } else {
                        amount
                    };
                    met.energy = (met.energy + actual_transfer).clamp(0.0, met.max_energy);
                    terrain.cells[outpost_idx].energy_store -= actual_transfer as f32;
                }
            }
        }
    }

    for &idx in &outpost_indices {
        let max_cap = match terrain.cells[idx].outpost_spec {
            OutpostSpecialization::Silo => silo_cap,
            _ => outpost_cap,
        };
        terrain.cells[idx].energy_store = terrain.cells[idx].energy_store.clamp(0.0, max_cap);
    }
}

pub fn handle_outposts(
    terrain: &mut TerrainGrid,
    entities: &mut [Entity],
    spatial_hash: &SpatialHash,
    width: u16,
    silo_cap: f32,
    outpost_cap: f32,
) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    for &idx in &outpost_indices {
        let (ox, oy) = ((idx % width as usize) as f64, (idx / width as usize) as f64);
        let owner_id = terrain.cells[idx].owner_id;

        let mut stored = terrain.cells[idx].energy_store;
        let spec = terrain.cells[idx].outpost_spec;

        spatial_hash.query_callback(ox, oy, 3.0, |e_idx| {
            let e = &mut entities[e_idx];
            if Some(e.metabolism.lineage_id) == owner_id {
                match spec {
                    OutpostSpecialization::Silo => {
                        if e.metabolism.energy > e.metabolism.max_energy * 0.5 {
                            let donation = e.metabolism.energy * 0.1;
                            e.metabolism.energy -= donation;
                            stored += donation as f32;
                        }
                    }
                    OutpostSpecialization::Nursery => {
                        if e.metabolism.energy < e.metabolism.max_energy * 0.5 && stored > 20.0 {
                            let grant = (e.metabolism.max_energy * 0.2).min(stored as f64);
                            e.metabolism.energy += grant;
                            stored -= grant as f32;
                        }
                    }
                    _ => {
                        if e.metabolism.energy > e.metabolism.max_energy * 0.8 {
                            let donation = e.metabolism.energy * 0.05;
                            e.metabolism.energy -= donation;
                            stored += donation as f32;
                        }
                        if e.metabolism.energy < e.metabolism.max_energy * 0.3 && stored > 10.0 {
                            let grant = (e.metabolism.max_energy * 0.1).min(stored as f64);
                            e.metabolism.energy += grant;
                            stored -= grant as f32;
                        }
                    }
                }
            }
        });

        let max_cap = match spec {
            OutpostSpecialization::Silo => silo_cap,
            _ => outpost_cap,
        };
        terrain.cells[idx].energy_store = stored.min(max_cap);
    }
}
