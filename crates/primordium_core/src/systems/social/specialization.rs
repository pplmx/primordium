use crate::config::AppConfig;
use primordium_data::{Intel, Specialization};

pub fn increment_spec_meter_components(
    intel: &mut Intel,
    spec: Specialization,
    amount: f32,
    config: &AppConfig,
) {
    if intel.specialization.is_none() {
        let bias_idx = match spec {
            Specialization::Soldier => 0,
            Specialization::Engineer => 1,
            Specialization::Provider => 2,
        };
        let meter = intel.spec_meters.entry(spec).or_insert(0.0);
        *meter += amount * (1.0 + intel.genotype.specialization_bias[bias_idx]);
        if *meter >= config.social.specialization_threshold {
            intel.specialization = Some(spec);
        }
    }
}
