use crate::model::brain::Brain;
use primordium_data::{AncestralTrait, Specialization};
use rand::Rng;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EntityStatus {
    Starving,
    Larva,
    Juvenile,
    Infected,
    Sharing,
    Mating,
    Hunting,
    Foraging,
    Soldier,
    Bonded,
    InTransit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Physics {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub symbol: char,
    pub home_x: f64,
    pub home_y: f64,
    pub sensing_range: f64,
    pub max_speed: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metabolism {
    pub trophic_potential: f32,
    pub energy: f64,
    #[serde(skip)]
    pub prev_energy: f64,
    pub max_energy: f64,
    pub peak_energy: f64,
    pub birth_tick: u64,
    pub generation: u32,
    pub offspring_count: u32,
    pub lineage_id: Uuid,
    pub has_metamorphosed: bool,
    #[serde(default)]
    pub is_in_transit: bool,
    #[serde(default)]
    pub migration_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub pathogen: Option<crate::model::state::pathogen::Pathogen>,
    pub infection_timer: u32,
    pub immunity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intel {
    pub genotype: Genotype,
    #[serde(skip)]
    pub last_hidden: [f32; 6],
    #[serde(skip)]
    pub last_aggression: f32,
    pub last_share_intent: f32,
    pub last_signal: f32,
    pub last_vocalization: f32,
    pub reputation: f32,
    pub rank: f32,
    pub bonded_to: Option<Uuid>,
    #[serde(skip)]
    pub last_inputs: [f32; 29],
    #[serde(skip)]
    pub last_activations: std::collections::HashMap<i32, f32>,
    pub specialization: Option<Specialization>,
    pub spec_meters: std::collections::HashMap<Specialization, f32>,
    pub ancestral_traits: std::collections::HashSet<AncestralTrait>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genotype {
    pub brain: Brain,
    pub sensing_range: f64,
    pub max_speed: f64,
    pub max_energy: f64,
    pub lineage_id: Uuid,
    pub metabolic_niche: f32,
    pub trophic_potential: f32,
    pub reproductive_investment: f32,
    pub maturity_gene: f32,
    pub mate_preference: f32,
    pub pairing_bias: f32,
    pub specialization_bias: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub physics: Physics,
    pub metabolism: Metabolism,
    pub health: Health,
    pub intel: Intel,
}

impl Entity {
    pub fn new_with_rng<R: Rng>(x: f64, y: f64, tick: u64, rng: &mut R) -> Self {
        let genotype = Genotype::new_random_with_rng(rng);
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            physics: Physics {
                x,
                y,
                vx: 0.0,
                vy: 0.0,
                r: 100,
                g: 200,
                b: 100,
                symbol: '●',
                home_x: x,
                home_y: y,
                sensing_range: genotype.sensing_range,
                max_speed: genotype.max_speed,
            },
            metabolism: Metabolism {
                trophic_potential: 0.5,
                energy: 100.0,
                prev_energy: 100.0,
                max_energy: 100.0,
                peak_energy: 100.0,
                birth_tick: tick,
                generation: 0,
                offspring_count: 0,
                lineage_id: genotype.lineage_id,
                has_metamorphosed: false,
                is_in_transit: false,
                migration_id: None,
            },
            health: Health {
                pathogen: None,
                infection_timer: 0,
                immunity: 0.0,
            },
            intel: Intel {
                genotype,
                last_hidden: [0.0; 6],
                last_aggression: 0.0,
                last_share_intent: 0.0,
                last_signal: 0.0,
                last_vocalization: 0.0,
                reputation: 1.0,
                rank: 0.5,
                bonded_to: None,
                last_inputs: [0.0; 29],
                last_activations: std::collections::HashMap::new(),
                specialization: None,
                spec_meters: std::collections::HashMap::new(),
                ancestral_traits: std::collections::HashSet::new(),
            },
        }
    }

    pub fn new(x: f64, y: f64, tick: u64) -> Self {
        let mut rng = rand::thread_rng();
        Self::new_with_rng(x, y, tick, &mut rng)
    }

    pub fn color(&self) -> Color {
        Color::Rgb(self.physics.r, self.physics.g, self.physics.b)
    }

    pub fn status(&self, threshold: f32, current_tick: u64, maturity_age: u64) -> EntityStatus {
        if self.metabolism.is_in_transit {
            return EntityStatus::InTransit;
        }

        let actual_maturity = (maturity_age as f32 * self.intel.genotype.maturity_gene) as u64;
        if self.metabolism.energy / self.metabolism.max_energy < 0.2 {
            EntityStatus::Starving
        } else if self.health.pathogen.is_some() {
            EntityStatus::Infected
        } else if !self.metabolism.has_metamorphosed {
            EntityStatus::Larva
        } else if (current_tick - self.metabolism.birth_tick) < actual_maturity {
            EntityStatus::Juvenile
        } else if self.intel.bonded_to.is_some() {
            EntityStatus::Bonded
        } else if self.intel.last_share_intent > threshold
            && self.metabolism.energy > self.metabolism.max_energy * 0.7
        {
            EntityStatus::Sharing
        } else if self.intel.rank > 0.8 && self.intel.last_aggression > threshold {
            EntityStatus::Soldier
        } else if self.intel.last_aggression > threshold {
            EntityStatus::Hunting
        } else {
            EntityStatus::Foraging
        }
    }

    pub fn symbol_for_status(&self, status: EntityStatus) -> char {
        match status {
            EntityStatus::Starving => '†',
            EntityStatus::Infected => '☣',
            EntityStatus::Larva => '⋯',
            EntityStatus::Juvenile => '◦',
            EntityStatus::Sharing => '♣',
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
            EntityStatus::Soldier => '⚔',
            EntityStatus::Bonded => '⚭',
            EntityStatus::InTransit => '✈',
        }
    }

    pub fn color_for_status(&self, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50),
            EntityStatus::Infected => Color::Rgb(154, 205, 50),
            EntityStatus::Larva => Color::Rgb(180, 180, 180),
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255),
            EntityStatus::Sharing => Color::Rgb(100, 200, 100),
            EntityStatus::Mating => Color::Rgb(255, 105, 180),
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),
            EntityStatus::Foraging => self.color(),
            EntityStatus::Soldier => Color::Red,
            EntityStatus::Bonded => Color::Rgb(255, 215, 0),
            EntityStatus::InTransit => Color::Rgb(150, 150, 150),
        }
    }

    pub fn is_mature(&self, current_tick: u64, maturity_age: u64) -> bool {
        let actual_maturity = (maturity_age as f32 * self.intel.genotype.maturity_gene) as u64;
        (current_tick - self.metabolism.birth_tick) >= actual_maturity
    }

    pub fn name(&self) -> String {
        let id_str = self.id.to_string();
        let bytes = id_str.as_bytes();
        let syllables = [
            "ae", "ba", "co", "da", "el", "fa", "go", "ha", "id", "jo", "ka", "lu", "ma", "na",
            "os", "pe", "qu", "ri", "sa", "tu", "vi", "wu", "xi", "yo", "ze",
        ];
        let prefix = [
            "Aethel", "Bel", "Cor", "Dag", "Eld", "Fin", "Grom", "Had", "Ith", "Jor", "Kael",
            "Luv", "Mor", "Nar", "Oth", "Pyr", "Quas", "Rhun", "Syl", "Tor", "Val", "Wun", "Xer",
            "Yor", "Zan",
        ];
        let p_idx = (bytes[0] as usize) % prefix.len();
        let s1_idx = (bytes[1] as usize) % syllables.len();
        let s2_idx = (bytes[2] as usize) % syllables.len();
        let tp = self.metabolism.trophic_potential;
        let role_prefix = if tp < 0.25 {
            "H-"
        } else if tp < 0.45 {
            "hO-"
        } else if tp < 0.55 {
            "O-"
        } else if tp < 0.75 {
            "cO-"
        } else {
            "C-"
        };
        format!(
            "{}{}{}{}-Gen{}",
            role_prefix,
            prefix[p_idx],
            syllables[s1_idx],
            syllables[s2_idx],
            self.metabolism.generation
        )
    }
}

impl Genotype {
    pub fn crossover_with_rng<R: Rng>(&self, other: &Self, rng: &mut R) -> Self {
        let brain = self.brain.crossover_with_rng(&other.brain, rng);

        let mut child_genotype = if rng.gen_bool(0.5) {
            self.clone()
        } else {
            other.clone()
        };
        child_genotype.brain = brain;
        child_genotype
    }

    pub fn crossover(&self, other: &Self) -> Self {
        let mut rng = rand::thread_rng();
        self.crossover_with_rng(other, &mut rng)
    }

    pub fn to_hex(&self) -> String {
        let bytes = serde_json::to_vec(self).unwrap_or_default();
        hex::encode(bytes)
    }

    pub fn from_hex(hex_str: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let genotype = serde_json::from_slice(&bytes)?;
        Ok(genotype)
    }

    pub fn distance(&self, other: &Self) -> f32 {
        self.brain.distance(&other.brain)
    }

    pub fn relatedness(&self, other: &Self) -> f32 {
        let dist = self.distance(other);
        (1.0 - (dist / 10.0)).clamp(0.0, 1.0)
    }

    pub fn new_random_with_rng<R: Rng>(rng: &mut R) -> Self {
        let brain = Brain::new_random_with_rng(rng);
        Self {
            brain,
            sensing_range: 10.0,
            max_speed: 1.0,
            max_energy: 100.0,
            lineage_id: Uuid::new_v4(),
            metabolic_niche: 0.5,
            trophic_potential: 0.5,
            reproductive_investment: 0.5,
            maturity_gene: 1.0,
            mate_preference: 0.5,
            pairing_bias: 0.5,
            specialization_bias: [0.33, 0.33, 0.34],
        }
    }

    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new_random_with_rng(&mut rng)
    }
}
