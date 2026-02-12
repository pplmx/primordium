use primordium_core::spatial_hash::SpatialHash;

#[test]
fn test_spatial_hash_overflow_protection() {
    let sh = SpatialHash::new(5.0, 100, 100);

    // Test extreme positive values that would cause overflow
    let huge_x = i32::MAX as f64 * 10.0;
    let huge_y = i32::MAX as f64 * 10.0;

    let result = sh.get_cell_idx(huge_x, huge_y);
    assert!(
        result.is_none(),
        "Should return None for overflow coordinates"
    );

    // Test extreme negative values
    let huge_neg_x = i32::MIN as f64 * 10.0;
    let huge_neg_y = i32::MIN as f64 * 10.0;

    let result = sh.get_cell_idx(huge_neg_x, huge_neg_y);
    assert!(
        result.is_none(),
        "Should return None for negative overflow coordinates"
    );
}

#[test]
fn test_spatial_hash_nan_safety() {
    let sh = SpatialHash::new(5.0, 100, 100);

    let result = sh.get_cell_idx(f64::NAN, 50.0);
    assert!(result.is_none(), "Should return None for NaN x coordinate");

    let result = sh.get_cell_idx(50.0, f64::NAN);
    assert!(result.is_none(), "Should return None for NaN y coordinate");

    let result = sh.get_cell_idx(f64::NAN, f64::NAN);
    assert!(
        result.is_none(),
        "Should return None for NaN both coordinates"
    );
}

#[test]
fn test_spatial_hash_infinity_safety() {
    let sh = SpatialHash::new(5.0, 100, 100);

    let result = sh.get_cell_idx(f64::INFINITY, 50.0);
    assert!(
        result.is_none(),
        "Should return None for infinite x coordinate"
    );

    let result = sh.get_cell_idx(50.0, f64::INFINITY);
    assert!(
        result.is_none(),
        "Should return None for infinite y coordinate"
    );

    let result = sh.get_cell_idx(f64::NEG_INFINITY, f64::NEG_INFINITY);
    assert!(
        result.is_none(),
        "Should return None for negative infinity coordinates"
    );
}

#[test]
fn test_spatial_hash_boundary_conditions() {
    let sh = SpatialHash::new(5.0, 100, 100);

    // Test exact boundaries within valid range
    let result = sh.get_cell_idx(0.0, 0.0);
    assert!(result.is_some(), "Should handle (0, 0) coordinate");

    let result = sh.get_cell_idx(99.0, 99.0);
    assert!(
        result.is_some(),
        "Should handle boundary (99, 99) coordinate"
    );

    // Test outside width/height boundaries
    let result = sh.get_cell_idx(105.0, 50.0);
    assert!(result.is_none(), "Should return None for x beyond width");

    let result = sh.get_cell_idx(50.0, 105.0);
    assert!(result.is_none(), "Should return None for y beyond height");
}

#[test]
fn test_spatial_hash_count_nearby_with_overflow_protection() {
    let mut sh = SpatialHash::new(5.0, 100, 100);
    let positions = vec![(10.0, 10.0), (15.0, 15.0), (20.0, 20.0)];
    sh.build_parallel(&positions, 100, 100);

    // Test with valid coordinates
    let count = sh.count_nearby(15.0, 15.0, 20.0);
    assert!(count > 0, "Should count nearby entities");

    // Test with overflow coordinates returns 0
    let huge_x = i32::MAX as f64 * 10.0;
    let count = sh.count_nearby(huge_x, 15.0, 20.0);
    assert_eq!(count, 0, "Should return 0 for overflow coordinates");
}

#[test]
fn test_query_callback_rejects_invalid_radii() {
    let mut sh = SpatialHash::new(5.0, 100, 100);
    let positions = vec![(10.0, 10.0), (15.0, 15.0), (20.0, 20.0)];
    sh.build_parallel(&positions, 100, 100);

    let mut count = 0;

    sh.query_callback(15.0, 15.0, f64::NAN, |_| count += 1);
    assert_eq!(count, 0, "Should reject NaN radius");

    count = 0;
    sh.query_callback(15.0, 15.0, f64::INFINITY, |_| count += 1);
    assert_eq!(count, 0, "Should reject Infinity radius");

    count = 0;
    sh.query_callback(15.0, 15.0, -5.0, |_| count += 1);
    assert_eq!(count, 0, "Should reject negative radius");
}

#[test]
fn test_query_callback_with_valid_input() {
    let mut sh = SpatialHash::new(5.0, 100, 100);
    let positions = vec![(10.0, 10.0), (15.0, 15.0), (20.0, 20.0)];
    sh.build_parallel(&positions, 100, 100);

    let mut count = 0;
    sh.query_callback(15.0, 15.0, 5.0, |_| count += 1);
    assert!(count > 0, "Should count entities within valid radius");
}

#[test]
fn test_query_into_rejects_invalid_inputs() {
    let mut sh = SpatialHash::new(5.0, 100, 100);
    let positions = vec![(10.0, 10.0), (15.0, 15.0), (20.0, 20.0)];
    sh.build_parallel(&positions, 100, 100);

    let mut result = Vec::new();

    sh.query_into(f64::NAN, 15.0, 5.0, &mut result);
    assert_eq!(result.len(), 0, "Should reject NaN X");

    sh.query_into(15.0, f64::NAN, 5.0, &mut result);
    assert_eq!(result.len(), 0, "Should reject NaN Y");

    sh.query_into(15.0, 15.0, f64::NAN, &mut result);
    assert_eq!(result.len(), 0, "Should reject NaN radius");

    sh.query_into(15.0, 15.0, -5.0, &mut result);
    assert_eq!(result.len(), 0, "Should reject negative radius");
}

#[test]
fn test_count_nearby_kin_fast_with_invalid_inputs() {
    let mut sh = SpatialHash::new(5.0, 100, 100);
    let lid = uuid::Uuid::new_v4();
    let data = vec![(10.0, 10.0, lid), (15.0, 15.0, lid)];
    sh.build_with_lineage(&data, 100, 100);

    assert_eq!(sh.count_nearby_kin_fast(f64::NAN, 15.0, 5.0, lid), 0);
    assert_eq!(sh.count_nearby_kin_fast(15.0, f64::NAN, 5.0, lid), 0);
    assert_eq!(sh.count_nearby_kin_fast(15.0, 15.0, f64::NAN, lid), 0);
    assert_eq!(sh.count_nearby_kin_fast(15.0, 15.0, f64::INFINITY, lid), 0);
    assert_eq!(sh.count_nearby_kin_fast(15.0, 15.0, -5.0, lid), 0);
}
