use primordium_data::{Entity, EntityStatus, Health, Intel, Metabolism, Physics};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn get_name_components(id: &Uuid, metabolism: &Metabolism) -> String {
    let id_str = id.to_string();
    let bytes = id_str.as_bytes();
    let syllables = [
        "ae", "ba", "co", "da", "el", "fa", "go", "ha", "id", "jo", "ka", "lu", "ma", "na", "os",
        "pe", "qu", "ri", "sa", "tu", "vi", "wu", "xi", "yo", "ze",
    ];
    let prefix = [
        "Aethel", "Bel", "Cor", "Dag", "Eld", "Fin", "Grom", "Had", "Ith", "Jor", "Kael", "Luv",
        "Mor", "Nar", "Oth", "Pyr", "Quas", "Rhun", "Syl", "Tor", "Val", "Wun", "Xer", "Yor",
        "Zan",
    ];
    let p_idx = (bytes[0] as usize) % prefix.len();
    let s1_idx = (bytes[1] as usize) % syllables.len();
    let s2_idx = (bytes[2] as usize) % syllables.len();
    let tp = metabolism.trophic_potential;
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
        role_prefix, prefix[p_idx], syllables[s1_idx], syllables[s2_idx], metabolism.generation
    )
}

pub fn get_name(entity: &Entity) -> String {
    get_name_components(&entity.identity.id, &entity.metabolism)
}

pub fn create_entity_with_rng<R: Rng>(x: f64, y: f64, tick: u64, rng: &mut R) -> Entity {
    let genotype = crate::brain::create_genotype_random_with_rng(rng);
    let id_u128 = rng.gen::<u128>();
    let id = Uuid::from_u128(id_u128);
    let mut entity = Entity {
        identity: primordium_data::Identity {
            id,
            name: String::new(),
            parent_id: None,
        },
        position: primordium_data::Position { x, y },
        velocity: primordium_data::Velocity { vx: 0.0, vy: 0.0 },
        appearance: primordium_data::Appearance {
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
        },
        physics: Physics {
            home_x: x,
            home_y: y,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            r: 100,
            g: 200,
            b: 100,
            symbol: '●',
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
            last_activations: primordium_data::Activations::default(),
            specialization: None,
            spec_meters: HashMap::new(),
            ancestral_traits: HashSet::new(),
        },
    };
    entity.identity.name = get_name_components(&entity.identity.id, &entity.metabolism);
    entity
}

pub fn create_entity(x: f64, y: f64, tick: u64) -> Entity {
    let mut rng = rand::thread_rng();
    create_entity_with_rng(x, y, tick, &mut rng)
}

pub fn calculate_status(
    metabolism: &Metabolism,
    health: &Health,
    intel: &Intel,
    threshold: f32,
    current_tick: u64,
    maturity_age: u64,
) -> EntityStatus {
    if metabolism.is_in_transit {
        return EntityStatus::InTransit;
    }

    let actual_maturity = (maturity_age as f32 * intel.genotype.maturity_gene) as u64;
    if metabolism.energy / metabolism.max_energy < 0.2 {
        EntityStatus::Starving
    } else if health.pathogen.is_some() {
        EntityStatus::Infected
    } else if !metabolism.has_metamorphosed {
        EntityStatus::Larva
    } else if (current_tick - metabolism.birth_tick) < actual_maturity {
        EntityStatus::Juvenile
    } else if intel.bonded_to.is_some() {
        EntityStatus::Bonded
    } else if intel.last_share_intent > threshold && metabolism.energy > metabolism.max_energy * 0.7
    {
        EntityStatus::Sharing
    } else if intel.rank > 0.8 && intel.last_aggression > threshold {
        EntityStatus::Soldier
    } else if intel.last_aggression > threshold {
        EntityStatus::Hunting
    } else {
        EntityStatus::Foraging
    }
}

pub fn get_entity_status(
    entity: &Entity,
    threshold: f32,
    current_tick: u64,
    maturity_age: u64,
) -> EntityStatus {
    calculate_status(
        &entity.metabolism,
        &entity.health,
        &entity.intel,
        threshold,
        current_tick,
        maturity_age,
    )
}

pub fn get_symbol_for_status(status: EntityStatus) -> char {
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

pub fn is_mature_components(
    metabolism: &Metabolism,
    intel: &Intel,
    current_tick: u64,
    maturity_age: u64,
) -> bool {
    let actual_maturity = (maturity_age as f32 * intel.genotype.maturity_gene) as u64;
    (current_tick - metabolism.birth_tick) >= actual_maturity
}

pub fn is_mature(entity: &Entity, current_tick: u64, maturity_age: u64) -> bool {
    is_mature_components(
        &entity.metabolism,
        &entity.intel,
        current_tick,
        maturity_age,
    )
}
