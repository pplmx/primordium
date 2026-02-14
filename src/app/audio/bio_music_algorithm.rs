use crate::app::audio::bio_music::{Note, NoteDuration};
use primordium_data::data::genotype::Genotype;

pub struct BioMusicAlgorithm;

impl BioMusicAlgorithm {
    pub fn genotype_to_melody(genotype: &Genotype) -> Vec<Note> {
        let mut melody = Vec::new();

        let base_pitch = Self::extract_base_pitch(genotype);

        let node_count = genotype.brain.nodes.len().max(4);
        for node_idx in 0..node_count {
            let pitch_offset = Self::extract_pitch_from_node(genotype, node_idx);
            let duration = Self::extract_duration_from_node(genotype, node_idx);

            melody.push(Note {
                pitch: ((base_pitch as i16 + pitch_offset as i16) % 128) as u8,
                duration,
                velocity: 0.7,
            });
        }

        melody
    }

    pub fn extract_base_pitch(genotype: &Genotype) -> u8 {
        let sensing_range = genotype.sensing_range;
        (sensing_range * 10.0) as u8 % 12
    }

    pub fn extract_tempo(genotype: &Genotype) -> u8 {
        let max_speed = genotype.max_speed;
        (60 + (max_speed * 60.0) as u8).min(180)
    }

    fn extract_pitch_from_node(genotype: &Genotype, node_idx: usize) -> i8 {
        let connections = &genotype.brain.connections;

        let incoming_weight_sum: f32 = connections
            .iter()
            .filter(|conn| conn.to == node_idx)
            .map(|conn| conn.weight.abs())
            .sum();

        if incoming_weight_sum > 0.0 {
            (incoming_weight_sum * 24.0).min(12.0) as i8 - 6
        } else {
            ((node_idx % 24) as i8) - 12
        }
    }

    fn extract_duration_from_node(genotype: &Genotype, node_idx: usize) -> NoteDuration {
        let connections = &genotype.brain.connections;

        let incoming_count = connections
            .iter()
            .filter(|conn| conn.to == node_idx)
            .count();

        match incoming_count {
            0..=2 => NoteDuration::Quarter,
            3..=5 => NoteDuration::Eighth,
            _ => NoteDuration::Sixteenth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primordium_data::data::genotype::Brain;
    use uuid::Uuid;

    fn create_test_genotype() -> Genotype {
        Genotype {
            brain: Brain {
                nodes: vec![],
                connections: vec![],
                next_node_id: 0,
                learning_rate: 0.1,
                weight_deltas: Default::default(),
                node_idx_map: Default::default(),
                topological_order: Default::default(),
                forward_connections: Default::default(),
                recurrent_connections: Default::default(),
                incoming_forward_connections: Default::default(),
                fast_forward_order: Default::default(),
                incoming_flat: Default::default(),
                incoming_offsets: Default::default(),
            },
            sensing_range: 0.5,
            max_speed: 0.5,
            max_energy: 100.0,
            lineage_id: Uuid::new_v4(),
            metabolic_niche: 0.5,
            trophic_potential: 0.5,
            reproductive_investment: 0.5,
            maturity_gene: 1.0,
            mate_preference: 0.5,
            pairing_bias: 0.5,
            regulatory_rules: Default::default(),
            specialization_bias: Default::default(),
        }
    }

    #[test]
    fn test_genotype_to_melody() {
        let genotype = create_test_genotype();

        let melody = BioMusicAlgorithm::genotype_to_melody(&genotype);

        assert!(!melody.is_empty());
        assert!(melody.len() <= 12);
    }

    #[test]
    fn test_base_pitch_range() {
        let genotype = create_test_genotype();
        let base_pitch = BioMusicAlgorithm::extract_base_pitch(&genotype);
        assert!(base_pitch < 12);
    }

    #[test]
    fn test_tempo_range() {
        let genotype = create_test_genotype();
        let tempo = BioMusicAlgorithm::extract_tempo(&genotype);
        assert!((60..=180).contains(&tempo));
    }
}
