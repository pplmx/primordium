use crate::brain::BrainLogic;
use primordium_data::Brain;
use rand::Rng;
use std::collections::HashMap;

pub fn brain_forward(
    brain: &Brain,
    inputs: [f32; 29],
    last_hidden: [f32; 6],
) -> ([f32; 12], [f32; 6]) {
    brain.forward(inputs, last_hidden)
}

pub fn mutate_brain<R: Rng>(
    brain: &mut Brain,
    config: &crate::config::AppConfig,
    specialization: Option<primordium_data::Specialization>,
    rng: &mut R,
) {
    brain.mutate_with_config(config, specialization, rng);
}

pub fn apply_grn_rules(
    genotype: &primordium_data::Genotype,
    metabolism: &primordium_data::Metabolism,
    oxygen_level: f64,
    carbon_level: f64,
    nearby_kin: usize,
    tick: u64,
) -> (f64, f64, f32) {
    let mut speed_mod = 1.0;
    let mut sensing_mod = 1.0;
    let mut repro_mod = 1.0;

    let energy_ratio = (metabolism.energy / metabolism.max_energy) as f32;
    let age_ratio = ((tick - metabolism.birth_tick) as f32 / 2000.0).min(1.0);

    for rule in &genotype.regulatory_rules {
        use primordium_data::{GeneType, RegulatoryOperator, RegulatorySensor};
        let sensor_value = match rule.sensor {
            RegulatorySensor::Oxygen => (oxygen_level / 21.0) as f32,
            RegulatorySensor::Carbon => (carbon_level / 1000.0) as f32,
            RegulatorySensor::EnergyRatio => energy_ratio,
            RegulatorySensor::NearbyKin => (nearby_kin as f32 / 10.0).min(1.0),
            RegulatorySensor::AgeRatio => age_ratio,
            RegulatorySensor::Clock => (tick % 1000) as f32 / 1000.0,
        };

        let triggered = match rule.operator {
            RegulatoryOperator::GreaterThan => sensor_value > rule.threshold,
            RegulatoryOperator::LessThan => sensor_value < rule.threshold,
        };

        if triggered {
            match rule.target {
                GeneType::Speed => speed_mod *= rule.modifier as f64,
                GeneType::Sensing => sensing_mod *= rule.modifier as f64,
                GeneType::ReproInvest => repro_mod *= rule.modifier,
                _ => {}
            }
        }
    }

    (speed_mod, sensing_mod, repro_mod)
}

pub fn mutate_genotype<R: Rng>(
    genotype: &mut primordium_data::Genotype,
    config: &crate::config::AppConfig,
    population: usize,
    is_radiation_storm: bool,
    specialization: Option<primordium_data::Specialization>,
    rng: &mut R,
    ancestral_genotype: Option<&primordium_data::Genotype>,
    stress_factor: f32,
) {
    let mut effective_mutation_rate = config.evolution.mutation_rate;
    let mut effective_mutation_amount = config.evolution.mutation_amount;

    if is_radiation_storm {
        effective_mutation_rate *= 5.0;
        effective_mutation_amount *= 2.0;
    }

    if let Some(ancestral) = ancestral_genotype {
        let recall_chance = 0.01 + (stress_factor * 0.05); // Up to 6% chance under high stress
        if rng.gen_bool(recall_chance as f64) {
            genotype.brain = ancestral.brain.clone();
            // Re-initialize map after brain replacement
            crate::brain::BrainLogic::initialize_node_idx_map(&mut genotype.brain);
        }
    }

    if config.evolution.population_aware && population > 0 {
        if population < config.evolution.bottleneck_threshold {
            let scaling =
                (config.evolution.bottleneck_threshold as f32 / population as f32).min(3.0);
            effective_mutation_rate *= scaling;
            effective_mutation_amount *= scaling;
        } else if population > config.evolution.stasis_threshold {
            effective_mutation_rate *= 0.5;
        }
    }

    let mut brain_config = config.clone();
    brain_config.evolution.mutation_rate = effective_mutation_rate;
    brain_config.evolution.mutation_amount = effective_mutation_amount;
    mutate_brain(&mut genotype.brain, &brain_config, specialization, rng);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.sensing_range = (genotype.sensing_range
            + rng.gen_range(-effective_mutation_amount as f64..effective_mutation_amount as f64)
                * genotype.sensing_range)
            .clamp(3.0, 15.0);
    }
    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.max_speed = (genotype.max_speed
            + rng.gen_range(-effective_mutation_amount as f64..effective_mutation_amount as f64)
                * genotype.max_speed)
            .clamp(0.5, 3.0);
    }

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.maturity_gene +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.maturity_gene = genotype.maturity_gene.clamp(0.5, 2.0);
    genotype.max_energy = (200.0 * genotype.maturity_gene as f64).clamp(100.0, 500.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.metabolic_niche +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.metabolic_niche = genotype.metabolic_niche.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.trophic_potential +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.trophic_potential = genotype.trophic_potential.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.reproductive_investment +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.reproductive_investment = genotype.reproductive_investment.clamp(0.1, 0.9);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.mate_preference +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.mate_preference = genotype.mate_preference.clamp(0.0, 1.0);

    if rng.gen::<f32>() < effective_mutation_rate {
        genotype.pairing_bias +=
            rng.gen_range(-effective_mutation_amount..effective_mutation_amount);
    }
    genotype.pairing_bias = genotype.pairing_bias.clamp(0.0, 1.0);

    for bias in &mut genotype.specialization_bias {
        if rng.gen::<f32>() < effective_mutation_rate {
            *bias = (*bias + rng.gen_range(-effective_mutation_amount..effective_mutation_amount))
                .clamp(0.0, 1.0);
        }
    }

    if rng.gen::<f32>() < effective_mutation_rate * 0.1 && !genotype.regulatory_rules.is_empty() {
        let idx = rng.gen_range(0..genotype.regulatory_rules.len());
        let rule = &mut genotype.regulatory_rules[idx];
        if rng.gen_bool(0.5) {
            rule.threshold += rng.gen_range(-0.1..0.1);
        } else {
            rule.modifier += rng.gen_range(-0.1..0.1);
        }
        rule.modifier = rule.modifier.clamp(0.1, 5.0);
    }

    if rng.gen::<f32>() < effective_mutation_rate * 0.05 {
        use primordium_data::{RegulatoryOperator, RegulatoryRule, RegulatorySensor};
        let sensors = [
            RegulatorySensor::Oxygen,
            RegulatorySensor::Carbon,
            RegulatorySensor::EnergyRatio,
            RegulatorySensor::NearbyKin,
            RegulatorySensor::AgeRatio,
        ];
        let targets = [
            primordium_data::GeneType::Sensing,
            primordium_data::GeneType::Speed,
            primordium_data::GeneType::ReproInvest,
        ];
        let new_rule = RegulatoryRule {
            sensor: sensors[rng.gen_range(0..sensors.len())],
            threshold: rng.gen_range(0.0..1.0),
            operator: if rng.gen_bool(0.5) {
                RegulatoryOperator::GreaterThan
            } else {
                RegulatoryOperator::LessThan
            },
            target: targets[rng.gen_range(0..targets.len())],
            modifier: rng.gen_range(0.5..1.5),
        };
        if genotype.regulatory_rules.len() < 5 {
            genotype.regulatory_rules.push(new_rule);
        }
    }

    if rng.gen::<f32>() < effective_mutation_rate * 0.02 && !genotype.regulatory_rules.is_empty() {
        let idx = rng.gen_range(0..genotype.regulatory_rules.len());
        genotype.regulatory_rules.remove(idx);
    }

    if population < 10 && population > 0 && rng.gen_bool(0.05) {
        match rng.gen_range(0..5) {
            0 => genotype.trophic_potential = rng.gen_range(0.0..1.0),
            1 => genotype.metabolic_niche = rng.gen_range(0.0..1.0),
            2 => genotype.mate_preference = rng.gen_range(0.0..1.0),
            3 => genotype.maturity_gene = rng.gen_range(0.5..2.0),
            _ => genotype.pairing_bias = rng.gen_range(0.0..1.0),
        }
    }
}

pub fn crossover_genotypes<R: Rng>(
    p1: &primordium_data::Genotype,
    p2: &primordium_data::Genotype,
    rng: &mut R,
) -> primordium_data::Genotype {
    let brain = crossover_brains(&p1.brain, &p2.brain, rng);

    primordium_data::Genotype {
        brain,
        sensing_range: if rng.gen_bool(0.5) {
            p1.sensing_range
        } else {
            p2.sensing_range
        },
        max_speed: if rng.gen_bool(0.5) {
            p1.max_speed
        } else {
            p2.max_speed
        },
        max_energy: if rng.gen_bool(0.5) {
            p1.max_energy
        } else {
            p2.max_energy
        },
        lineage_id: p1.lineage_id,
        metabolic_niche: if rng.gen_bool(0.5) {
            p1.metabolic_niche
        } else {
            p2.metabolic_niche
        },
        trophic_potential: if rng.gen_bool(0.5) {
            p1.trophic_potential
        } else {
            p2.trophic_potential
        },
        reproductive_investment: if rng.gen_bool(0.5) {
            p1.reproductive_investment
        } else {
            p2.reproductive_investment
        },
        maturity_gene: if rng.gen_bool(0.5) {
            p1.maturity_gene
        } else {
            p2.maturity_gene
        },
        mate_preference: if rng.gen_bool(0.5) {
            p1.mate_preference
        } else {
            p2.mate_preference
        },
        pairing_bias: if rng.gen_bool(0.5) {
            p1.pairing_bias
        } else {
            p2.pairing_bias
        },
        specialization_bias: if rng.gen_bool(0.5) {
            p1.specialization_bias
        } else {
            p2.specialization_bias
        },
        regulatory_rules: if rng.gen_bool(0.5) {
            p1.regulatory_rules.clone()
        } else {
            p2.regulatory_rules.clone()
        },
    }
}

pub fn crossover_brains<R: Rng>(p1: &Brain, p2: &Brain, rng: &mut R) -> Brain {
    let mut child_nodes = p1.nodes.clone();
    let mut child_connections = Vec::new();
    let mut map2 = std::collections::HashMap::new();
    for c in &p2.connections {
        map2.insert(c.innovation, c);
    }

    for c1 in &p1.connections {
        if let Some(c2) = map2.get(&c1.innovation) {
            if rng.gen_bool(0.5) {
                child_connections.push(c1.clone());
            } else {
                child_connections.push((*c2).clone());
            }
        } else {
            child_connections.push(c1.clone());
        }
    }

    let mut existing_node_ids: std::collections::HashSet<usize> =
        child_nodes.iter().map(|n| n.id).collect();

    let p2_node_map: std::collections::HashMap<usize, &primordium_data::Node> =
        p2.nodes.iter().map(|n| (n.id, n)).collect();

    for c in &child_connections {
        if !existing_node_ids.contains(&c.from) {
            if let Some(&n) = p2_node_map.get(&c.from) {
                child_nodes.push(n.clone());
                existing_node_ids.insert(c.from);
            }
        }
        if !existing_node_ids.contains(&c.to) {
            if let Some(&n) = p2_node_map.get(&c.to) {
                child_nodes.push(n.clone());
                existing_node_ids.insert(c.to);
            }
        }
    }

    Brain {
        nodes: child_nodes,
        connections: child_connections,
        next_node_id: p1.next_node_id.max(p2.next_node_id),
        learning_rate: if rng.gen_bool(0.5) {
            p1.learning_rate
        } else {
            p2.learning_rate
        },
        weight_deltas: HashMap::new(),
        node_idx_map: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use primordium_data::Genotype;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_test_genotype() -> Genotype {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        crate::brain::create_genotype_random_with_rng(&mut rng)
    }

    fn create_test_config() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn test_mutate_genotype_respects_clamping_bounds() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let config = create_test_config();
        let mut genotype = create_test_genotype();

        for _ in 0..100 {
            mutate_genotype(
                &mut genotype,
                &config,
                100,
                false,
                None,
                &mut rng,
                None,
                0.0,
            );
        }

        assert!(genotype.sensing_range >= 3.0 && genotype.sensing_range <= 15.0);
        assert!(genotype.max_speed >= 0.5 && genotype.max_speed <= 3.0);
        assert!(genotype.max_energy >= 100.0 && genotype.max_energy <= 500.0);
        assert!(genotype.maturity_gene >= 0.5 && genotype.maturity_gene <= 2.0);
        assert!(genotype.metabolic_niche >= 0.0 && genotype.metabolic_niche <= 1.0);
        assert!(genotype.trophic_potential >= 0.0 && genotype.trophic_potential <= 1.0);
        assert!(genotype.reproductive_investment >= 0.1 && genotype.reproductive_investment <= 0.9);
        assert!(genotype.mate_preference >= 0.0 && genotype.mate_preference <= 1.0);
        assert!(genotype.pairing_bias >= 0.0 && genotype.pairing_bias <= 1.0);
    }

    #[test]
    fn test_mutate_genotype_population_aware_bottleneck_scaling() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(999);
        let mut rng2 = ChaCha8Rng::seed_from_u64(999);
        let config = create_test_config();

        let mut genotype_bottleneck = create_test_genotype();
        let original_brain_bottleneck = genotype_bottleneck.brain.clone();

        let mut genotype_large = create_test_genotype();
        let original_brain_large = genotype_large.brain.clone();

        mutate_genotype(
            &mut genotype_bottleneck,
            &config,
            5,
            false,
            None,
            &mut rng1,
            None,
            0.0,
        );

        mutate_genotype(
            &mut genotype_large,
            &config,
            1000,
            false,
            None,
            &mut rng2,
            None,
            0.0,
        );

        let bottleneck_changes =
            count_weight_differences(&original_brain_bottleneck, &genotype_bottleneck.brain);
        let large_changes = count_weight_differences(&original_brain_large, &genotype_large.brain);

        assert!(bottleneck_changes >= large_changes);
    }

    #[test]
    fn test_mutate_genotype_radiation_storm_increases_mutation() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(777);
        let mut rng2 = ChaCha8Rng::seed_from_u64(777);
        let config = create_test_config();

        let mut genotype_normal = create_test_genotype();
        let mut genotype_storm = create_test_genotype();

        mutate_genotype(
            &mut genotype_normal,
            &config,
            100,
            false,
            None,
            &mut rng1,
            None,
            0.0,
        );

        mutate_genotype(
            &mut genotype_storm,
            &config,
            100,
            true,
            None,
            &mut rng2,
            None,
            0.0,
        );

        assert!(!genotype_storm.brain.connections.is_empty());
    }

    #[test]
    fn test_mutate_genotype_genetic_drift_in_small_population() {
        let config = create_test_config();

        let mut drift_occurred = false;
        for seed in 0..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut genotype = create_test_genotype();
            let original_trophic = genotype.trophic_potential;
            let original_niche = genotype.metabolic_niche;
            let original_mate = genotype.mate_preference;
            let original_maturity = genotype.maturity_gene;
            let original_pairing = genotype.pairing_bias;

            mutate_genotype(&mut genotype, &config, 5, false, None, &mut rng, None, 0.0);

            let trophic_drift = (genotype.trophic_potential - original_trophic).abs() > 0.3;
            let niche_drift = (genotype.metabolic_niche - original_niche).abs() > 0.3;
            let mate_drift = (genotype.mate_preference - original_mate).abs() > 0.3;
            let maturity_drift = (genotype.maturity_gene - original_maturity).abs() > 0.3;
            let pairing_drift = (genotype.pairing_bias - original_pairing).abs() > 0.3;

            if trophic_drift || niche_drift || mate_drift || maturity_drift || pairing_drift {
                drift_occurred = true;
                break;
            }
        }

        assert!(drift_occurred);
    }

    #[test]
    fn test_atavistic_recall_replaces_brain_with_ancestral() {
        let config = create_test_config();

        let mut ancestral = create_test_genotype();
        ancestral.brain.learning_rate = 0.999;

        let mut recall_occurred = false;
        for seed in 0..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut genotype = create_test_genotype();
            genotype.brain.learning_rate = 0.0;

            mutate_genotype(
                &mut genotype,
                &config,
                100,
                false,
                None,
                &mut rng,
                Some(&ancestral),
                1.0, // Full stress for max chance
            );

            if (genotype.brain.learning_rate - 0.999).abs() < 0.01 {
                recall_occurred = true;
                break;
            }
        }

        assert!(recall_occurred);
    }

    #[test]
    fn test_crossover_genotypes_produces_valid_offspring() {
        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let p1 = create_test_genotype();
        let p2 = create_test_genotype();

        let child = crossover_genotypes(&p1, &p2, &mut rng);

        assert_eq!(child.lineage_id, p1.lineage_id);
        assert!(child.sensing_range == p1.sensing_range || child.sensing_range == p2.sensing_range);
        assert!(child.max_speed == p1.max_speed || child.max_speed == p2.max_speed);
        assert!(!child.brain.nodes.is_empty());
        assert!(!child.brain.connections.is_empty());
    }

    #[test]
    fn test_crossover_brains_preserves_innovation_alignment() {
        let mut rng = ChaCha8Rng::seed_from_u64(456);
        let p1 = Brain::new_random_with_rng(&mut rng);
        let mut rng = ChaCha8Rng::seed_from_u64(789);
        let p2 = Brain::new_random_with_rng(&mut rng);

        let mut rng = ChaCha8Rng::seed_from_u64(111);
        let child = crossover_brains(&p1, &p2, &mut rng);

        assert!(!child.nodes.is_empty());
        assert!(child.next_node_id >= p1.next_node_id.min(p2.next_node_id));

        for conn in &child.connections {
            assert!(child.nodes.iter().any(|n| n.id == conn.from),);
            assert!(child.nodes.iter().any(|n| n.id == conn.to),);
        }
    }

    #[test]
    fn test_crossover_determinism_with_same_seed() {
        let p1 = create_test_genotype();
        let p2 = create_test_genotype();

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let child1 = crossover_genotypes(&p1, &p2, &mut rng1);

        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let child2 = crossover_genotypes(&p1, &p2, &mut rng2);

        assert_eq!(child1.sensing_range, child2.sensing_range);
        assert_eq!(child1.max_speed, child2.max_speed);
        assert_eq!(child1.trophic_potential, child2.trophic_potential);
        assert_eq!(
            child1.brain.connections.len(),
            child2.brain.connections.len()
        );
    }

    fn count_weight_differences(b1: &Brain, b2: &Brain) -> usize {
        let mut differences = 0;
        for (c1, c2) in b1.connections.iter().zip(b2.connections.iter()) {
            if (c1.weight - c2.weight).abs() > 0.001 {
                differences += 1;
            }
        }
        differences
    }
}
