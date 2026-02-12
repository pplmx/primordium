use crate::brain::BrainLogic;
use primordium_data::Brain;
use rand::Rng;

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

pub struct GrnContext<'a> {
    pub genotype: &'a primordium_data::Genotype,
    pub metabolism: &'a primordium_data::Metabolism,
    pub oxygen_level: f64,
    pub carbon_level: f64,
    pub nearby_kin: usize,
    pub tick: u64,
}

pub fn apply_grn_rules(ctx: GrnContext) -> (f64, f64, f32) {
    let mut speed_mod = 1.0;
    let mut sensing_mod = 1.0;
    let mut repro_mod = 1.0;

    let energy_ratio = (ctx.metabolism.energy / ctx.metabolism.max_energy) as f32;
    let age_ratio = ((ctx.tick - ctx.metabolism.birth_tick) as f32 / 2000.0).min(1.0);

    for rule in &ctx.genotype.regulatory_rules {
        use primordium_data::{GeneType, RegulatoryOperator, RegulatorySensor};
        let sensor_value = match rule.sensor {
            RegulatorySensor::Oxygen => (ctx.oxygen_level / 21.0) as f32,
            RegulatorySensor::Carbon => (ctx.carbon_level / 1000.0) as f32,
            RegulatorySensor::EnergyRatio => energy_ratio,
            RegulatorySensor::NearbyKin => (ctx.nearby_kin as f32 / 10.0).min(1.0),
            RegulatorySensor::AgeRatio => age_ratio,
            RegulatorySensor::Clock => (ctx.tick % 1000) as f32 / 1000.0,
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

pub struct MutationParams<'a> {
    pub config: &'a crate::config::AppConfig,
    pub population: usize,
    pub is_radiation_storm: bool,
    pub specialization: Option<primordium_data::Specialization>,
    pub ancestral_genotype: Option<&'a primordium_data::Genotype>,
    pub stress_factor: f32,
}

pub fn mutate_genotype<R: Rng>(
    genotype_arc: &mut std::sync::Arc<primordium_data::Genotype>,
    params: &MutationParams<'_>,
    rng: &mut R,
) {
    let genotype = std::sync::Arc::make_mut(genotype_arc);
    let mut effective_mutation_rate = params.config.evolution.mutation_rate;
    let mut effective_mutation_amount = params.config.evolution.mutation_amount;

    if params.is_radiation_storm {
        effective_mutation_rate *= 5.0;
        effective_mutation_amount *= 2.0;
    }

    if let Some(ancestral) = params.ancestral_genotype {
        let recall_chance = 0.05 + (params.stress_factor * 0.10);
        if rng.gen_bool(recall_chance as f64) {
            genotype.brain = ancestral.brain.clone();
            genotype.sensing_range = ancestral.sensing_range;
            genotype.max_speed = ancestral.max_speed;
            genotype.maturity_gene = ancestral.maturity_gene;
            crate::brain::BrainLogic::initialize_node_idx_map(&mut genotype.brain);
        }
    }

    if params.config.evolution.population_aware && params.population > 0 {
        if params.population < params.config.evolution.bottleneck_threshold {
            let scaling = (params.config.evolution.bottleneck_threshold as f32
                / (params.population as f32).max(1.0))
            .min(3.0);
            effective_mutation_rate *= scaling;
            effective_mutation_amount *= scaling;
        } else if params.population > params.config.evolution.stasis_threshold {
            effective_mutation_rate *= 0.5;
        }
    }

    let mut brain_config = params.config.clone();
    brain_config.evolution.mutation_rate = effective_mutation_rate;
    brain_config.evolution.mutation_amount = effective_mutation_amount;
    mutate_brain(
        &mut genotype.brain,
        &brain_config,
        params.specialization,
        rng,
    );

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

    if params.population < 10 && params.population > 0 && rng.gen_bool(0.05) {
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
    let brain = p1.brain.crossover_with_rng(&p2.brain, rng);

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

    fn count_weight_differences(b1: &Brain, b2: &Brain) -> usize {
        let mut differences = 0;
        for (c1, c2) in b1.connections.iter().zip(b2.connections.iter()) {
            if (c1.weight - c2.weight).abs() > 0.001 {
                differences += 1;
            }
        }
        differences
    }

    #[test]
    fn test_mutate_genotype_respects_clamping_bounds() {
        let mut rng = ChaCha8Rng::seed_from_u64(12345);
        let config = create_test_config();
        let mut genotype = std::sync::Arc::new(create_test_genotype());

        for _ in 0..100 {
            mutate_genotype(
                &mut genotype,
                &MutationParams {
                    config: &config,
                    population: 100,
                    is_radiation_storm: false,
                    specialization: None,
                    ancestral_genotype: None,
                    stress_factor: 0.0,
                },
                &mut rng,
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

        let mut genotype_bottleneck = std::sync::Arc::new(create_test_genotype());
        let original_brain_bottleneck = genotype_bottleneck.brain.clone();

        let mut genotype_large = std::sync::Arc::new(create_test_genotype());
        let original_brain_large = genotype_large.brain.clone();

        mutate_genotype(
            &mut genotype_bottleneck,
            &MutationParams {
                config: &config,
                population: 5,
                is_radiation_storm: false,
                specialization: None,
                ancestral_genotype: None,
                stress_factor: 0.0,
            },
            &mut rng1,
        );

        mutate_genotype(
            &mut genotype_large,
            &MutationParams {
                config: &config,
                population: 1000,
                is_radiation_storm: false,
                specialization: None,
                ancestral_genotype: None,
                stress_factor: 0.0,
            },
            &mut rng2,
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

        let mut genotype_normal = std::sync::Arc::new(create_test_genotype());
        let mut genotype_storm = std::sync::Arc::new(create_test_genotype());

        mutate_genotype(
            &mut genotype_normal,
            &MutationParams {
                config: &config,
                population: 100,
                is_radiation_storm: false,
                specialization: None,
                ancestral_genotype: None,
                stress_factor: 0.0,
            },
            &mut rng1,
        );

        mutate_genotype(
            &mut genotype_storm,
            &MutationParams {
                config: &config,
                population: 100,
                is_radiation_storm: true,
                specialization: None,
                ancestral_genotype: None,
                stress_factor: 0.0,
            },
            &mut rng2,
        );

        let normal_changes =
            count_weight_differences(&create_test_genotype().brain, &genotype_normal.brain);
        let storm_changes =
            count_weight_differences(&create_test_genotype().brain, &genotype_storm.brain);

        assert!(storm_changes >= normal_changes);
    }

    #[test]
    fn test_mutate_genotype_genetic_drift_in_small_population() {
        let config = create_test_config();

        let mut drift_occurred = false;
        for seed in 0..500 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut genotype = std::sync::Arc::new(create_test_genotype());
            let original_trophic = genotype.trophic_potential;
            let original_niche = genotype.metabolic_niche;
            let original_mate = genotype.mate_preference;
            let original_maturity = genotype.maturity_gene;
            let original_pairing = genotype.pairing_bias;

            mutate_genotype(
                &mut genotype,
                &MutationParams {
                    config: &config,
                    population: 5,
                    is_radiation_storm: false,
                    specialization: None,
                    ancestral_genotype: None,
                    stress_factor: 0.0,
                },
                &mut rng,
            );

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
            let mut genotype = std::sync::Arc::new(create_test_genotype());
            std::sync::Arc::make_mut(&mut genotype).brain.learning_rate = 0.0;

            mutate_genotype(
                &mut genotype,
                &MutationParams {
                    config: &config,
                    population: 100,
                    is_radiation_storm: false,
                    specialization: None,
                    ancestral_genotype: Some(&ancestral),
                    stress_factor: 1.0, // Full stress for max chance
                },
                &mut rng,
            );

            if (genotype.brain.learning_rate - 0.999).abs() < 0.001 {
                recall_occurred = true;
                break;
            }
        }

        assert!(recall_occurred);
    }
}
