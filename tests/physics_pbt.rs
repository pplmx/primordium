use primordium_core::spatial_hash::SpatialHash;
use primordium_lib::model::lifecycle;
use proptest::prelude::*;

prop_compose! {
    fn arb_position()(
        x in 0.0f64..100.0,
        y in 0.0f64..100.0
    ) -> (f64, f64) {
        (x, y)
    }
}

prop_compose! {
    fn arb_energy_bounds()(
        max_energy in 100.0f64..3000.0
    ) -> f64 {
        max_energy
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_spatial_hash_cell_index_consistency(
        (x, y) in arb_position(),
        cell_size in 5.0f64..20.0
    ) {
        let width = 100;
        let height = 100;
        let spatial = SpatialHash::new(cell_size, width, height);

        if x.is_finite() && y.is_finite() && x >= 0.0 && y < width as f64 && y >= 0.0 && y < height as f64 {
            if let Some(cell) = spatial.get_cell_idx(x, y) {
                prop_assert!(cell < spatial.cell_offsets.len() - 1,
                    "Cell index {} out of range [0, {})", cell, spatial.cell_offsets.len() - 1);
            }
        }
    }

    #[test]
    fn test_entity_physics_positions_finite(
        x in 0.0f64..100.0,
        y in 0.0f64..100.0,
        vx in -5.0f64..5.0,
        vy in -5.0f64..5.0
    ) {
        let mut e = lifecycle::create_entity(x, y, 0);
        e.physics.vx = vx;
        e.physics.vy = vy;

        prop_assert!(e.physics.x.is_finite(), "x coordinate must be finite");
        prop_assert!(e.physics.y.is_finite(), "y coordinate must be finite");
        prop_assert!(e.physics.vx.is_finite(), "vx must be finite");
        prop_assert!(e.physics.vy.is_finite(), "vy must be finite");
    }

    #[test]
    fn test_entity_energy_consistency(
        max_energy in 100.0f64..3000.0
    ) {
        let mut e = lifecycle::create_entity(50.0, 50.0, 0);
        e.metabolism.max_energy = max_energy;

        prop_assert!(e.metabolism.energy >= 0.0, "Energy must be non-negative");
        prop_assert!(e.metabolism.max_energy > 0.0, "Max energy must be positive");
        prop_assert!(e.metabolism.max_energy >= 50.0, "Max energy should be >= 50.0");
    }
}
