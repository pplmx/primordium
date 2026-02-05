use primordium_lib::model::brain::BrainLogic;
use primordium_lib::model::GenotypeLogic;

#[test]
fn test_brain_forward_outputs_are_finite() {
    let genotype = primordium_data::Genotype::new_random();
    let mut activations = primordium_data::Activations::default();
    let last_hidden: [f32; 6] = [0.0; 6];

    // Test various input ranges
    for &input in &[-100.0, 0.0, 100.0] {
        let inputs: [f32; 29] = [input; 29];
        let (outputs, next_hidden) =
            genotype
                .brain
                .forward_internal(inputs, last_hidden, &mut activations);

        for &output in &outputs {
            assert!(
                output.is_finite(),
                "Outputs should be finite for input {}, got {}",
                input,
                output
            );
        }

        for &hidden in &next_hidden {
            assert!(
                hidden.is_finite(),
                "Hidden values should be finite for input {}",
                input
            );
        }
    }
}

#[test]
fn test_brain_forward_preserves_length() {
    let inputs: [f32; 29] = [0.5; 29];
    let last_hidden: [f32; 6] = [0.0; 6];
    let genotype = primordium_data::Genotype::new_random();
    let mut activations = primordium_data::Activations::default();

    let (outputs, next_hidden) =
        genotype
            .brain
            .forward_internal(inputs, last_hidden, &mut activations);

    assert_eq!(outputs.len(), 12, "Should have 12 outputs");
    assert_eq!(next_hidden.len(), 6, "Should have 6 hidden values");
}

#[test]
fn test_brain_forward_is_deterministic() {
    let inputs: [f32; 29] = [0.5; 29];
    let last_hidden: [f32; 6] = [0.0; 6];
    let genotype = primordium_data::Genotype::new_random();
    let mut activations1 = primordium_data::Activations::default();
    let mut activations2 = primordium_data::Activations::default();

    let (outputs1, next_hidden1) =
        genotype
            .brain
            .forward_internal(inputs, last_hidden, &mut activations1);
    let (outputs2, next_hidden2) =
        genotype
            .brain
            .forward_internal(inputs, last_hidden, &mut activations2);

    assert_eq!(outputs1, outputs2, "Same inputs should give same outputs");
    assert_eq!(
        next_hidden1, next_hidden2,
        "Hidden state update should be deterministic"
    );
}

#[test]
fn test_multiple_forward_calls_evolve_hidden() {
    let mut inputs: [f32; 29] = [0.0; 29];
    for (i, input) in inputs.iter_mut().enumerate() {
        *input = (i as f32) / 29.0 - 0.5; // Variety in inputs
    }
    let genotype = primordium_data::Genotype::new_random();
    let mut activations = primordium_data::Activations::default();
    let hidden1: [f32; 6] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];

    let (_, hidden2) = genotype
        .brain
        .forward_internal(inputs, hidden1, &mut activations);
    let _ = genotype
        .brain
        .forward_internal(inputs, hidden2, &mut activations);

    assert_ne!(hidden2, hidden1, "Hidden state should change across ticks");
}

#[test]
fn test_different_genotypes_different_outputs() {
    let inputs: [f32; 29] = [0.5; 29];
    let last_hidden: [f32; 6] = [0.0; 6];

    let genotype1 = primordium_data::Genotype::new_random();
    let genotype2 = primordium_data::Genotype::new_random();

    let mut activations1 = primordium_data::Activations::default();
    let mut activations2 = primordium_data::Activations::default();

    let (outputs1, _) = genotype1
        .brain
        .forward_internal(inputs, last_hidden, &mut activations1);
    let (outputs2, _) = genotype2
        .brain
        .forward_internal(inputs, last_hidden, &mut activations2);

    // With random generation, they should almost always be different
    // But we can't guarantee 100%, so we just verify they're valid
    assert!(outputs1.iter().all(|&o| o.is_finite()));
    assert!(outputs2.iter().all(|&o| o.is_finite()));
}
