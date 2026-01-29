use primordium_data::Pathogen;
use rand::Rng;
use uuid::Uuid;

pub fn create_random_pathogen() -> Pathogen {
    let mut rng = rand::thread_rng();
    let manipulation = if rng.gen_bool(0.3) {
        Some((rng.gen_range(22..33), 1.0))
    } else {
        None
    };
    Pathogen {
        id: Uuid::new_v4(),
        lethality: rng.gen_range(0.05..0.5),
        transmission: rng.gen_range(0.01..0.1),
        duration: rng.gen_range(200..800),
        virulence: rng.gen_range(0.5..1.5),
        behavior_manipulation: manipulation,
    }
}

pub fn mutate_pathogen(pathogen: &mut Pathogen) {
    let mut rng = rand::thread_rng();
    pathogen.lethality = (pathogen.lethality + rng.gen_range(-0.02..0.02)).clamp(0.01, 1.0);
    pathogen.transmission = (pathogen.transmission + rng.gen_range(-0.01..0.01)).clamp(0.005, 0.2);
    if rng.gen_bool(0.1) {
        pathogen.behavior_manipulation = Some((rng.gen_range(22..33), 1.0));
    }
}
