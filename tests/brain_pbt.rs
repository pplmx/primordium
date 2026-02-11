use primordium_lib::model::brain::{Brain, BrainLogic, Connection};
use primordium_lib::model::state::entity::Genotype;
use proptest::prelude::*;
use uuid::Uuid;

// Strategies for generating arbitrary brain components
prop_compose! {
    fn arb_connection(max_node: usize)(
        from in 0..max_node,
        to in 0..max_node,
        weight in -10.0f32..10.0f32,
        enabled in any::<bool>(),
        innovation in any::<usize>()
    ) -> Connection {
        Connection { from, to, weight, enabled, innovation }
    }
}

prop_compose! {
    fn arb_brain(max_conns: usize)(
        connections in prop::collection::vec(arb_connection(47), 0..max_conns)
    ) -> Brain {
        let mut brain = Brain::new_random(); // Correct base initialization
        brain.connections = connections;
        brain.initialize_node_idx_map();
        brain
    }
}

prop_compose! {
    fn arb_genotype()(
        brain in arb_brain(100),
        sensing_range in 1.0f64..20.0f64,
        max_speed in 0.1f64..5.0f64,
        max_energy in 50.0f64..2000.0f64,
        metabolic_niche in 0.0f32..1.0f32,
        trophic_potential in 0.0f32..1.0f32,
        reproductive_investment in 0.1f32..0.9f32,
        maturity_gene in 0.5f32..2.0f32
    ) -> Genotype {
        Genotype {
            brain,
            sensing_range,
            max_speed,
            max_energy,
            lineage_id: Uuid::new_v4(),
            metabolic_niche,
            trophic_potential,
            reproductive_investment,
            maturity_gene,
            mate_preference: 0.5,
            pairing_bias: 0.5,
            specialization_bias: [0.33, 0.33, 0.34],
            regulatory_rules: Vec::new(),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_brain_forward_no_nan(
        brain in arb_brain(50),
        inputs in any::<[f32; 29]>() // Fixed input array generation
    ) {
        let mut activations = primordium_data::Activations::default();
        let (outputs, next_hidden) = brain.forward_internal(inputs, [0.0; 6], &mut activations);

        for &o in &outputs {
            prop_assert!(o.is_finite(), "Brain output produced non-finite value: {}", o);
        }
        for &h in &next_hidden {
            prop_assert!(h.is_finite(), "Brain hidden state produced non-finite value: {}", h);
        }
    }

    #[test]
    fn test_genotype_hex_roundtrip(genotype in arb_genotype()) {
        let hex = genotype.to_hex();
        let mut decoded = Genotype::from_hex(&hex).expect("Failed to decode HexDNA");
        decoded.brain.initialize_node_idx_map();

        // Use epsilon for float comparisons due to JSON serialization precision
        prop_assert!((genotype.sensing_range - decoded.sensing_range).abs() < 1e-10);
        prop_assert!((genotype.max_speed - decoded.max_speed).abs() < 1e-10);
        prop_assert!((genotype.max_energy - decoded.max_energy).abs() < 1e-10);

        prop_assert_eq!(genotype.lineage_id, decoded.lineage_id);
        prop_assert_eq!(genotype.metabolic_niche, decoded.metabolic_niche);
        prop_assert_eq!(genotype.trophic_potential, decoded.trophic_potential);
        prop_assert_eq!(genotype.reproductive_investment, decoded.reproductive_investment);
        prop_assert_eq!(genotype.maturity_gene, decoded.maturity_gene);

        // Brain comparison
        prop_assert_eq!(genotype.brain.nodes.len(), decoded.brain.nodes.len());
        prop_assert_eq!(genotype.brain.connections.len(), decoded.brain.connections.len());

        for (c1, c2) in genotype.brain.connections.iter().zip(decoded.brain.connections.iter()) {
            prop_assert_eq!(c1.from, c2.from);
            prop_assert_eq!(c1.to, c2.to);
            prop_assert!((c1.weight - c2.weight).abs() < 0.0001);
            prop_assert_eq!(c1.enabled, c2.enabled);
            prop_assert_eq!(c1.innovation, c2.innovation);
        }
    }

}
