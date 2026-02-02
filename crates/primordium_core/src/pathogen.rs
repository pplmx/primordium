use primordium_data::Pathogen;
use rand::Rng;
use uuid::Uuid;

pub fn create_random_pathogen_with_rng<R: Rng>(rng: &mut R) -> Pathogen {
    let manipulation = if rng.gen_bool(0.3) {
        Some((rng.gen_range(22..33), 1.0))
    } else {
        None
    };
    Pathogen {
        id: Uuid::from_u128(rng.gen::<u128>()),
        lethality: rng.gen_range(0.05..0.5),
        transmission: rng.gen_range(0.01..0.1),
        duration: rng.gen_range(200..800),
        virulence: rng.gen_range(0.5..1.5),
        behavior_manipulation: manipulation,
    }
}

pub fn create_random_pathogen() -> Pathogen {
    let mut rng = rand::thread_rng();
    create_random_pathogen_with_rng(&mut rng)
}

pub fn mutate_pathogen_with_rng<R: Rng>(pathogen: &mut Pathogen, rng: &mut R) {
    pathogen.lethality = (pathogen.lethality + rng.gen_range(-0.02..0.02)).clamp(0.01, 1.0);
    pathogen.transmission = (pathogen.transmission + rng.gen_range(-0.01..0.01)).clamp(0.005, 0.2);
    if rng.gen_bool(0.1) {
        pathogen.behavior_manipulation = Some((rng.gen_range(22..33), 1.0));
    }
}

pub fn mutate_pathogen(pathogen: &mut Pathogen) {
    let mut rng = rand::thread_rng();
    mutate_pathogen_with_rng(pathogen, &mut rng);
}
