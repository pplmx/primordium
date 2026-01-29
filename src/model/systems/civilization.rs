use crate::model::spatial_hash::SpatialHash;
use crate::model::terrain::{OutpostSpecialization, TerrainGrid, TerrainType};
use primordium_data::Entity;
use std::collections::VecDeque;
use uuid::Uuid;

/// Phase 62: Outpost Power Grid (Civ Level 2)
/// Connected outposts (via canals/rivers) automatically balance and share energy stores.
pub fn resolve_power_grid(terrain: &mut TerrainGrid, width: u16, height: u16) {
    let cell_count = terrain.cells.len();
    let mut visited = vec![false; cell_count];
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    for &start_idx in &outpost_indices {
        if visited[start_idx] {
            continue;
        }

        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_idx);
        visited[start_idx] = true;

        while let Some(current) = queue.pop_front() {
            group.push(current);
            let cx = (current % width as usize) as i32;
            let cy = (current / width as usize) as i32;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let nidx = (ny as usize * width as usize) + nx as usize;
                        if !visited[nidx] {
                            let cell = &terrain.cells[nidx];
                            if matches!(
                                cell.terrain_type,
                                TerrainType::Outpost | TerrainType::River
                            ) {
                                visited[nidx] = true;
                                queue.push_back(nidx);
                            }
                        }
                    }
                }
            }
        }

        let outpost_group: Vec<usize> = group
            .into_iter()
            .filter(|&i| matches!(terrain.cells[i].terrain_type, TerrainType::Outpost))
            .collect();

        if outpost_group.len() > 1 {
            // Phase 63: Resource Pipelining (Energy Flow)
            let total_energy: f32 = outpost_group
                .iter()
                .map(|&i| terrain.cells[i].energy_store)
                .sum();
            let avg_energy = total_energy / outpost_group.len() as f32;
            for &i in &outpost_group {
                // Flow towards equilibrium
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

pub fn handle_outposts_ecs(
    terrain: &mut TerrainGrid,
    world: &mut hecs::World,
    entity_handles: &[hecs::Entity],
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
            let handle = entity_handles[e_idx];
            if let Ok(mut metabolism) = world.get::<&mut primordium_data::Metabolism>(handle) {
                if Some(metabolism.lineage_id) == owner_id {
                    match spec {
                        OutpostSpecialization::Silo => {
                            if metabolism.energy > metabolism.max_energy * 0.5 {
                                let donation = metabolism.energy * 0.1;
                                metabolism.energy -= donation;
                                stored += donation as f32;
                            }
                        }
                        OutpostSpecialization::Nursery => {
                            if metabolism.energy < metabolism.max_energy * 0.5 && stored > 20.0 {
                                let grant = (metabolism.max_energy * 0.2).min(stored as f64);
                                metabolism.energy += grant;
                                stored -= grant as f32;
                            }
                        }
                        _ => {
                            if metabolism.energy > metabolism.max_energy * 0.8 {
                                let donation = metabolism.energy * 0.05;
                                metabolism.energy -= donation;
                                stored += donation as f32;
                            }
                            if metabolism.energy < metabolism.max_energy * 0.3 && stored > 10.0 {
                                let grant = (metabolism.max_energy * 0.1).min(stored as f64);
                                metabolism.energy += grant;
                                stored -= grant as f32;
                            }
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

/// Phase 63: Outpost Specialization Logic
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
                        // Collect more surplus
                        if e.metabolism.energy > e.metabolism.max_energy * 0.5 {
                            let donation = e.metabolism.energy * 0.1;
                            e.metabolism.energy -= donation;
                            stored += donation as f32;
                        }
                    }
                    OutpostSpecialization::Nursery => {
                        // Grants more to needy
                        if e.metabolism.energy < e.metabolism.max_energy * 0.5 && stored > 20.0 {
                            let grant = (e.metabolism.max_energy * 0.2).min(stored as f64);
                            e.metabolism.energy += grant;
                            stored -= grant as f32;
                        }
                    }
                    _ => {
                        // Default Outpost Logic
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
