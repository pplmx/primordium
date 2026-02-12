use primordium_core::terrain::TerrainGrid;
use primordium_data::TerrainType;

#[test]
fn test_terrain_negative_coordinates_wrapping_fix() {
    let terrain = TerrainGrid::generate(50, 50, 42);

    let cell = terrain.get(-10.0, -10.0);
    assert_eq!(cell.terrain_type, TerrainType::Plains);

    let cell = terrain.get(-0.5, 50.0);
    assert_eq!(cell.terrain_type, TerrainType::Plains);

    let cell = terrain.get(50.0, -0.5);
    assert_eq!(cell.terrain_type, TerrainType::Plains);
}

#[test]
fn test_terrain_boundary_clamping() {
    let terrain = TerrainGrid::generate(50, 50, 42);

    let cell = terrain.get(-100.0, 50.0);
    assert_eq!(
        cell.terrain_type,
        TerrainType::Plains,
        "Negative coordinates should clamp to (0,0)"
    );

    let cell = terrain.get(1000.0, 50.0);
    assert!(
        cell.terrain_type == TerrainType::Plains,
        "Coordinates out of bounds should return Plains"
    );

    let cell = terrain.get(-10.0, -10.0);
    assert_eq!(
        cell.terrain_type,
        TerrainType::Plains,
        "Both negative coordinates should clamp to (0,0)"
    );
}

#[test]
fn test_terrain_cell_unsigned_coord_handling() {
    let mut terrain = TerrainGrid::generate(2, 2, 42);
    terrain.set_cell_type(1, 1, TerrainType::Mountain);

    let cell = terrain.get_cell(u16::MAX, u16::MAX);
    assert_eq!(cell.terrain_type, TerrainType::Mountain);
}

#[test]
fn test_terrain_extreme_values() {
    let terrain = TerrainGrid::generate(50, 50, 42);

    let cell = terrain.get(f64::MAX, f64::MAX);
    assert_eq!(cell.terrain_type, TerrainType::Plains);

    let cell = terrain.get(f64::MIN, f64::MIN);
    assert_eq!(cell.terrain_type, TerrainType::Plains);

    let cell = terrain.get(f64::NAN, 50.0);
    assert_eq!(cell.terrain_type, TerrainType::Plains);
}

#[test]
fn test_terrain_movement_modifier_robustness() {
    let terrain = TerrainGrid::generate(50, 50, 42);

    assert!(terrain.movement_modifier(-10.0, 50.0) >= 0.0);
    assert!(terrain.movement_modifier(50.0, -10.0) >= 0.0);
    assert!(terrain.movement_modifier(f64::NAN, 50.0) >= 0.0);
}

#[test]
fn test_terrain_food_spawn_modifier_robustness() {
    let terrain = TerrainGrid::generate(50, 50, 42);

    assert!(terrain.food_spawn_modifier(-10.0, 50.0) >= 0.0);
    assert!(terrain.food_spawn_modifier(50.0, -10.0) >= 0.0);
    assert!(terrain.food_spawn_modifier(f64::INFINITY, 50.0) >= 0.0);
}
