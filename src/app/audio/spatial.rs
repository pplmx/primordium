pub struct SpatialAudio;

impl SpatialAudio {
    pub fn calculate_stereo_panning(
        world_x: f64,
        world_y: f64,
        world_width: u16,
        world_height: u16,
    ) -> (f32, f32) {
        let norm_x = world_x / world_width as f64;
        let _norm_y = world_y / world_height as f64;

        let pan = (norm_x - 0.5) * 2.0;
        let angle = pan as f32 * std::f32::consts::FRAC_PI_4;
        let left = (angle.cos() - angle.sin()) * 0.707;
        let right = (angle.cos() + angle.sin()) * 0.707;

        (left, right)
    }

    pub fn apply_distance_attenuation(sample: f32, distance: f64, max_distance: f64) -> f32 {
        let normalized_distance = (distance / max_distance).min(1.0);
        let attenuation = 1.0 / (1.0 + normalized_distance * normalized_distance);
        sample * attenuation as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panning_left() {
        let (left, right) = SpatialAudio::calculate_stereo_panning(0.0, 50.0, 100, 100);
        assert!(
            left > right,
            "Left channel should be louder when position is left"
        );
    }

    #[test]
    fn test_panning_right() {
        let (left, right) = SpatialAudio::calculate_stereo_panning(100.0, 50.0, 100, 100);
        assert!(
            left < right,
            "Right channel should be louder when position is right"
        );
    }

    #[test]
    fn test_panning_center() {
        let (left, right) = SpatialAudio::calculate_stereo_panning(50.0, 50.0, 100, 100);
        assert!(f32::abs(left - right) < 0.01);
    }

    #[test]
    fn test_attenuation_near() {
        let attenuated = SpatialAudio::apply_distance_attenuation(1.0, 10.0, 100.0);
        assert!(attenuated > 0.8);
    }

    #[test]
    fn test_attenuation_far() {
        let attenuated = SpatialAudio::apply_distance_attenuation(1.0, 90.0, 100.0);
        assert!(attenuated < 0.8, "Far distance should attenuate more");
    }

    #[test]
    fn test_attenuation_clamped() {
        let attenuated = SpatialAudio::apply_distance_attenuation(1.0, 200.0, 100.0);
        assert!(attenuated > 0.0);
    }
}
