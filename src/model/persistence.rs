use crate::model::world::World;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const CURRENT_SAVE_VERSION: u32 = 1;

/// Struct used for saving the world state with versioning metadata.
/// Holds a reference to the world to avoid cloning.
#[derive(Serialize)]
pub struct SaveStateRef<'a> {
    pub version: u32,
    pub world: &'a World,
}

/// Struct used for loading the world state.
/// owns the world data.
#[derive(Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub world: World,
}

/// Saves the world to a file with versioning metadata.
pub fn save_world(world: &mut World, path: impl AsRef<Path>) -> Result<()> {
    // Ensure transient state is prepared for serialization
    world.prepare_for_save();

    let state = SaveStateRef {
        version: CURRENT_SAVE_VERSION,
        world,
    };

    let data = serde_json::to_string_pretty(&state).context("Failed to serialize save state")?;

    fs::write(path, data).context("Failed to write save file")?;

    Ok(())
}

/// Loads the world from a file, handling version migration.
pub fn load_world(path: impl AsRef<Path>) -> Result<World> {
    let content = fs::read_to_string(&path).context("Failed to read save file")?;

    // First try to deserialize as the current versioned format
    match serde_json::from_str::<SaveState>(&content) {
        Ok(state) => {
            // Check version and migrate if necessary
            match state.version {
                1 => {
                    let mut world = state.world;
                    world.post_load();
                    Ok(world)
                }
                v if v > CURRENT_SAVE_VERSION => {
                    anyhow::bail!(
                        "Save file version {} is newer than supported version {}",
                        v,
                        CURRENT_SAVE_VERSION
                    );
                }
                _ => {
                    // Future migration logic would go here
                    // e.g. migrate_v1_to_v2(state.world)
                    anyhow::bail!("Unsupported save version: {}", state.version);
                }
            }
        }
        Err(_) => {
            // If that fails, assume it's a legacy (v0) save file containing just the World
            tracing::info!("Failed to load as versioned save, attempting legacy load...");
            let mut world: World =
                serde_json::from_str(&content).context("Failed to deserialize legacy save file")?;

            world.post_load();
            tracing::info!("Legacy save loaded successfully");
            Ok(world)
        }
    }
}
