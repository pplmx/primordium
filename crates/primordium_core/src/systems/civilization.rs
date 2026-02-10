use crate::lineage_registry::LineageRegistry;
use crate::spatial_hash::SpatialHash;
use crate::terrain::{OutpostSpecialization, TerrainGrid, TerrainType};
use primordium_data::{Entity, Metabolism};
use rayon::prelude::*;
use uuid::Uuid;

/// Phase 66: Contested Ownership Logic
/// Detects when enemy Alphas challenge outpost ownership and transfers
/// control if enemy tribal power significantly exceeds defender power.
pub fn resolve_contested_ownership(
    terrain: &mut TerrainGrid,
    width: u16,
    _height: u16,
    spatial_hash: &SpatialHash,
    snapshots: &[crate::snapshot::InternalEntitySnapshot],
    lineage_registry: &LineageRegistry,
) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    // Collect ownership transfers to apply
    let mut transfers: Vec<(usize, Option<Uuid>)> = Vec::new();

    for &idx in &outpost_indices {
        let owner_id = terrain.cells[idx].owner_id;
        let Some(current_owner) = owner_id else {
            continue;
        };

        let (ox, oy) = ((idx % width as usize) as f64, (idx / width as usize) as f64);

        // Calculate tribal power around the outpost (radius = 5)
        let mut power_map: std::collections::HashMap<Uuid, (f64, usize)> =
            std::collections::HashMap::new();

        spatial_hash.query_callback(ox, oy, 5.0, |e_idx| {
            let snap = &snapshots[e_idx];
            // Only count entities with sufficient energy (valid combatants)
            if snap.energy > 20.0 {
                let entry = power_map.entry(snap.lineage_id).or_insert((0.0, 0));
                entry.0 += snap.energy;
                entry.1 += 1;
            }
        });

        // Get current owner's power
        let (owner_energy, _owner_count) =
            power_map.get(&current_owner).copied().unwrap_or((0.0, 0));

        // Check if any enemy lineage significantly outpowers the owner
        let mut strongest_enemy: Option<(Uuid, f64, usize)> = None;
        let mut strongest_enemy_power = 0.0;

        for (lineage_id, (energy, count)) in &power_map {
            if *lineage_id == current_owner {
                continue;
            }

            // Get enemy civilization level (higher level = better organization)
            let enemy_level = lineage_registry
                .lineages
                .get(lineage_id)
                .map(|r| r.civilization_level)
                .unwrap_or(0);

            // Power factor considers both energy and civilization level
            // Level 2+ civilizations get 20% power bonus
            let power_factor = if enemy_level >= 2 { 1.2 } else { 1.0 };
            let adjusted_power = energy * power_factor;

            if adjusted_power > strongest_enemy_power {
                strongest_enemy_power = adjusted_power;
                strongest_enemy = Some((*lineage_id, *energy, *count));
            }
        }

        // Transfer ownership if enemy power > 2.5x owner power
        // AND enemy has at least 3 entities present
        if let Some((enemy_id, _enemy_energy, enemy_count)) = strongest_enemy {
            let owner_power = owner_energy.max(50.0); // Minimum defense threshold
            let power_ratio = strongest_enemy_power / owner_power;

            if power_ratio > 2.5 && enemy_count >= 3 {
                // Ownership transfer
                transfers.push((idx, Some(enemy_id)));

                // Clear energy store during transition (represents pillaging/disruption)
                terrain.cells[idx].energy_store *= 0.5;
            }
        }
    }

    // Apply transfers
    for (idx, new_owner) in transfers {
        terrain.cells[idx].owner_id = new_owner;
        // Reset specialization to Standard after takeover
        terrain.cells[idx].outpost_spec = OutpostSpecialization::Standard;
    }
}

/// Phase 66: Outpost Specialization Upgrades
/// Allows Level 2+ lineages to upgrade outposts to Silo or Nursery.
pub fn resolve_outpost_upgrades(
    terrain: &mut TerrainGrid,
    width: u16,
    _height: u16,
    spatial_hash: &SpatialHash,
    snapshots: &[crate::snapshot::InternalEntitySnapshot],
    lineage_registry: &LineageRegistry,
) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    for &idx in &outpost_indices {
        let owner_id = terrain.cells[idx].owner_id;
        let Some(current_owner) = owner_id else {
            continue;
        };

        // Check civilization level
        let civ_level = lineage_registry
            .lineages
            .get(&current_owner)
            .map(|r| r.civilization_level)
            .unwrap_or(0);

        // Only Level 2+ can upgrade
        if civ_level < 2 {
            continue;
        }

        let current_spec = terrain.cells[idx].outpost_spec;
        let stored = terrain.cells[idx].energy_store;

        let (ox, oy) = ((idx % width as usize) as f64, (idx / width as usize) as f64);

        // Calculate nearby tribal energy pool
        let mut nearby_tribal_energy: f64 = 0.0;
        let mut nearby_count: usize = 0;

        spatial_hash.query_callback(ox, oy, 4.0, |e_idx| {
            let snap = &snapshots[e_idx];
            if snap.lineage_id == current_owner {
                nearby_tribal_energy += snap.energy;
                nearby_count += 1;
            }
        });

        // Upgrade logic based on tribal needs and available resources
        match current_spec {
            OutpostSpecialization::Standard => {
                // Upgrade to Silo if: tribe has high energy surplus and outpost has space
                // Upgrade to Nursery if: tribe has many low-energy members
                let avg_tribe_energy = if nearby_count > 0 {
                    nearby_tribal_energy / nearby_count as f64
                } else {
                    0.0
                };

                let upgrade_cost: f32 = 200.0; // Energy cost to upgrade

                if stored > upgrade_cost {
                    // Decision: if tribe members are generally healthy (avg energy > 60),
                    // build Silo for storage. Otherwise build Nursery for healing.
                    if avg_tribe_energy > 60.0 {
                        // Upgrade to Silo
                        terrain.cells[idx].outpost_spec = OutpostSpecialization::Silo;
                        terrain.cells[idx].energy_store -= upgrade_cost;
                    } else if nearby_count >= 3 {
                        // Upgrade to Nursery (need enough members to benefit)
                        terrain.cells[idx].outpost_spec = OutpostSpecialization::Nursery;
                        terrain.cells[idx].energy_store -= upgrade_cost;
                    }
                }
            }
            OutpostSpecialization::Silo | OutpostSpecialization::Nursery => {
                // Already specialized - no automatic downgrade
                // Future: Could allow downgrading if severely damaged
            }
        }
    }
}

/// Phase 62: Outpost Power Grid (Civ Level 2)
/// Connected outposts (via canals/rivers) automatically balance and share energy stores.
pub fn resolve_power_grid(
    terrain: &mut TerrainGrid,
    width: u16,
    height: u16,
    lineage_registry: &LineageRegistry,
) {
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

            // Check if cell can participate in power grid
            let can_connect =
                if matches!(cell.terrain_type, TerrainType::Outpost | TerrainType::River) {
                    if cell.terrain_type == TerrainType::Outpost {
                        // Outposts need level 2+ owner to participate in power grid
                        if let Some(owner_id) = cell.owner_id {
                            let level = lineage_registry
                                .lineages
                                .get(&owner_id)
                                .map(|r| r.civilization_level)
                                .unwrap_or(0);
                            level >= 2
                        } else {
                            false // Unowned outposts don't connect
                        }
                    } else {
                        true // Rivers always connect
                    }
                } else {
                    false
                };

            if !can_connect {
                continue;
            }

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

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &idx in &outpost_indices {
        let root = find(idx, &mut parent);
        groups.entry(root).or_default().push(idx);
    }

    let group_data: Vec<Vec<usize>> = groups.into_values().filter(|g| g.len() > 1).collect();

    let terrain_ref = &*terrain;
    let changes: Vec<(usize, f32)> = group_data
        .par_iter()
        .flat_map(|group| {
            let total_energy: f32 = group
                .iter()
                .map(|&i| terrain_ref.cells[i].energy_store)
                .sum();
            let avg_energy = total_energy / group.len() as f32;

            group
                .iter()
                .map(move |&i| {
                    let current = terrain_ref.cells[i].energy_store;
                    let flow = (avg_energy - current) * 0.1;
                    (i, flow)
                })
                .collect::<Vec<_>>()
        })
        .collect();

    for (i, flow) in changes {
        terrain.cells[i].energy_store += flow;
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

pub struct OutpostContext<'a> {
    pub entity_handles: &'a [hecs::Entity],
    pub spatial_hash: &'a SpatialHash,
    pub snapshots: &'a [crate::snapshot::InternalEntitySnapshot],
    pub width: u16,
    pub silo_cap: f32,
    pub outpost_cap: f32,
}

pub fn handle_outposts_ecs(
    terrain: &mut TerrainGrid,
    world: &mut hecs::World,
    ctx: &OutpostContext<'_>,
) {
    let outpost_indices: Vec<usize> = terrain.outpost_indices.iter().copied().collect();

    let actions: Vec<OutpostAction> = outpost_indices
        .par_iter()
        .fold(Vec::new, |mut acc: Vec<OutpostAction>, &idx| {
            let (ox, oy) = (
                (idx % ctx.width as usize) as f64,
                (idx / ctx.width as usize) as f64,
            );
            let owner_id = terrain.cells[idx].owner_id;
            let stored = terrain.cells[idx].energy_store;
            let spec = terrain.cells[idx].outpost_spec;

            ctx.spatial_hash.query_callback(ox, oy, 3.0, |e_idx| {
                let snap = &ctx.snapshots[e_idx];
                if Some(snap.lineage_id) == owner_id {
                    match spec {
                        OutpostSpecialization::Silo => {
                            if snap.energy > snap.max_energy * 0.5 {
                                // Estimation of surplus
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: -snap.energy * 0.1,
                                    outpost_idx: idx,
                                });
                            }
                        }
                        OutpostSpecialization::Nursery => {
                            if snap.energy < snap.max_energy * 0.5 && stored > 20.0 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: (snap.max_energy * 0.2).min(stored as f64),
                                    outpost_idx: idx,
                                });
                            }
                        }
                        _ => {
                            if snap.energy > snap.max_energy * 0.8 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: -snap.energy * 0.05,
                                    outpost_idx: idx,
                                });
                            } else if snap.energy < snap.max_energy * 0.3 && stored > 10.0 {
                                acc.push(OutpostAction::TransferEnergy {
                                    entity_idx: e_idx,
                                    amount: (snap.max_energy * 0.1).min(stored as f64),
                                    outpost_idx: idx,
                                });
                            }
                        }
                    }
                }
            });
            acc
        })
        .reduce(Vec::new, |mut a: Vec<OutpostAction>, b| {
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
                let handle = ctx.entity_handles[entity_idx];
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
            OutpostSpecialization::Silo => ctx.silo_cap,
            _ => ctx.outpost_cap,
        };
        // Apply passive decay (entropy) to all outposts
        let decay = if terrain.cells[idx].owner_id.is_some() {
            0.05 // Maintained outposts decay slowly
        } else {
            0.5 // Abandoned outposts decay quickly
        };
        terrain.cells[idx].energy_store =
            (terrain.cells[idx].energy_store - decay).clamp(0.0, max_cap);
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
