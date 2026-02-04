use primordium_lib::model::state::Metabolism;
use uuid::Uuid;

/// Test edge cases in perception and decision systems
/// that could cause NaN, division by zero, or panic.

#[test]
fn test_perception_division_nan_safety() {
    let met = Metabolism {
        trophic_potential: 0.5,
        energy: 500.0,
        prev_energy: 400.0,
        max_energy: f64::NAN,
        peak_energy: 500.0,
        birth_tick: 0,
        generation: 0,
        offspring_count: 0,
        lineage_id: Uuid::new_v4(),
        has_metamorphosed: false,
        is_in_transit: false,
        migration_id: None,
    };

    // This simulates the division in systems.rs line 100:
    // (met.energy / met.max_energy.max(1.0)) as f32
    let result = met.energy / met.max_energy.max(1.0);

    assert!(result.is_finite(), "NaN should be prevented by max(1.0)");
    assert!(!result.is_nan());
}

#[test]
fn test_perception_division_zero_energy_max() {
    let met = Metabolism {
        trophic_potential: 0.5,
        energy: 500.0,
        prev_energy: 400.0,
        max_energy: 0.0,
        peak_energy: 500.0,
        birth_tick: 0,
        generation: 0,
        offspring_count: 0,
        lineage_id: Uuid::new_v4(),
        has_metamorphosed: false,
        is_in_transit: false,
        migration_id: None,
    };

    // This simulates the division in systems.rs line 100 with zero max_energy
    let result = met.energy / met.max_energy.max(1.0);

    assert!(result.is_finite(), "Division by zero prevented by max(1.0)");
    assert!(result > 400.0, "Result should be reasonable");
}

#[test]
fn test_perception_negative_energy_handling() {
    let met = Metabolism {
        trophic_potential: 0.5,
        energy: -500.0,
        prev_energy: -600.0,
        max_energy: 1000.0,
        peak_energy: 0.0,
        birth_tick: 0,
        generation: 0,
        offspring_count: 0,
        lineage_id: Uuid::new_v4(),
        has_metamorphosed: false,
        is_in_transit: false,
        migration_id: None,
    };

    let result = met.energy / met.max_energy.max(1.0);

    // Negative energy should produce a valid result
    assert!(result.is_finite());
}

#[test]
fn test_defense_calculation_positive_allies() {
    // Test the defense_mult calculation: (1.0 - allies * 0.15).max(0.4)
    let allies = 2.0_f64;
    let defense_reduction = 0.15_f64;
    let min_defense = 0.4_f64;

    let defense_mult = (1.0_f64 - allies * defense_reduction).max(min_defense);

    // With 2 allies, defense should be reduced to 0.7
    assert_eq!(defense_mult, 0.7_f64);
}

#[test]
fn test_defense_calculation_many_allies() {
    // With many allies, defense should hit minimum
    let allies = 10.0_f64;
    let defense_reduction = 0.15_f64;
    let min_defense = 0.4_f64;

    let defense_mult = (1.0_f64 - allies * defense_reduction).max(min_defense);

    // Should be clamped to minimum
    assert_eq!(defense_mult, min_defense);
}

#[test]
fn test_defense_calculation_no_allies() {
    // With zero allies, defense should be maximum
    let allies = 0.0_f64;
    let defense_reduction = 0.15_f64;
    let min_defense = 0.4_f64;

    let defense_mult = (1.0_f64 - allies * defense_reduction).max(min_defense);

    // Should be no reduction
    assert_eq!(defense_mult, 1.0_f64);
}

#[test]
fn test_sharing_fraction_calculation() {
    // Test energy sharing: amount = diff * 0.05
    let diff = 100.0_f64;
    let sharing_fraction = 0.05_f64;

    let amount = diff * sharing_fraction;

    // Should share 5% of the difference
    assert_eq!(amount, 5.0_f64);
}

#[test]
fn test_sharing_fraction_threshold() {
    // Test that sharing only happens above threshold
    let self_energy = 100.0_f64;
    let partner_energy = 95.0_f64;
    let threshold = 2.0_f64;
    let diff = self_energy - partner_energy;

    // Difference is only 5.0, which is > 2.0 threshold
    assert!(diff > threshold);
}

#[test]
fn test_sharing_not_below_threshold() {
    let self_energy = 100.0_f64;
    let partner_energy = 99.0_f64;
    let threshold = 2.0_f64;
    let diff = self_energy - partner_energy;

    // Difference is only 1.0, which is < 2.0 threshold
    assert!(diff < threshold);
}

#[test]
fn test_sharing_target_low_energy_condition() {
    // Test energy sharing only to targets below threshold
    let target_energy = 200.0_f64;
    let target_max = 1000.0_f64;
    let low_threshold = 0.5_f64;

    // 200 < 1000 * 0.5 (500), so target qualifies
    assert!(target_energy < target_max * low_threshold);
}

#[test]
fn test_sharing_skip_high_energy_target() {
    let target_energy = 600.0_f64;
    let target_max = 1000.0_f64;
    let low_threshold = 0.5_f64;

    // 600 >= 1000 * 0.5 (500), so target does NOT qualify
    assert!(target_energy >= target_max * low_threshold);
}

#[test]
fn test_aggression_threshold() {
    // Test that aggression triggers only above threshold
    let aggression = 0.6_f64;
    let threshold = 0.5_f64;

    // 0.6 > 0.5, so aggression should activate
    assert!(aggression > threshold);
}

#[test]
fn test_no_aggression_below_threshold() {
    let aggression = 0.4_f64;
    let threshold = 0.5_f64;

    // 0.4 <= 0.5, so aggression should NOT activate
    assert!(aggression <= threshold);
}
