use primordium_data::{Entity, Pathogen};
use uuid::Uuid;

#[derive(Debug)]
pub enum InteractionCommand {
    Kill {
        target_idx: usize,
        attacker_idx: usize,
        attacker_lineage: Uuid,
        cause: String,
    },
    TransferEnergy {
        target_idx: usize,
        amount: f64,
    },
    Birth {
        parent_idx: usize,
        baby: Box<Entity>,
        genetic_distance: f32,
    },
    EatFood {
        food_index: usize,
        attacker_idx: usize,
        x: f64,
        y: f64,
    },
    Infect {
        target_idx: usize,
        pathogen: Pathogen,
    },
    Fertilize {
        x: f64,
        y: f64,
        amount: f32,
    },
    UpdateReputation {
        target_idx: usize,
        delta: f32,
    },
    TribalSplit {
        target_idx: usize,
        new_color: (u8, u8, u8),
    },
    TribalTerritory {
        x: f64,
        y: f64,
        is_war: bool,
    },
    Bond {
        target_idx: usize,
        partner_id: Uuid,
    },
    BondBreak {
        target_idx: usize,
    },
    Dig {
        x: f64,
        y: f64,
        attacker_idx: usize,
    },
    Build {
        x: f64,
        y: f64,
        attacker_idx: usize,
        is_nest: bool,
        is_outpost: bool,
    },
    Metamorphosis {
        target_idx: usize,
    },
}
