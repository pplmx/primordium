use primordium_io::persistence::{load_rkyv, save_rkyv};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::world::World;
use std::fs;

#[tokio::test]
async fn test_world_snapshot_rkyv_roundtrip() {
    let mut config = AppConfig::default();
    config.world.width = 10;
    config.world.height = 10;
    let world = World::new(5, config).unwrap();

    let snapshot = world.create_snapshot(None);
    let path = "test_snapshot.rkyv";

    // Save
    save_rkyv(&snapshot, path).expect("Failed to save rkyv snapshot");

    // Load
    let loaded: primordium_lib::model::snapshot::WorldSnapshot =
        load_rkyv(path).expect("Failed to load rkyv snapshot");

    assert_eq!(loaded.tick, snapshot.tick);
    assert_eq!(loaded.entities.len(), snapshot.entities.len());
    assert_eq!(loaded.width, snapshot.width);

    // Cleanup
    let _ = fs::remove_file(path);
}
