use crate::config::AppConfig;
use primordium_data::{Intel, Metabolism, Physics};
use rand::Rng;

pub fn calculate_social_rank_components(
    metabolism: &Metabolism,
    intel: &Intel,
    tick: u64,
    config: &AppConfig,
) -> f32 {
    let energy_score = (metabolism.energy / metabolism.max_energy).clamp(0.0, 1.0) as f32;
    let age = tick - metabolism.birth_tick;

    // Age with decay: peaks at age_rank_normalization * 0.7, then slowly declines
    // This prevents "elder exploit" where old age alone guarantees high rank
    let peak_age = config.social.age_rank_normalization * 0.7;
    let age_score_raw = (age as f32 / config.social.age_rank_normalization).min(1.0);
    let age_score = if age as f32 > peak_age {
        let excess = (age as f32 - peak_age) / (config.social.age_rank_normalization - peak_age);
        (1.0 - excess.powi(2)).max(0.0) * age_score_raw
    } else {
        age_score_raw
    };

    let offspring_score =
        (metabolism.offspring_count as f32 / config.social.offspring_rank_normalization).min(1.0);
    let rep_score = intel.reputation.clamp(0.0, 1.0);

    let w = config.social.rank_weights;
    w[0] * energy_score + w[1] * age_score + w[2] * offspring_score + w[3] * rep_score
}

pub fn start_tribal_split_components<R: Rng>(
    _phys: &Physics,
    _met: &Metabolism,
    intel: &Intel,
    crowding: f32,
    config: &AppConfig,
    rng: &mut R,
) -> Option<(u8, u8, u8)> {
    // Redesigned split condition: High rank + High crowding = Alpha-led migration
    // This ensures the fittest lead the new tribe, not the weakest
    if crowding > config.evolution.crowding_threshold
        && intel.rank > config.social.sharing_threshold * 0.6
    {
        Some((
            rng.gen_range(0..255),
            rng.gen_range(0..255),
            rng.gen_range(0..255),
        ))
    } else {
        None
    }
}

pub fn are_same_tribe_components(phys1: &Physics, phys2: &Physics, config: &AppConfig) -> bool {
    let dist = (phys1.r as i32 - phys2.r as i32).abs()
        + (phys1.g as i32 - phys2.g as i32).abs()
        + (phys1.b as i32 - phys2.b as i32).abs();
    dist < config.social.tribe_color_threshold
}
