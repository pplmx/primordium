use primordium_lib::model::config::AppConfig;
use primordium_lib::model::environment::Environment;
use primordium_lib::model::pathogen::Pathogen;
use primordium_lib::model::world::World;

#[test]
fn test_pathogen_transmission() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.game_mode = primordium_lib::model::config::GameMode::Cooperative; // Disable predation
    let mut world = World::new(0, config).expect("Failed to create world");
    let env = Environment::default();

    // 1. Setup Infected Patient Zero
    let mut patient_zero = primordium_lib::model::entity::Entity::new(10.0, 10.0, 0);
    patient_zero.vx = 0.0;
    patient_zero.vy = 0.0;
    let pathogen = Pathogen {
        id: uuid::Uuid::new_v4(),
        lethality: 0.1,
        transmission: 1.0, // Guaranteed spread
        duration: 100,
        virulence: 2.0, // High virulence
    };
    patient_zero.pathogen = Some(pathogen.clone());
    patient_zero.infection_timer = pathogen.duration;
    world.entities.push(patient_zero);

    // 2. Setup Victim nearby (same position to be sure)
    let mut victim = primordium_lib::model::entity::Entity::new(10.0, 10.0, 0);
    victim.vx = 0.0;
    victim.vy = 0.0;
    world.entities.push(victim);

    // 3. Update world to spread infection
    world.update(&env).expect("Update failed");

    // 4. Verify victim is infected
    for (i, e) in world.entities.iter().enumerate() {
        println!("Entity {}: Infected={}", i, e.pathogen.is_some());
    }

    let infected_count = world
        .entities
        .iter()
        .filter(|e| e.pathogen.is_some())
        .count();
    assert_eq!(
        infected_count, 2,
        "Pathogen should have spread to neighbor in 1 tick"
    );
}

#[test]
fn test_immunity_gain() {
    let mut entity = primordium_lib::model::entity::Entity::new(0.0, 0.0, 0);
    let initial_immunity = entity.immunity;

    let pathogen = Pathogen {
        id: uuid::Uuid::new_v4(),
        lethality: 0.0,
        transmission: 1.0,
        duration: 1, // Rapid recovery
        virulence: 2.0,
    };

    entity.pathogen = Some(pathogen);
    entity.infection_timer = 1;

    entity.process_infection(); // timer -> 0
    entity.process_infection(); // recovered

    assert!(entity.pathogen.is_none());
    assert!(
        entity.immunity > initial_immunity,
        "Entity should gain immunity after recovery"
    );
}
