use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Specialization {
    Soldier,
    Engineer,
    Provider,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AncestralTrait {
    HardenedMetabolism, // Lower idle cost
    AcuteSenses,        // Higher sensing range
    SwiftMovement,      // Higher max speed
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LineageGoal {
    Expansion,
    Dominance,
    Resilience,
}
